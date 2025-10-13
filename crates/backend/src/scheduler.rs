//! Defines jobs which are ran at set intervals in the background.

use chrono_tz::Europe::Stockholm;
use clokwerk::{AsyncScheduler, TimeUnits};
use rocket::{tokio, Orbit, Rocket};
use std::time::Duration;

pub fn start(_rocket: &Rocket<Orbit>) {
    let mut scheduler = AsyncScheduler::with_tz(Stockholm);

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
}
