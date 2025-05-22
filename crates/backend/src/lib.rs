use auth::oidc::OidcInitializer;
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

mod auth;
mod error;
mod files;
mod guards;
mod pool;
mod routes;
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
        .attach(OidcInitializer)
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
                routes::slide_group::archive_slide_group,
                routes::slide_group::create_slide_group,
                routes::slide_group::get_slide_group,
                routes::slide_group::list_slide_groups,
                routes::slide_group::publish_slide_group,
                routes::slide_group::update_slide_group,
            ],
        )
        .register("/api", catchers![routes::auth::not_logged_in])
        .mount(
            "/auth",
            routes![
                routes::auth::login,
                routes::auth::login_authenticated,
                routes::auth::logout,
                routes::auth::oidc_callback,
                routes::auth::user_info,
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
