use common::dtos::SessionDto;
use rocket::{
    http::{uri::Host, CookieJar},
    response::Redirect,
    serde::json::Json,
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
        oidc_client
            .redirect_url
            .clone()
            .unwrap_or(format!("{scheme}://{host}/auth/oidc-callback")),
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

#[rocket::get("/user")]
pub async fn user_info(session: Session) -> Json<SessionDto> {
    Json(SessionDto {
        username: session.username,
        is_admin: session.is_admin,
    })
}

#[cfg(test)]
mod tests {
    use common::dtos::SessionDto;
    use rocket::http::Status;

    use crate::test_utils::TestClient;

    #[test]
    fn get_user_info() {
        let mut client = TestClient::new();
        client.login_as("johndoe", false);

        let response = client.get("/auth/user").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(SessionDto {
                username: "johndoe".to_string(),
                is_admin: false
            })
        );
    }

    #[test]
    fn get_user_info_admin() {
        let mut client = TestClient::new();
        client.login_as("janedoe", true);

        let response = client.get("/auth/user").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json(),
            Some(SessionDto {
                username: "janedoe".to_string(),
                is_admin: true
            })
        );
    }
}
