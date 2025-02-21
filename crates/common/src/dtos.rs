use std::collections::HashMap;

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
pub struct CreatedDto {
    pub id: i32,
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
    pub slides: Vec<SlideDto>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateSlideGroupDto {
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub start_date: DateTimeUtc,
    pub end_date: Option<DateTimeUtc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateSlideDto {
    pub position: i32,
    pub slide_group: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct MoveSlidesDto {
    pub new_positions: HashMap<i32, i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SlideDto {
    pub id: i32,
    pub position: i32,
    pub archive_date: Option<DateTimeUtc>,
    pub content: Vec<ContentDto>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateContentDto {
    pub slide: i32,
    pub screen: i32,
    pub content_type: ContentType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ContentDto {
    pub id: i32,
    pub screen: i32,
    pub content_type: ContentType,
    pub file_path: String,
    pub archive_date: Option<DateTimeUtc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContentType {
    Html,
    Image,
    Video,
}

impl From<ContentType> for entity::sea_orm_active_enums::ContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::Html => Self::Html,
            ContentType::Image => Self::Image,
            ContentType::Video => Self::Video,
        }
    }
}

impl From<entity::sea_orm_active_enums::ContentType> for ContentType {
    fn from(value: entity::sea_orm_active_enums::ContentType) -> Self {
        match value {
            entity::sea_orm_active_enums::ContentType::Html => Self::Html,
            entity::sea_orm_active_enums::ContentType::Image => Self::Image,
            entity::sea_orm_active_enums::ContentType::Video => Self::Video,
        }
    }
}
