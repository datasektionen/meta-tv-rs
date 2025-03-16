use common::dtos::{CreateScreenDto, CreateSlideDto, CreateSlideGroupDto};
use rocket::local::blocking::Client;
use sea_orm::prelude::DateTimeUtc;

#[macro_export]
macro_rules! assert_created {
    ($response: expr, $route_prefix: expr, $id: expr) => {
        assert_eq!($response.status(), rocket::http::Status::Created);
        assert_eq!(
            $response.headers().get_one("Location"),
            Some(format!("{}/{}", $route_prefix, $id).as_ref())
        );
        assert_eq!(
            $response.into_json(),
            Some(common::dtos::CreatedDto { id: $id })
        );
    };
}

pub fn util_create_slide_group(client: &Client) {
    let response = client
        .post("/api/slide-group")
        .json(&CreateSlideGroupDto {
            title: "Lorem Ipsum".to_string(),
            priority: 0,
            hidden: false,
            start_date: DateTimeUtc::from_timestamp_nanos(1739471974000000),
            end_date: None,
        })
        .dispatch();
    assert_created!(response, "/api/slide-group", 1);
}

pub fn util_create_slide(client: &Client, id: i32, position: i32) {
    let response = client
        .post("/api/slide")
        .json(&CreateSlideDto {
            position,
            slide_group: 1,
        })
        .dispatch();
    assert_created!(response, "/api/slide", id);
}

pub fn util_create_screens(client: &Client) {
    macro_rules! create_screen {
        ($name: expr, $position: expr, $id: expr) => {
            let response = client
                .post("/api/screen")
                .json(&CreateScreenDto {
                    name: $name.to_string(),
                    position: $position,
                })
                .dispatch();
            assert_created!(response, "/api/screen", $id);
        };
    }

    create_screen!("Left", 0, 1);
    create_screen!("Center", 1, 2);
    create_screen!("Right", 2, 3);
}
