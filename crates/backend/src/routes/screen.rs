use common::dtos::{CreateScreenDto, ScreenDto};
use rocket::{http::Status, serde::json::Json};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use sea_orm_rocket::Connection;

use crate::{error::AppError, pool::Db, session::User};

#[get("/screen")]
pub async fn list_screens(conn: Connection<'_, Db>) -> Result<Json<Vec<ScreenDto>>, AppError> {
    let db = conn.into_inner();

    let screens = entity::screen::Entity::find()
        .order_by_asc(entity::screen::Column::Position)
        .all(db)
        .await?
        .into_iter()
        .map(ScreenDto::from)
        .collect::<Vec<_>>();

    Ok(Json(screens))
}

#[post("/screen", data = "<screen>")]
pub async fn create_screen(
    _user: User,
    conn: Connection<'_, Db>,
    screen: Json<CreateScreenDto>,
) -> Result<Status, AppError> {
    let db = conn.into_inner();

    entity::screen::ActiveModel {
        name: Set(screen.name.to_string()),
        position: Set(screen.position),
        ..Default::default()
    }
    .save(db)
    .await?;

    Ok(Status::Created)
}

#[cfg(test)]
mod tests {
    use common::dtos::{CreateScreenDto, ScreenDto};
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn create_and_list_screens() {
        let client = Client::tracked(crate::rocket()).unwrap();
        macro_rules! create_screen {
            ($name: expr, $position: expr) => {
                let response = client
                    .post("/api/screen")
                    .json(&CreateScreenDto {
                        name: $name.to_string(),
                        position: $position,
                    })
                    .dispatch();
                assert_eq!(response.status(), Status::Created);
                assert_eq!(response.into_string(), None);
            };
        }

        create_screen!("Left", 0);
        create_screen!("Right", 2);
        create_screen!("Center", 1);

        let response = client.get("/api/screen").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(vec![
                ScreenDto {
                    id: 1,
                    name: "Left".to_string(),
                    position: 0,
                },
                ScreenDto {
                    id: 3,
                    name: "Center".to_string(),
                    position: 1,
                },
                ScreenDto {
                    id: 2,
                    name: "Right".to_string(),
                    position: 2,
                }
            ])
        );
    }
}
