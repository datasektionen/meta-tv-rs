use common::dtos::{CreatedDto, LangDto};
use rocket::{response::status::Created, serde::json::Json};

pub mod auth;
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

// We need to create our own type since we can't derive `FromFormField` for `common::dtos::LangDto`.
#[derive(Copy, Clone, FromFormField, Default)]
pub enum Lang {
    En,
    #[default]
    Sv,
}

impl From<Lang> for LangDto {
    fn from(value: Lang) -> Self {
        match value {
            Lang::En => Self::En,
            Lang::Sv => Self::Sv,
        }
    }
}
