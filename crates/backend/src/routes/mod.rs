use common::dtos::CreatedDto;
use rocket::{response::status::Created, serde::json::Json};

pub mod content;
pub mod health;
pub mod screen;
pub mod screen_feed;
pub mod slide;
pub mod slide_group;

type CreatedResponse = Created<Json<CreatedDto>>;

fn build_created_response(route_prefix: &'static str, id: i32) -> CreatedResponse {
    Created::new(format!("{route_prefix}/{id}")).body(Json(CreatedDto { id }))
}
