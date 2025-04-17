use rocket::{
    http::{uri::Host, CookieJar},
    response::Redirect,
    State,
};

use crate::{
    auth::{self, oidc::OidcClient, Session},
    error::AppError,
    guards::scheme::RequestScheme,
};

#[rocket::get("/login", rank = 2)]
pub async fn login(
    oidc_client: &State<OidcClient>,
    scheme: RequestScheme,
    host: &Host<'_>,
    jar: &CookieJar<'_>,
) -> Result<Redirect, AppError> {
    let url = auth::begin_authentication(
        format!("{scheme}://{host}/auth/oidc-callback"),
        oidc_client,
        jar,
    )
    .await?;

    Ok(Redirect::to(url))
}

#[rocket::get("/login")]
pub async fn login_authenticated(_session: Session) -> Redirect {
    Redirect::to("/")
}

#[rocket::get("/oidc-callback?<code>&<state>")]
pub async fn oidc_callback(
    code: &str,
    state: &str,
    oidc_client: &State<OidcClient>,
    jar: &CookieJar<'_>,
) -> Result<Redirect, AppError> {
    auth::finish_authentication(code, state, oidc_client, jar).await?;

    Ok(Redirect::to("/"))
}

#[rocket::get("/logout")]
pub async fn logout(jar: &CookieJar<'_>) -> Redirect {
    auth::logout(jar);

    Redirect::to("/")
}
