use entity::sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AppErrorDto {
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ScreenDto {
    pub id: i32,
    pub name: String,
    pub position: i32,
}

#[cfg(feature = "entity")]
impl From<entity::screen::Model> for ScreenDto {
    fn from(screen: entity::screen::Model) -> Self {
        Self {
            id: screen.id,
            name: screen.name,
            position: screen.position,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateScreenDto {
    pub name: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SlideGroupDto {
    pub id: i32,
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub created_by: String,
    pub start_date: DateTimeUtc,
    pub end_date: Option<DateTimeUtc>,
    pub archive_date: Option<DateTimeUtc>,
    pub published: bool,
    // TODO: slide group
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateSlideGroupDto {
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub start_date: DateTimeUtc,
    pub end_date: Option<DateTimeUtc>,
}
