use std::{convert::Infallible, fmt};

use rocket::{
    request::{FromRequest, Outcome},
    Request,
};

const HTTP_DOMAINS: &[&str] = &["localhost", "0.0.0.0", "127.0.0.1"];

pub enum RequestScheme {
    Http,
    Https,
}

impl fmt::Display for RequestScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Https => write!(f, "https"),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestScheme {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let scheme = if let Some(proto) = req.headers().get_one("X-Forwarded-Proto") {
            match proto {
                "https" => Self::Https,
                "http" => Self::Http,
                val => {
                    warn!("Unknown X-Forwarded-Proto value: {val}");

                    Self::Https
                }
            }
        } else if req
            .host()
            .map(|h| h.domain())
            .map(|d| HTTP_DOMAINS.iter().any(|x| x == d))
            .unwrap_or(false)
        {
            Self::Http
        } else {
            warn!("X-Forwarded-Proto is not set for request to non-localhost!");

            Self::Https
        };

        Outcome::Success(scheme)
    }
}
