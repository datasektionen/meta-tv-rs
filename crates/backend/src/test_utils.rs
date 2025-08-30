use core::fmt;

use chrono::{Days, Utc};
use common::dtos::{CreateScreenDto, CreateSlideDto, CreateSlideGroupDto};
use rocket::{
    http::{uri::Origin, Cookie},
    local::blocking::{Client, LocalRequest},
};
use sea_orm::prelude::DateTimeUtc;

use crate::auth::{Session, AUTH_COOKIE};

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

#[macro_export]
macro_rules! assert_app_error {
    ($response: expr, $app_error: expr) => {
        assert_eq!($response.status(), $app_error.status());
        assert_eq!(
            $response.into_json(),
            Some(common::dtos::AppErrorDto::from($app_error))
        );
    };
}

pub struct TestClient {
    client: Client,
    cookie: Option<Cookie<'static>>,
}

macro_rules! req_method {
    ($method: ident) => {
        pub fn $method<'c, 'u: 'c, U>(&'c self, uri: U) -> LocalRequest<'c>
        where
            U: TryInto<Origin<'u>> + fmt::Display,
        {
            self.add_cookie(self.client.$method(uri))
        }
    };
}

impl TestClient {
    pub fn new() -> Self {
        Self {
            client: Client::tracked(crate::rocket()).expect("failed to init rocket client"),
            cookie: None,
        }
    }
    pub fn login_as(&mut self, username: &str, is_admin: bool) {
        self.cookie = Some(get_user_cookie(username, is_admin));
    }
    pub fn logout(&mut self) {
        self.cookie = None;
    }

    fn add_cookie<'c>(&self, request: LocalRequest<'c>) -> LocalRequest<'c> {
        if let Some(cookie) = &self.cookie {
            request.private_cookie(cookie.clone())
        } else {
            request
        }
    }

    req_method!(get);
    req_method!(post);
    req_method!(put);
    req_method!(delete);
}

pub fn util_create_slide_group(client: &TestClient) {
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

pub fn util_create_slide(client: &TestClient, id: i32, position: i32) {
    let response = client
        .post("/api/slide")
        .json(&CreateSlideDto {
            position,
            slide_group: 1,
        })
        .dispatch();
    assert_created!(response, "/api/slide", id);
}

fn get_user_cookie(username: &str, is_admin: bool) -> Cookie<'static> {
    let session = Session {
        username: username.to_string(),
        is_admin,
        expiration: Utc::now()
            .checked_add_days(Days::new(30))
            .expect("chrono add days failed"),
    };
    let value = serde_json::to_string(&session).expect("failed to serialize session");

    (AUTH_COOKIE, value).into()
}
