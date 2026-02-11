//! Data transfer objects used when sending data between the frontend and backend.

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
    pub created_by: OwnerDto,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub archive_date: Option<DateTime<Utc>>,
    pub published: bool,
    pub slides: Vec<SlideDto>,
}

impl SlideGroupDto {
    pub fn is_owner(&self, user_info: &UserInfoDto) -> bool {
        user_info.is_admin
            || match &self.created_by {
                OwnerDto::User(username) => username == &user_info.username,
                OwnerDto::Group(group) => user_info
                    .memberships
                    .iter()
                    .any(|membership| membership.as_group() == group.as_group()),
            }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum OwnerDto {
    User(String),
    Group(GroupDto),
}

impl OwnerDto {
    pub fn id(&self) -> String {
        match self {
            Self::User(user) => user.clone(),
            Self::Group(group) => group.as_group(),
        }
    }

    /// Returns true if the provided user info is considered an owner of this.
    pub fn is_owner(&self, user_info: &UserInfoDto) -> bool {
        user_info.is_admin
            || match &self {
                OwnerDto::User(username) => username == &user_info.username,
                OwnerDto::Group(group) => user_info
                    .memberships
                    .iter()
                    .any(|membership| membership.as_group() == group.as_group()),
            }
    }
}

impl Default for OwnerDto {
    fn default() -> Self {
        Self::User(String::default())
    }
}

/// Represents a group in Hive.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub struct GroupDto {
    pub name: String,
    pub id: String,
    pub domain: String,
}

impl GroupDto {
    /// Return self as group identifier syntax: `id@domain`
    pub fn as_group(&self) -> String {
        format!("{}@{}", self.id, self.domain)
    }
}

impl From<TaggedGroupDto> for GroupDto {
    fn from(value: TaggedGroupDto) -> Self {
        Self {
            name: value.group_name,
            id: value.group_id,
            domain: value.group_domain,
        }
    }
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
    pub url: String,
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
    pub url: String,
    pub duration: i32, // milliseconds
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserInfoDto {
    pub username: String,
    pub is_admin: bool,
    pub memberships: Vec<TaggedGroupDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LangDto {
    En,
    #[default]
    Sv,
}

impl Display for LangDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LangDto::En => "en",
            LangDto::Sv => "sv",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct TaggedGroupDto {
    pub group_name: String,
    pub group_id: String,
    pub group_domain: String,
    pub tag_content: Option<String>,
}

impl TaggedGroupDto {
    /// Return self as group identifier syntax: `id@domain`
    pub fn as_group(&self) -> String {
        format!("{}@{}", self.group_id, self.group_domain)
    }
}
