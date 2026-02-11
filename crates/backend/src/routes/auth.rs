use common::dtos::UserInfoDto;
use rocket::{
    http::{uri::Host, CookieJar},
    response::Redirect,
    serde::json::Json,
    State,
};

use crate::{
    auth::{self, hive::HiveClient, oidc::OidcClient, Session},
    error::AppError,
    guards::scheme::RequestScheme,
    routes::Lang,
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

#[allow(dead_code)]
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

/// Get the username and groups of the logged in user, as well as if they're an admin.
#[rocket::get("/user?<lang>")]
pub async fn user_info(
    lang: Option<Lang>,
    session: Session,
    hive_client: &State<HiveClient>,
) -> Result<Json<UserInfoDto>, AppError> {
    let memberships = if session.is_admin {
        hive_client
            .tagged_groups(lang.unwrap_or_default().into())
            .await?
    } else {
        hive_client
            .tagged_memberships(&session.username, lang.unwrap_or_default().into())
            .await?
    };

    Ok(Json(UserInfoDto {
        username: session.username,
        is_admin: session.is_admin,
        memberships,
    }))
}

#[catch(401)]
pub async fn not_logged_in() -> AppError {
    AppError::Unauthenticated
}

#[cfg(test)]
mod tests {
    use common::dtos::UserInfoDto;
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
            Some(UserInfoDto {
                username: "johndoe".to_string(),
                is_admin: false,
                memberships: Vec::new(),
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
            Some(UserInfoDto {
                username: "janedoe".to_string(),
                is_admin: true,
                memberships: Vec::new(),
            })
        );
    }
}
