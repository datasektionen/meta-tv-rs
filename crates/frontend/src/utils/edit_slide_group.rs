//! Contains a variant of EditSlideGroupDto, which also stores the slides' content data, instead of
//! just their IDs. This information is required when rendering the UI, but it isn't stored in
//! `EditSlideGroupDto` since the relevant APIs isn't supposed to edit the content data.

use chrono::{DateTime, Utc};
use common::dtos::{ContentDto, EditSlideDto, EditSlideGroupDto, OwnerDto, SlideGroupDto};
use reactive_stores::Store;

#[derive(Clone, Debug, PartialEq, Eq, Default, Store)]
pub struct EditSlideGroup {
    pub id: i32,
    pub title: String,
    pub priority: i32,
    pub hidden: bool,
    pub created_by: OwnerDto,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub archive_date: Option<DateTime<Utc>>,
    pub published: bool,
    #[store(key: (Option<i32>, i32) = |slide| slide.id())]
    pub slides: Vec<EditSlide>,
}

impl From<SlideGroupDto> for EditSlideGroup {
    fn from(value: SlideGroupDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            priority: value.priority,
            hidden: value.hidden,
            created_by: value.created_by,
            start_date: value.start_date,
            end_date: value.end_date,
            archive_date: value.archive_date,
            published: value.published,
            slides: value
                .slides
                .into_iter()
                .map(|slide| EditSlide {
                    existing: Some(ExistingSlide {
                        id: slide.id,
                        archive_date: slide.archive_date,
                    }),
                    position: slide.position,
                    content: slide.content.into_iter().collect(),
                })
                .collect(),
        }
    }
}

impl From<EditSlideGroup> for EditSlideGroupDto {
    fn from(value: EditSlideGroup) -> Self {
        Self {
            id: value.id,
            title: value.title,
            priority: value.priority,
            hidden: value.hidden,
            created_by: value.created_by,
            start_date: value.start_date,
            end_date: value.end_date,
            archive_date: value.archive_date,
            published: value.published,
            slides: value
                .slides
                .into_iter()
                .map(|slide| match slide.existing {
                    Some(ExistingSlide { id, archive_date }) => EditSlideDto::Existing {
                        id,
                        position: slide.position,
                        archive_date,
                        content: slide
                            .content
                            .into_iter()
                            .map(|content| content.id)
                            .collect(),
                    },
                    None => EditSlideDto::New {
                        position: slide.position,
                        content: slide
                            .content
                            .into_iter()
                            .map(|content| content.id)
                            .collect(),
                    },
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Store)]
pub struct EditSlide {
    // Is none if the slide hasn't been created server side yet.
    pub existing: Option<ExistingSlide>,
    pub position: i32,
    // List of content entity IDs.
    pub content: Vec<ContentDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Store)]
pub struct ExistingSlide {
    pub id: i32,
    pub archive_date: Option<DateTime<Utc>>,
}

impl EditSlide {
    /// Returns value which uniquely identifies this slide in the current UI state.
    pub fn id(&self) -> (Option<i32>, i32) {
        (
            self.existing.as_ref().map(|existing| existing.id),
            self.position,
        )
    }
}

impl Default for EditSlide {
    fn default() -> Self {
        Self {
            existing: None,
            position: 0,
            content: Vec::new(),
        }
    }
}
