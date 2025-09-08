use common::dtos::{CreateSlideDto, MoveSlidesDto};
use rocket::{http::Status, serde::json::Json};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use sea_orm_rocket::Connection;

use crate::{auth::Session, error::AppError, pool::Db};

use super::{build_created_response, CreatedResponse};

#[post("/slide", data = "<slide>")]
pub async fn create_slide(
    _session: Session, // for access control only
    conn: Connection<'_, Db>,
    slide: Json<CreateSlideDto>,
) -> Result<CreatedResponse, AppError> {
    let db = conn.into_inner();

    let res = entity::slide::ActiveModel {
        position: Set(slide.position),
        group: Set(slide.slide_group),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // NOTE: non-existent route
    Ok(build_created_response("/api/slide", res.id))
}

#[post("/slide/bulk-move", data = "<positions>")]
pub async fn bulk_move_slides(
    _session: Session, // for access control only
    conn: Connection<'_, Db>,
    positions: Json<MoveSlidesDto>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    for (&slide_id, &new_position) in &positions.new_positions {
        let slide = entity::slide::Entity::find_by_id(slide_id)
            .one(&txn)
            .await?
            .ok_or(AppError::SlideNotFound)?;

        if slide.archive_date.is_some() {
            return Err(AppError::SlideArchived);
        }

        if slide.position != new_position {
            entity::slide::ActiveModel {
                id: Set(slide_id),
                position: Set(new_position),
                ..Default::default()
            }
            .update(&txn)
            .await?;
        }
    }

    txn.commit().await?;

    Ok(Status::NoContent)
}

#[delete("/slide/<id>")]
pub async fn delete_slide(
    _session: Session,
    id: i32,
    conn: Connection<'_, Db>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    let now = chrono::Utc::now().naive_utc();

    let slide = entity::slide::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or(AppError::SlideNotFound)?;

    if slide.archive_date.is_some() {
        return Err(AppError::SlideArchived)
    } 

    entity::slide::ActiveModel {
        id: Set(id),
        archive_date: Set(Some(now)),
        ..Default::default()
    }
    .update(&txn)
    .await?;

    txn.commit().await?;

    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use common::dtos::{AppErrorDto, MoveSlidesDto, SlideDto, SlideGroupDto};
    use rocket::http::Status;
    use sea_orm::prelude::DateTimeUtc;

    use crate::test_utils::{util_create_slide, util_create_slide_group, TestClient};

    #[test]
    fn create_slides_and_list_slide_groups() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        util_create_slide(&client, 1, 1);
        util_create_slide(&client, 2, 2);
        util_create_slide(&client, 3, 3);

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
                slides: vec![
                    SlideDto {
                        id: 1,
                        position: 1,
                        archive_date: None,
                        content: vec![],
                    },
                    SlideDto {
                        id: 2,
                        position: 2,
                        archive_date: None,
                        content: vec![],
                    },
                    SlideDto {
                        id: 3,
                        position: 3,
                        archive_date: None,
                        content: vec![],
                    }
                ],
            }])
        );
    }

    #[test]
    fn move_slides_and_list_slide_groups() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        util_create_slide(&client, 1, 1);
        util_create_slide(&client, 2, 2);
        util_create_slide(&client, 3, 3);

        let response = client
            .post("/api/slide/bulk-move")
            .json(&MoveSlidesDto {
                new_positions: [(1, 5), (2, 3)].into(),
            })
            .dispatch();
        assert_eq!(response.status(), Status::NoContent);
        assert_eq!(response.into_string(), None);

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
                slides: vec![
                    SlideDto {
                        id: 2,
                        position: 3,
                        archive_date: None,
                        content: vec![],
                    },
                    SlideDto {
                        id: 3,
                        position: 3,
                        archive_date: None,
                        content: vec![],
                    },
                    SlideDto {
                        id: 1,
                        position: 5,
                        archive_date: None,
                        content: vec![],
                    },
                ],
            }])
        );
    }

    #[test]
    fn move_non_exiting_slide() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        let response = client
            .post("/api/slide/bulk-move")
            .json(&MoveSlidesDto {
                new_positions: [(1, 5)].into(),
            })
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.into_json(),
            Some(AppErrorDto {
                msg: "slide not found".to_string()
            })
        );
    }

    // TODO: test moving archived slide
}
