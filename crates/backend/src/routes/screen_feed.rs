use std::env;

use common::dtos::{AppErrorDto, FeedEntryDto};
use entity::{sea_orm::entity::prelude::Expr, sea_orm_active_enums::ContentType};
use rocket::{
    response::stream::{Event, EventStream},
    tokio::{
        select,
        time::{self, Duration},
    },
    Shutdown,
};
use sea_orm::{
    sqlx::types::chrono, ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use sea_orm_rocket::Connection;

use crate::{error::AppError, pool::Db};

const FEED_ENTRY_DURATION: i32 = 10_000;

#[get("/feed/<screen>")]
pub async fn get_screen_feed(
    screen: i32,
    conn: Connection<'_, Db>,
    mut shutdown: Shutdown,
) -> EventStream![Event + '_] {
    let feed_entry_duration = env::var("FEED_ENTRY_DURATION")
        .unwrap_or(FEED_ENTRY_DURATION.to_string())
        .parse::<i32>()
        .unwrap_or(FEED_ENTRY_DURATION);

    error!("{}", feed_entry_duration);

    EventStream! {
        let db = conn.into_inner();
        // TODO use queue/notify instead of interval
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            select! {
                _ = interval.tick() => {
                    match compute_feed(screen, db, feed_entry_duration).await {
                        Ok(data) => yield Event::json(&data),
                        Err(err) => {
                            let status = err.status();
                            if status.code >= 500 {
                                // debug prints enum variant name, display shows thiserror message
                                error!("While handling [/feed/{screen}], encountered {err:?}: {err}");
                            }
                            yield Event::json(&AppErrorDto::from(err));
                        }
                    }
                },
                _ = &mut shutdown => {
                    break;
                }
            };
        }
    }
}

#[derive(FromQueryResult)]
struct PartialEntry {
    priority: i32,
    content_type: Option<ContentType>,
    file_path: Option<String>,
}

/// Get the slideshow feed for a screen, taking into account the following criteria:
/// - Only non-archived content from non-archived slide groups and slides are considered
/// - Only published slide groups are considered
/// - Only groups whose start date is before current time and end date (if any) after current time
///   are considered
/// - Of the groups to be considered, only the ones with the maximum priority are included
/// - Feed entries are sorted by group id first, then position, then slide id, to ensure stable ordering
/// - If a slide is to be shown but does not have content on the given screen, send an image with
///   empty url instead (so that the feed is still aligned with other screens)
/// - For now, slide duration is hardcoded as 30 seconds
///
/// Assumptions:
/// - There is at most one non-archived content per slide
async fn compute_feed(screen: i32, db: &DatabaseConnection, feed_entry_duration: i32) -> Result<Vec<FeedEntryDto>, AppError> {
    let now = chrono::Utc::now();

    let entries: Vec<PartialEntry> = entity::slide::Entity::find()
        .select_only()
        // add relevant columns to select
        .column(entity::slide_group::Column::Id)
        .column(entity::slide_group::Column::Priority)
        .column(entity::slide_group::Column::StartDate)
        .column(entity::content::Column::ContentType)
        .column(entity::slide::Column::Id)
        .column(entity::content::Column::FilePath)
        // joins
        .inner_join(entity::slide_group::Entity)
        .join(
            JoinType::LeftJoin,
            entity::content::Relation::Slide
                .def()
                .rev()
                .on_condition(move |_left, right| {
                    // NOTE: these have to be done here on the join, otherwise rows of slides with
                    // no suitable content won't show up
                    Condition::all()
                        // only get current screen
                        .add(Expr::col((right.clone(), entity::content::Column::Screen)).eq(screen))
                        // ensure not archived
                        .add(Expr::col((right, entity::content::Column::ArchiveDate)).is_null())
                }),
        )
        // used for later filtering by only highest priority items
        .order_by_desc(entity::slide_group::Column::Priority)
        // ensure stable order, and that position is respected
        .order_by_asc(entity::slide_group::Column::Id)
        .order_by_asc(entity::slide::Column::Position)
        .order_by_asc(entity::slide::Column::Id)
        .order_by_asc(entity::content::Column::Id)
        // ensure published
        .filter(entity::slide_group::Column::Published.eq(true))
        // ensure not hidden
        .filter(entity::slide_group::Column::Hidden.eq(false))
        // ensure not archived
        // NOTE: archive_date of content is checked in join statement
        .filter(entity::slide_group::Column::ArchiveDate.is_null())
        .filter(entity::slide::Column::ArchiveDate.is_null())
        // check if in time interval
        .filter(entity::slide_group::Column::StartDate.lte(now))
        .filter(
            Condition::any()
                .add(entity::slide_group::Column::EndDate.is_null())
                .add(entity::slide_group::Column::EndDate.gte(now)),
        )
        .into_model()
        .all(db)
        .await?;

    // TODO use `RANK() OVER (ORDER BY priority)` to only get highest priority items,
    // which requires putting the above query in a sub-query and some custom queries because
    // seaquery does not support it: https://github.com/SeaQL/sea-query/issues/601
    // See also: https://www.postgresql.org/docs/current/tutorial-window.html
    let max_priority = entries.first().map(|entry| entry.priority).unwrap_or(0);

    Ok(entries
        .into_iter()
        .filter(|entry| entry.priority == max_priority)
        .map(|entry| FeedEntryDto {
            content_type: entry
                .content_type
                .map(|ct| ct.into())
                .unwrap_or(common::dtos::ContentType::Image),
            file_path: entry.file_path.unwrap_or_default(),
            duration: feed_entry_duration,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use common::dtos::{ContentType as ContentTypeDto, FeedEntryDto};
    use entity::sea_orm_active_enums::ContentType;
    use migration::MigratorTrait;
    use sea_orm::sqlx::types::chrono::Utc;
    use sea_orm::ActiveValue::Set;
    use sea_orm::{ActiveModelTrait, EntityTrait};

    use crate::routes::screen_feed::FEED_ENTRY_DURATION;

    use super::compute_feed;

    #[async_test]
    async fn feed_computation() {
        let db = sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("failed to connect to in memory database");
        migration::Migrator::up(&db, None)
            .await
            .expect("failed to perform migrations");

        // --- SEED DATA ---

        // screens added by default but not in this test due to custom db
        // add screens
        let screens = [
            entity::screen::ActiveModel {
                name: Set("Left".to_string()),
                position: Set(0),
                ..Default::default()
            },
            entity::screen::ActiveModel {
                name: Set("Center".to_string()),
                position: Set(1),
                ..Default::default()
            },
        ];
        entity::screen::Entity::insert_many(screens)
            .exec(&db)
            .await
            .expect("failed to insert screen");

        // add slide groups
        let lorem = "Lorem Ipsum".to_string();
        let now = Utc::now();
        let yesterday = now
            .checked_sub_days(::chrono::Days::new(1))
            .unwrap()
            .naive_utc();
        let tomorrow = now
            .checked_add_days(::chrono::Days::new(1))
            .unwrap()
            .naive_utc();
        let slide_groups = [
            // one with lower priority
            // id = 1
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(0),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                published: Set(true),
                ..Default::default()
            },
            // one archived
            // id = 2
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                archive_date: Set(Some(yesterday)),
                published: Set(true),
                ..Default::default()
            },
            // one "normal" (hidden)
            // id = 3
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(true),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                published: Set(true),
                ..Default::default()
            },
            // one not published
            // id = 4
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                published: Set(false),
                ..Default::default()
            },
            // one not started
            // id = 5
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(tomorrow),
                published: Set(true),
                ..Default::default()
            },
            // one "normal" (non hidden)
            // id = 6
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                published: Set(true),
                ..Default::default()
            },
            // one already ended
            // id = 7
            entity::slide_group::ActiveModel {
                title: Set(lorem.clone()),
                priority: Set(1),
                hidden: Set(false),
                created_by: Set(lorem.clone()),
                start_date: Set(yesterday),
                end_date: Set(Some(yesterday)),
                published: Set(true),
                ..Default::default()
            },
        ];
        entity::slide_group::Entity::insert_many(slide_groups)
            .exec(&db)
            .await
            .expect("failed to insert slide groups");

        // insert dummy slides/content on groups that won't show up, just to ensure they really
        // don't show up
        for i in [1, 2, 4, 5, 7] {
            let slide = entity::slide::ActiveModel {
                position: Set(0),
                group: Set(i),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            entity::content::ActiveModel {
                slide: Set(slide.id),
                screen: Set(1),
                content_type: Set(ContentType::Image),
                file_path: Set(format!("slide_{}_content_0", slide.id)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert content");
        }

        // NOTE: The slides below are added to groups in a weird order to test sorting

        // add archived slide (will not be shown)
        {
            let slide = entity::slide::ActiveModel {
                position: Set(0),
                group: Set(6),
                archive_date: Set(Some(yesterday)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 6);

            entity::content::ActiveModel {
                slide: Set(slide.id),
                screen: Set(1),
                content_type: Set(ContentType::Image),
                file_path: Set(format!("slide_{}_content_0", slide.id)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert content");
        }

        // add slide with archived content (will be shown as empty)
        {
            let slide = entity::slide::ActiveModel {
                position: Set(2),
                group: Set(6),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 7);

            entity::content::ActiveModel {
                slide: Set(slide.id),
                screen: Set(1),
                content_type: Set(ContentType::Image),
                file_path: Set(format!("slide_{}_content_0", slide.id)),
                archive_date: Set(Some(yesterday)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert content");
        }

        // add slide with both archived and non-archived content (only non-archived content will be
        // shown)
        {
            let slide = entity::slide::ActiveModel {
                position: Set(1),
                group: Set(6),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 8);

            let contents = [
                entity::content::ActiveModel {
                    slide: Set(slide.id),
                    screen: Set(1),
                    content_type: Set(ContentType::Image),
                    file_path: Set(format!("slide_{}_content_0", slide.id)),
                    archive_date: Set(Some(yesterday)),
                    ..Default::default()
                },
                entity::content::ActiveModel {
                    slide: Set(slide.id),
                    screen: Set(1),
                    content_type: Set(ContentType::Image),
                    file_path: Set(format!("slide_{}_content_1", slide.id)),
                    ..Default::default()
                },
            ];
            entity::content::Entity::insert_many(contents)
                .exec(&db)
                .await
                .expect("failed to insert content");
        }

        // add slide with content only in a different screen (will be shown as empty on screen 1)
        {
            let slide = entity::slide::ActiveModel {
                position: Set(3),
                group: Set(6),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 9);

            entity::content::ActiveModel {
                slide: Set(slide.id),
                screen: Set(2),
                content_type: Set(ContentType::Image),
                file_path: Set(format!("slide_{}_content_0", slide.id)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert content");
        }

        // add "normal" slide with image content (will be shown)
        {
            let slide = entity::slide::ActiveModel {
                position: Set(4),
                group: Set(6),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 10);

            entity::content::ActiveModel {
                slide: Set(slide.id),
                screen: Set(1),
                content_type: Set(ContentType::Image),
                file_path: Set(format!("slide_{}_content_0", slide.id)),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert content");
        }

        // add "normal" slide with video and html content in both screens (will be shown)
        // NOTE: different group now
        {
            let slide = entity::slide::ActiveModel {
                position: Set(0),
                group: Set(3),
                ..Default::default()
            }
            .insert(&db)
            .await
            .expect("failed to insert slide");

            assert_eq!(slide.id, 11);

            let contents = [
                entity::content::ActiveModel {
                    slide: Set(slide.id),
                    screen: Set(1),
                    content_type: Set(ContentType::Html),
                    file_path: Set(format!("slide_{}_content_0", slide.id)),
                    ..Default::default()
                },
                entity::content::ActiveModel {
                    slide: Set(slide.id),
                    screen: Set(2),
                    content_type: Set(ContentType::Video),
                    file_path: Set(format!("slide_{}_content_1", slide.id)),
                    ..Default::default()
                },
            ];
            entity::content::Entity::insert_many(contents)
                .exec(&db)
                .await
                .expect("failed to insert content");
        }

        let feed = compute_feed(1, &db, FEED_ENTRY_DURATION).await.expect("failed to compute feed");
        assert_eq!(
            feed,
            [
                // FeedEntryDto {
                //     content_type: ContentTypeDto::Html,
                //     file_path: "slide_11_content_0".to_string(),
                //     duration: FEED_ENTRY_DURATION,
                // },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "slide_8_content_1".to_string(),
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "".to_string(), // slide 7
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "".to_string(), // slide 9
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "slide_10_content_0".to_string(),
                    duration: FEED_ENTRY_DURATION,
                },
            ]
        );

        let feed = compute_feed(2, &db, FEED_ENTRY_DURATION).await.expect("failed to compute feed");
        assert_eq!(
            feed,
            [
                // FeedEntryDto {
                //     content_type: ContentTypeDto::Video,
                //     file_path: "slide_11_content_1".to_string(),
                //     duration: FEED_ENTRY_DURATION,
                // },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "".to_string(), // slide 8
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "".to_string(), // slide 7
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "slide_9_content_0".to_string(),
                    duration: FEED_ENTRY_DURATION,
                },
                FeedEntryDto {
                    content_type: ContentTypeDto::Image,
                    file_path: "".to_string(), // slide 10
                    duration: FEED_ENTRY_DURATION,
                },
            ]
        );
    }

    // TODO test the eventstream when proper signaling is implemented
    // Useful reference: https://github.com/rwf2/Rocket/blob/v0.5.1/examples/chat/src/tests.rs#L33
}
