//! Defines jobs which are ran at set intervals in the background.

use chrono_tz::Europe::Stockholm;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use entity::slide_group;
use rocket::{tokio, Orbit, Rocket};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use sea_orm_rocket::Database;
use std::{fmt::Display, future::Future, time::Duration};

use crate::pool::Db;

/// Handle result from async function, logging the error if it has failed without panicing.
async fn log_job_error<O, E, R>(result: R)
where
    R: Future<Output = Result<O, E>>,
    E: Display,
{
    if let Err(error) = result.await {
        println!("Job generated error: {}", error);
    }
}

pub async fn unpin_slide_groups(db: DatabaseConnection) -> Result<(), DbErr> {
    println!("Unpinning slides groups");

    slide_group::Entity::update_many()
        .set(slide_group::ActiveModel {
            priority: ActiveValue::Set(0),
            ..Default::default()
        })
        .filter(slide_group::Column::ArchiveDate.is_null())
        .exec(&db)
        .await?;

    Ok(())
}

pub async fn start(rocket: &Rocket<Orbit>) {
    let mut scheduler = AsyncScheduler::with_tz(Stockholm);

    let db = Db::fetch(rocket)
        .expect("Rocket is in orbit phase")
        .conn
        .clone();

    let cloned_db = db.clone();
    scheduler
        .every(1.days())
        .at("05:00")
        .run(move || log_job_error(unpin_slide_groups(cloned_db.clone())));

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
}
