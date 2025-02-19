use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
use rocket::{http::Status, serde::json::Json};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use sea_orm_rocket::Connection;

use crate::{error::AppError, pool::Db, session::User};

#[get("/slide-group")]
pub async fn list_slide_groups(
    conn: Connection<'_, Db>,
) -> Result<Json<Vec<SlideGroupDto>>, AppError> {
    let db = conn.into_inner();

    let groups = entity::slide_group::Entity::find()
        .order_by_asc(entity::slide_group::Column::Id)
        .all(db)
        .await?
        .into_iter()
        // TODO fetch slides and content
        .map(|row| SlideGroupDto {
            id: row.id,
            title: row.title,
            priority: row.priority,
            hidden: row.hidden,
            created_by: row.created_by,
            start_date: row.start_date.and_utc(),
            end_date: row.end_date.map(|d| d.and_utc()),
            archive_date: row.archive_date.map(|d| d.and_utc()),
            published: row.published,
        })
        .collect::<Vec<_>>();

    Ok(Json(groups))
}

#[post("/slide-group", data = "<slide_group>")]
pub async fn create_slide_group(
    user: User,
    conn: Connection<'_, Db>,
    slide_group: Json<CreateSlideGroupDto>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();

    entity::slide_group::ActiveModel {
        title: Set(slide_group.title.clone()),
        priority: Set(slide_group.priority),
        hidden: Set(slide_group.hidden),
        created_by: Set(user.username),
        start_date: Set(slide_group.start_date.naive_utc()),
        end_date: Set(slide_group.end_date.as_ref().map(|d| d.naive_utc())),
        archive_date: Set(None),
        published: Set(false),
        ..Default::default()
    }
    .save(db)
    .await?;

    Ok(Status::Created)
}

#[put("/slide-group/<id>", data = "<slide_group>")]
pub async fn update_slide_group(
    _user: User, // ensure logged in
    conn: Connection<'_, Db>,
    id: i32,
    slide_group: Json<CreateSlideGroupDto>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();

    entity::slide_group::ActiveModel {
        id: Set(id),
        title: Set(slide_group.title.clone()),
        priority: Set(slide_group.priority),
        hidden: Set(slide_group.hidden),
        start_date: Set(slide_group.start_date.naive_utc()),
        end_date: Set(slide_group.end_date.as_ref().map(|d| d.naive_utc())),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(Status::NoContent)
}

#[put("/slide-group/<id>/publish")]
pub async fn publish_slide_group(
    _user: User, // ensure logged in
    conn: Connection<'_, Db>,
    id: i32,
) -> Result<Status, AppError> {
    let db = conn.into_inner();

    entity::slide_group::ActiveModel {
        id: Set(id),
        published: Set(true),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use sea_orm::prelude::DateTimeUtc;

    #[test]
    fn create_and_list_slide_group() {
        let client = Client::tracked(crate::rocket()).unwrap();

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
            }])
        );
    }

    #[test]
    fn update_and_list_slide_group() {
        let client = Client::tracked(crate::rocket()).unwrap();

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

        // TODO change user for update to test created_by

        let response = client
            .put("/api/slide-group/1")
            .json(&CreateSlideGroupDto {
                title: "Lorem Ipsum".to_string(),
                priority: 1,
                hidden: false,
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: Some(DateTimeUtc::from_timestamp_nanos(1739471975000000)),
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
                priority: 1,
                hidden: false,
                created_by: "johndoe".to_string(), // TODO
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: Some(DateTimeUtc::from_timestamp_nanos(1739471975000000)),
                archive_date: None,
                published: false,
            }])
        );
    }

    #[test]
    fn publish_slide_group() {
        let client = Client::tracked(crate::rocket()).unwrap();

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

        let response = client.put("/api/slide-group/1/publish").dispatch();
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
                published: true,
            }])
        );
    }
}
