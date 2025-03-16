use files::FilesInitializer;
use migration::MigratorTrait;
use pool::Db;
use rocket::{
    fairing::{self, AdHoc},
    Build, Rocket,
};
use sea_orm_rocket::Database;

#[macro_use]
extern crate rocket;

mod error;
mod files;
mod pool;
mod routes;
mod session;
#[cfg(test)]
mod test_utils;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}

pub(crate) fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(FilesInitializer)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
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
                routes::slide_group::create_slide_group,
                routes::slide_group::get_slide_group,
                routes::slide_group::list_slide_groups,
                routes::slide_group::publish_slide_group,
                routes::slide_group::update_slide_group,
            ],
        )
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
        println!("Error: {err}");
    }
}
