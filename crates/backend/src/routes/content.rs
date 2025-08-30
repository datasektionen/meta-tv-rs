use common::dtos::CreateContentDto;
use rocket::{data::Capped, form::Form, fs::TempFile, serde::json::Json, State};
use sea_orm::{
    sqlx::types::chrono::Utc, ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, Set, TransactionTrait,
};
use sea_orm_rocket::Connection;

use crate::{auth::Session, error::AppError, files::Files, pool::Db};

use super::{build_created_response, CreatedResponse};

#[derive(FromForm)]
pub(crate) struct Upload<'r> {
    data: Json<CreateContentDto>,
    file: Capped<TempFile<'r>>,
}

#[post("/content", data = "<upload>")]
pub async fn create_content(
    _session: Session, // for access control only
    conn: Connection<'_, Db>,
    files: &State<Files>,
    mut upload: Form<Upload<'_>>,
) -> Result<CreatedResponse, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    // ensure screen exists
    entity::screen::Entity::find_by_id(upload.data.screen)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::ScreenNotFound)?;

    // ensure slide exists and is not archived (deleted)
    entity::slide::Entity::find_by_id(upload.data.slide)
        .join(
            JoinType::LeftJoin,
            entity::slide::Relation::SlideGroup.def(),
        )
        .filter(entity::slide::Column::ArchiveDate.is_null())
        .filter(entity::slide_group::Column::ArchiveDate.is_null())
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::SlideNotFound)?;

    // archive (delete) content already on this screen (if any)
    entity::content::Entity::update_many()
        .filter(entity::content::Column::Slide.eq(upload.data.slide))
        .filter(entity::content::Column::Screen.eq(upload.data.screen))
        .set(entity::content::ActiveModel {
            archive_date: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        })
        .exec(&txn)
        .await?;

    let file_path = files.upload_file(&mut upload.file).await?;

    let res = entity::content::ActiveModel {
        slide: Set(upload.data.slide),
        screen: Set(upload.data.screen),
        content_type: Set(upload.data.content_type.into()),
        file_path: Set(file_path
            .to_str()
            .expect("path to be valid utf-8")
            .to_string()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    // NOTE: non-existent route
    Ok(build_created_response("/api/content", res.id))
}

#[cfg(test)]
mod tests {
    use common::dtos::{
        AppErrorDto, ContentDto, ContentType, CreateContentDto, SlideDto, SlideGroupDto,
    };
    use rocket::http::{self, Status};
    use rocket::serde::json;
    use sea_orm::prelude::DateTimeUtc;

    use crate::assert_created;
    use crate::test_utils::{util_create_slide, util_create_slide_group, TestClient};

    fn util_prepare_upload(data: &CreateContentDto, file: &str) -> (http::ContentType, String) {
        // There isn't a better way to test this :/
        // https://github.com/rwf2/Rocket/issues/1591
        let ct = "multipart/form-data; boundary=X-BOUNDARY"
            .parse::<http::ContentType>()
            .unwrap();
        let body = [
            "--X-BOUNDARY",
            r#"Content-Disposition: form-data; name="data""#,
            "",
            &json::to_string(data).unwrap(),
            "--X-BOUNDARY",
            r#"Content-Disposition: form-data; name="file"; filename="foo.html""#,
            "Content-Type: text/html",
            "",
            file,
            "--X-BOUNDARY--",
            "",
        ]
        .join("\r\n");

        (ct, body)
    }

    #[test]
    fn create_content_and_list_slide_groups() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);
        util_create_slide(&client, 1, 1);

        let data = CreateContentDto {
            slide: 1,
            screen: 1,
            content_type: ContentType::Html,
        };
        let (ct, body) = util_prepare_upload(&data, "<p>hello world</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_created!(response, "/api/content", 1);

        let data = CreateContentDto {
            slide: 1,
            screen: 2,
            content_type: ContentType::Html,
        };
        let (ct, body) = util_prepare_upload(&data, "<p>lorem ipsum</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_created!(response, "/api/content", 2);

        let response = client.get("/api/slide-group").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(vec![SlideGroupDto {
                id: 1,
                title: "Lorem Ipsum".to_string(),
                priority: 0,
                hidden: false,
                created_by: "johndoe".to_string(),
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: None,
                archive_date: None,
                published: false,
                slides: vec![SlideDto {
                    id: 1,
                    position: 1,
                    archive_date: None,
                    content: vec![
                        ContentDto {
                            id: 1,
                            screen: 1,
                            content_type: ContentType::Html,
                            file_path: "82/8286230721b68e5d0e15dabf39d5938611b053c320f95ed8a4fa556fd41e7457.html".to_string(),
                            archive_date: None,
                        },
                        ContentDto {
                            id: 2,
                            screen: 2,
                            content_type: ContentType::Html,
                            file_path: "39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html".to_string(),
                            archive_date: None,
                        }
                    ]
                },],
            }])
        );

        let response = client
            .get(
                "/uploads/82/8286230721b68e5d0e15dabf39d5938611b053c320f95ed8a4fa556fd41e7457.html",
            )
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("<p>hello world</p>".to_string())
        );

        let response = client
            .get(
                "/uploads/39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html",
            )
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("<p>lorem ipsum</p>".to_string())
        );
    }

    #[test]
    fn overwrite_content_and_list_slide_groups() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);
        util_create_slide(&client, 1, 1);

        let data = CreateContentDto {
            slide: 1,
            screen: 1,
            content_type: ContentType::Html,
        };
        let (ct, body) = util_prepare_upload(&data, "<p>hello world</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_created!(response, "/api/content", 1);

        let (ct, body) = util_prepare_upload(&data, "<p>lorem ipsum</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_created!(response, "/api/content", 2);

        let response = client.get("/api/slide-group").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(vec![SlideGroupDto {
                id: 1,
                title: "Lorem Ipsum".to_string(),
                priority: 0,
                hidden: false,
                created_by: "johndoe".to_string(),
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: None,
                archive_date: None,
                published: false,
                slides: vec![SlideDto {
                    id: 1,
                    position: 1,
                    archive_date: None,
                    content: vec![ContentDto {
                        id: 2,
                        screen: 1,
                        content_type: ContentType::Html,
                        file_path: "39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html".to_string(),
                        archive_date: None,
                    }]
                },],
            }])
        );

        // previous upload still available
        let response = client
            .get(
                "/uploads/82/8286230721b68e5d0e15dabf39d5938611b053c320f95ed8a4fa556fd41e7457.html",
            )
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("<p>hello world</p>".to_string())
        );

        let response = client
            .get(
                "/uploads/39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html",
            )
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("<p>lorem ipsum</p>".to_string())
        );
    }

    // screens are created by default
    // #[test]
    // fn missing_screen() {
    //     let mut client = TestClient::new();
    //     client.login_as("johndoe", false);
    //
    //     util_create_slide_group(&client);
    //     util_create_slide(&client, 1, 1);
    //
    //     let data = CreateContentDto {
    //         slide: 1,
    //         screen: 1,
    //         content_type: ContentType::Html,
    //     };
    //     let (ct, body) = util_prepare_upload(&data, "<p>hello world</p>");
    //     let response = client.post("/api/content").header(ct).body(body).dispatch();
    //     assert_eq!(response.status(), Status::NotFound);
    //     assert_eq!(
    //         response.into_json(),
    //         Some(AppErrorDto {
    //             msg: "screen not found".to_string()
    //         })
    //     );
    // }

    #[test]
    fn missing_slide() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);


        let data = CreateContentDto {
            slide: 1,
            screen: 1,
            content_type: ContentType::Html,
        };
        let (ct, body) = util_prepare_upload(&data, "<p>hello world</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.into_json(),
            Some(AppErrorDto {
                msg: "slide not found".to_string()
            })
        );
    }

    // TODO: test archived slides/slide groups
}
