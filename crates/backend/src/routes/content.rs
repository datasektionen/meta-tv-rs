use common::dtos::{ContentDto, CreateContentDto};
use rocket::{
    data::Capped, form::Form, fs::TempFile, response::status::Created, serde::json::Json, State,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use sea_orm_rocket::Connection;

use crate::{auth::Session, error::AppError, files::Files, pool::Db};

#[derive(FromForm)]
pub(crate) struct Upload<'r> {
    data: Json<CreateContentDto>,
    file: Capped<TempFile<'r>>,
}

#[post("/content", data = "<upload>")]
pub async fn create_content(
    _session: Session,
    conn: Connection<'_, Db>,
    files: &State<Files>,
    mut upload: Form<Upload<'_>>,
) -> Result<Created<Json<ContentDto>>, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    // ensure screen exists
    entity::screen::Entity::find_by_id(upload.data.screen)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::ScreenNotFound)?;

    let key = files.upload_file(&mut upload.file).await?;

    let res = entity::content::ActiveModel {
        slide: Set(None),
        screen: Set(upload.data.screen),
        content_type: Set(upload.data.content_type.into()),
        file_path: Set(key),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    // NOTE: non-existent route
    Ok(
        Created::new(format!("/api/content/{}", res.id)).body(Json(ContentDto {
            id: res.id,
            screen: res.screen,
            content_type: upload.data.content_type,
            url: files.file_url(&res.file_path),
            archive_date: None,
        })),
    )
}

#[cfg(test)]
mod tests {
    use common::dtos::{
        AppErrorDto, ContentDto, ContentType, CreateContentDto, OwnerDto, SlideDto, SlideGroupDto,
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
            screen: 1,
            content_type: ContentType::Html,
        };
        let (ct, body) = util_prepare_upload(&data, "<p>hello world</p>");
        let response = client.post("/api/content").header(ct).body(body).dispatch();
        assert_created!(response, "/api/content", 1);

        let data = CreateContentDto {
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
                created_by: OwnerDto::User("johndoe".to_string()),
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
                            url: "82/8286230721b68e5d0e15dabf39d5938611b053c320f95ed8a4fa556fd41e7457.html".to_string(),
                            archive_date: None,
                        },
                        ContentDto {
                            id: 2,
                            screen: 2,
                            content_type: ContentType::Html,
                            url: "39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html".to_string(),
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
                created_by: OwnerDto::User("johndoe".to_string()),
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
                        url: "39/39acde5bed34e4bd9a0374f628ea4c64fb515f92a9aa981e8a8d8414ef9ad799.html".to_string(),
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
