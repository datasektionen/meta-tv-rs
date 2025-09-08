use common::dtos::{ContentDto, CreateSlideGroupDto, SlideDto, SlideGroupDto};
use rocket::{http::Status, serde::json::Json};
use sea_orm::{
    sqlx::types::chrono, ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait,
    QueryFilter, QueryOrder, QueryTrait, Set, TransactionTrait,
};
use sea_orm_rocket::Connection;

use crate::{auth::Session, error::AppError, pool::Db};

use super::{build_created_response, CreatedResponse};

#[get("/slide-group")]
pub async fn list_slide_groups(
    conn: Connection<'_, Db>,
) -> Result<Json<Vec<SlideGroupDto>>, AppError> {
    let db = conn.into_inner();

    // use transaction to ensure data is consistent
    let txn = db.begin().await?;

    let groups = entity::slide_group::Entity::find()
        .order_by_asc(entity::slide_group::Column::Id)
        .filter(entity::slide_group::Column::ArchiveDate.is_null())
        .all(&txn)
        .await?;

    // TODO: make this run in parallel
    let mut res = Vec::with_capacity(groups.len());
    for group in groups {
        res.push(get_slide_group_dto(group, true, &txn).await?);
    }

    Ok(Json(res))
}

#[get("/slide-group/<id>")]
pub async fn get_slide_group(
    id: i32,
    conn: Connection<'_, Db>,
) -> Result<Json<SlideGroupDto>, AppError> {
    let db = conn.into_inner();

    // use transaction to ensure data is consistent
    let txn = db.begin().await?;

    let group = entity::slide_group::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or(AppError::SlideGroupNotFound)?;

    Ok(Json(get_slide_group_dto(group, true, &txn).await?))
}

async fn get_slide_group_dto(
    group: entity::slide_group::Model,
    hide_archived: bool,
    txn: &DatabaseTransaction,
) -> Result<SlideGroupDto, AppError> {
    let slides = entity::slide::Entity::find()
        .order_by_asc(entity::slide::Column::Position)
        .order_by_asc(entity::slide::Column::Id)
        .order_by_asc(entity::content::Column::Id)
        .find_with_related(entity::content::Entity)
        .filter(entity::slide::Column::Group.eq(group.id))
        .apply_if(hide_archived.then_some(()), |query, _v| {
            query
                .filter(entity::slide::Column::ArchiveDate.is_null())
                .filter(entity::content::Column::ArchiveDate.is_null())
        })
        .all(txn)
        .await?;

    Ok(SlideGroupDto {
        id: group.id,
        title: group.title,
        priority: group.priority,
        hidden: group.hidden,
        created_by: group.created_by,
        start_date: group.start_date.and_utc(),
        end_date: group.end_date.map(|d| d.and_utc()),
        archive_date: group.archive_date.map(|d| d.and_utc()),
        published: group.published,
        slides: slides
            .into_iter()
            .map(|(slide, content)| SlideDto {
                id: slide.id,
                position: slide.position,
                archive_date: slide.archive_date.map(|d| d.and_utc()),
                content: content
                    .into_iter()
                    .map(|content| ContentDto {
                        id: content.id,
                        screen: content.screen,
                        content_type: content.content_type.into(),
                        file_path: content.file_path,
                        archive_date: content.archive_date.map(|d| d.and_utc()),
                    })
                    .collect(),
            })
            .collect(),
    })
}

#[post("/slide-group", data = "<slide_group>")]
pub async fn create_slide_group(
    session: Session,
    conn: Connection<'_, Db>,
    slide_group: Json<CreateSlideGroupDto>,
) -> Result<CreatedResponse, AppError> {
    let db = conn.into_inner();

    let group = entity::slide_group::ActiveModel {
        title: Set(slide_group.title.clone()),
        priority: Set(slide_group.priority),
        hidden: Set(slide_group.hidden),
        created_by: Set(session.username),
        start_date: Set(slide_group.start_date.naive_utc()),
        end_date: Set(slide_group.end_date.as_ref().map(|d| d.naive_utc())),
        archive_date: Set(None),
        published: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(build_created_response("/api/slide-group", group.id))
}

#[put("/slide-group/<id>", data = "<slide_group>")]
pub async fn update_slide_group(
    _session: Session, // ensure logged in
    conn: Connection<'_, Db>,
    id: i32,
    slide_group: Json<CreateSlideGroupDto>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    get_non_archived_slide_group(id, &txn).await?;

    entity::slide_group::ActiveModel {
        id: Set(id),
        title: Set(slide_group.title.clone()),
        priority: Set(slide_group.priority),
        hidden: Set(slide_group.hidden),
        start_date: Set(slide_group.start_date.naive_utc()),
        end_date: Set(slide_group.end_date.as_ref().map(|d| d.naive_utc())),
        ..Default::default()
    }
    .update(&txn)
    .await?;

    txn.commit().await?;

    Ok(Status::NoContent)
}

#[put("/slide-group/<id>/publish")]
pub async fn publish_slide_group(
    _session: Session, // ensure logged in
    conn: Connection<'_, Db>,
    id: i32,
) -> Result<Status, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    get_non_archived_slide_group(id, &txn).await?;

    entity::slide_group::ActiveModel {
        id: Set(id),
        published: Set(true),
        ..Default::default()
    }
    .update(&txn)
    .await?;

    txn.commit().await?;

    Ok(Status::NoContent)
}

#[delete("/slide-group/<id>")]
pub async fn archive_slide_group(
    _session: Session, // ensure logged in
    conn: Connection<'_, Db>,
    id: i32,
) -> Result<Status, AppError> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    get_non_archived_slide_group(id, &txn).await?;

    let now = chrono::Utc::now().naive_utc();

    entity::slide_group::ActiveModel {
        id: Set(id),
        archive_date: Set(Some(now)),
        ..Default::default()
    }
    .update(&txn)
    .await?;

    txn.commit().await?;

    Ok(Status::NoContent)
}

async fn get_non_archived_slide_group(
    id: i32,
    txn: &DatabaseTransaction,
) -> Result<entity::slide_group::Model, AppError> {
    let group = entity::slide_group::Entity::find_by_id(id)
        .one(txn)
        .await?
        .ok_or(AppError::SlideGroupNotFound)?;

    if group.archive_date.is_some() {
        Err(AppError::SlideGroupArchived)
    } else {
        Ok(group)
    }
}

#[cfg(test)]
mod tests {
    use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
    use rocket::http::Status;
    use sea_orm::prelude::DateTimeUtc;

    use crate::assert_app_error;
    use crate::error::AppError;
    use crate::test_utils::{util_create_slide_group, TestClient};

    #[test]
    fn create_and_list_slide_group() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        client.logout();

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
                slides: vec![],
            }])
        );
    }

    #[test]
    fn create_and_get_slide_group() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        client.logout();

        let response = client.get("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(SlideGroupDto {
                id: 1,
                title: "Lorem Ipsum".to_string(),
                priority: 0,
                hidden: false,
                created_by: "johndoe".to_string(),
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: None,
                archive_date: None,
                published: false,
                slides: vec![],
            })
        );
    }

    #[test]
    fn get_slide_group_not_found() {
        let client = TestClient::new();

        let response = client.get("/api/slide-group/1").dispatch();
        assert_app_error!(response, AppError::SlideGroupNotFound);
    }

    #[test]
    fn update_and_list_slide_group() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        client.login_as("not_johndoe", false);

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
                created_by: "johndoe".to_string(),
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: Some(DateTimeUtc::from_timestamp_nanos(1739471975000000)),
                archive_date: None,
                published: false,
                slides: vec![],
            }])
        );
    }

    #[test]
    fn update_slide_group_not_found() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

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
        assert_app_error!(response, AppError::SlideGroupNotFound);
    }

    #[test]
    fn update_slide_group_archived() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        let response = client.delete("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);
        assert_eq!(response.into_string(), None);

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
        assert_app_error!(response, AppError::SlideGroupArchived);
    }

    #[test]
    fn publish_slide_group() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

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
                created_by: "johndoe".to_string(),
                start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
                end_date: None,
                archive_date: None,
                published: true,
                slides: vec![],
            }])
        );
    }

    #[test]
    fn publish_slide_group_not_found() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        let response = client.put("/api/slide-group/1/publish").dispatch();
        assert_app_error!(response, AppError::SlideGroupNotFound);
    }

    #[test]
    fn publish_slide_group_archived() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        let response = client.delete("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);
        assert_eq!(response.into_string(), None);

        let response = client.put("/api/slide-group/1/publish").dispatch();
        assert_app_error!(response, AppError::SlideGroupArchived);
    }

    #[test]
    fn archive_slide_group() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        let response = client.delete("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);
        assert_eq!(response.into_string(), None);

        let response = client.get("/api/slide-group").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json(), Some(Vec::<SlideGroupDto>::new()));

        let response = client.get("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(matches!(
            response.into_json(),
            Some(SlideGroupDto {
                id: 1,
                archive_date: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn archive_slide_group_not_found() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        let response = client.delete("/api/slide-group/1").dispatch();
        assert_app_error!(response, AppError::SlideGroupNotFound);
    }

    #[test]
    fn archive_slide_group_already_archived() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        util_create_slide_group(&client);

        let response = client.delete("/api/slide-group/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);
        assert_eq!(response.into_string(), None);

        // archiving again throws error
        let response = client.delete("/api/slide-group/1").dispatch();
        assert_app_error!(response, AppError::SlideGroupArchived);
    }
}
