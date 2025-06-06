use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AppErrorDto {
    pub msg: String,
}

impl Display for AppErrorDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for AppErrorDto {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreatedDto {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateScreenDto {
    pub name: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct SlideGroupDto {
    pub id: i32,
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub created_by: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub archive_date: Option<DateTime<Utc>>,
    pub published: bool,
    pub slides: Vec<SlideDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateSlideGroupDto {
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateSlideDto {
    pub position: i32,
    pub slide_group: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MoveSlidesDto {
    pub new_positions: HashMap<i32, i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct SlideDto {
    pub id: i32,
    pub position: i32,
    pub archive_date: Option<DateTime<Utc>>,
    pub content: Vec<ContentDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateContentDto {
    pub slide: i32,
    pub screen: i32,
    pub content_type: ContentType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct ContentDto {
    pub id: i32,
    pub screen: i32,
    pub content_type: ContentType,
    pub file_path: String,
    pub archive_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ContentType {
    Html,
    #[default]
    Image,
    Video,
}

#[cfg(feature = "entity")]
impl From<ContentType> for entity::sea_orm_active_enums::ContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::Html => Self::Html,
            ContentType::Image => Self::Image,
            ContentType::Video => Self::Video,
        }
    }
}

#[cfg(feature = "entity")]
impl From<entity::sea_orm_active_enums::ContentType> for ContentType {
    fn from(value: entity::sea_orm_active_enums::ContentType) -> Self {
        match value {
            entity::sea_orm_active_enums::ContentType::Html => Self::Html,
            entity::sea_orm_active_enums::ContentType::Image => Self::Image,
            entity::sea_orm_active_enums::ContentType::Video => Self::Video,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FeedEntryDto {
    pub content_type: ContentType,
    pub file_path: String,
    pub duration: i32, // milliseconds
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SessionDto {
    pub username: String,
    pub is_admin: bool,
}
