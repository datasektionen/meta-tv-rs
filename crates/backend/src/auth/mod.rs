use chrono::{DateTime, Utc};
use oidc::OidcClient;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub mod oidc;

// can't be __Host- because it would not work on http://localhost in Chrome
const LOGIN_FLOW_CONTEXT_COOKIE: &str = "Meta-TV-Login-Flow-Context";
pub const AUTH_COOKIE: &str = "Meta-TV-Auth";

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub username: String,
    pub is_admin: bool,
    pub expiration: DateTime<Utc>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        get_current_session(request.cookies()).or_forward(Status::Unauthorized)
    }
}

pub async fn begin_authentication(
    redirect_url: String,
    oidc_client: &OidcClient,
    jar: &CookieJar<'_>,
) -> Result<String, AppError> {
    let (url, context) = oidc_client.begin_authentication(redirect_url).await?;

    let value = serde_json::to_string(&context).map_err(AppError::StateSerializationError)?;

    let cookie = Cookie::build((LOGIN_FLOW_CONTEXT_COOKIE, value))
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(rocket::time::Duration::minutes(5));

    jar.add_private(cookie);

    Ok(url)
}

pub async fn finish_authentication(
    code: &str,
    state: &str,
    oidc_client: &OidcClient,
    jar: &CookieJar<'_>,
) -> Result<Session, AppError> {
    let cookie = jar
        .get_private(LOGIN_FLOW_CONTEXT_COOKIE)
        .ok_or(AppError::AuthenticationFlowExpired)?;

    let context = serde_json::from_str(cookie.value_trimmed())
        .map_err(AppError::StateDeserializationError)?;

    let session = oidc_client
        .finish_authentication(context, code, state)
        .await?;

    debug!("User {} logged in successfully", session.username);

    let value = serde_json::to_string(&session).map_err(AppError::StateSerializationError)?;

    // easier to set max age than expires because chrono =/= time
    let delta = session.expiration - Utc::now();

    let cookie = Cookie::build((AUTH_COOKIE, value))
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(rocket::time::Duration::seconds(delta.num_seconds()));
    // ^ we ignore sub-sec nanoseconds for simplicity

    jar.add_private(cookie);
    jar.remove_private(LOGIN_FLOW_CONTEXT_COOKIE);

    Ok(session)
}

fn get_current_session(jar: &CookieJar<'_>) -> Option<Session> {
    let cookie = jar.get_private(AUTH_COOKIE)?;
    let session = serde_json::from_str::<Session>(cookie.value_trimmed()).ok();

    session.filter(|session| session.expiration >= Utc::now())
}

pub fn logout(jar: &CookieJar<'_>) {
    jar.remove_private(AUTH_COOKIE);
}
