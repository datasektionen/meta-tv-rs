use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use auth::oidc::OidcInitializer;
use files::FilesInitializer;
use migration::MigratorTrait;
use pool::Db;
use rocket::{
    fairing::{self, AdHoc},
    fs::{FileServer, NamedFile},
    Build, Rocket,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use sea_orm_rocket::Database;

use crate::auth::hive::HiveInitializer;

#[macro_use]
extern crate rocket;

mod auth;
mod cached_file;
mod error;
mod files;
mod guards;
mod pool;
mod routes;
mod scheduler;
#[cfg(test)]
mod test_utils;

#[get("/<file..>", rank = 12)]
async fn serve_file(file: PathBuf) -> Option<NamedFile> {
    let file = NamedFile::open(Path::new("/www/static/").join(file))
        .await
        .ok();

    if let Some(file) = file {
        if !file.path().is_dir() {
            return Some(file);
        }
    }

    NamedFile::open(Path::new("/www/static/index.html"))
        .await
        .ok()
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}

async fn setup_screens(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;

    if entity::screen::Entity::find()
        .one(conn)
        .await
        .unwrap()
        .is_some()
    {
        return Ok(rocket);
    }

    entity::screen::ActiveModel {
        name: Set("Left".to_string()),
        position: Set(0),
        ..Default::default()
    }
    .insert(conn)
    .await
    .unwrap();

    entity::screen::ActiveModel {
        name: Set("Center".to_string()),
        position: Set(1),
        ..Default::default()
    }
    .insert(conn)
    .await
    .unwrap();

    entity::screen::ActiveModel {
        name: Set("Right".to_string()),
        position: Set(2),
        ..Default::default()
    }
    .insert(conn)
    .await
    .unwrap();

    Ok(rocket)
}

pub(crate) fn rocket() -> Rocket<Build> {
    std::thread::sleep(Duration::from_secs(2)); // Sleep to prevent race stuff (macbook go zoom)
    rocket::build()
        .attach(FilesInitializer)
        .attach(HiveInitializer)
        .attach(OidcInitializer)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .attach(AdHoc::try_on_ignite("Screens", setup_screens))
        .attach(AdHoc::on_liftoff("Scheduled tasks", |rocket| {
            Box::pin(scheduler::start(rocket))
        }))
        .mount("/", FileServer::from("/www/static/").rank(11))
        .mount("/", routes![serve_file])
        .mount(
            "/api",
            routes![
                routes::content::create_content,
                routes::health::health,
                routes::screen::create_screen,
                routes::screen::list_screens,
                routes::screen_feed::get_screen_feed,
                routes::slide::create_slide,
                routes::slide::bulk_move_slides,
                routes::slide::delete_slide,
                routes::slide_group::archive_slide_group,
                routes::slide_group::create_slide_group,
                routes::slide_group::get_slide_group,
                routes::slide_group::list_slide_groups,
                routes::slide_group::publish_slide_group,
                routes::slide_group::update_slide_group,
                routes::slide_group::update_slide_group_owner,
            ],
        )
        .register("/api", catchers![routes::auth::not_logged_in])
        .mount(
            "/auth",
            routes![
                routes::auth::login,
                // routes::auth::login_authenticated,
                routes::auth::logout,
                routes::auth::oidc_callback,
                routes::auth::user_info,
                routes::auth::user_memberships,
            ],
        )
        .register("/auth", catchers![routes::auth::not_logged_in])
}

#[rocket::main]
async fn start() -> Result<(), rocket::Error> {
    rocket().launch().await?;
    Ok(())
}

pub fn main() {
    let result = start();

    println!("Rocket: deorbit.");

    if let Some(err) = result.err() {
        println!("Error: {err:?}");
    }
}
