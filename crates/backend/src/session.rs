use rocket::{
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
};

#[allow(dead_code)]
pub struct User {
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(_request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        // TODO
        Some(User {
            username: "johndoe".to_string(),
        })
        .or_forward(Status::Unauthorized)
        // request
        // .cookies()
        // .get_private("username")
        // .map(|cookie| cookie.value().to_string())
        // .map(|username| User { username })
        // .or_forward(Status::Unauthorized)
    }
}
