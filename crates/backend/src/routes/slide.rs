use common::dtos::{CreateSlideDto, CreatedDto, MoveSlidesDto};
use rocket::{http::Status, serde::json::Json};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use sea_orm_rocket::Connection;

use crate::{error::AppError, pool::Db, session::User};

#[post("/slide", data = "<slide>")]
pub async fn create_slide(
    _user: User, // for access control only
    conn: Connection<'_, Db>,
    slide: Json<CreateSlideDto>,
) -> Result<Json<CreatedDto>, AppError> {
    let db = conn.into_inner();

    let res = entity::slide::ActiveModel {
        position: Set(slide.position),
        group: Set(slide.slide_group),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(Json(CreatedDto { id: res.id }))
}

#[post("/slide/bulk-move", data = "<positions>")]
pub async fn bulk_move_slides(
    _user: User, // for access control only
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

#[cfg(test)]
mod tests {
    use common::dtos::{
        AppErrorDto, CreateSlideDto, CreateSlideGroupDto, CreatedDto, MoveSlidesDto, SlideDto,
        SlideGroupDto,
    };
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use sea_orm::prelude::DateTimeUtc;

    fn util_create_slide_group(client: &Client) {
        let response = client
            .post("/api/slide-group")
            .json(&CreateSlideGroupDto {
                title: "Lorem Ipsum".to_string(),
                priority: 0,
                hidden: false,
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: None,
            })
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        assert_eq!(response.into_string(), None);
    }

    fn util_create_slide(client: &Client, id: i32, position: i32) {
        let response = client
            .post("/api/slide")
            .json(&CreateSlideDto {
                position,
                slide_group: 1,
            })
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json(), Some(CreatedDto { id }));
    }

    #[test]
    fn create_slides_and_list_slide_groups() {
        let client = Client::tracked(crate::rocket()).unwrap();

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
                created_by: "johndoe".to_string(), // TODO
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
        let client = Client::tracked(crate::rocket()).unwrap();

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
                created_by: "johndoe".to_string(), // TODO
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
        let client = Client::tracked(crate::rocket()).unwrap();

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
