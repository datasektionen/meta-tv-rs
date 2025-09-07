use std::borrow::Cow;

use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata,
        CoreResponseType, CoreRevocableToken, CoreRevocationErrorResponse,
        CoreTokenIntrospectionResponse, CoreTokenType,
    },
    AdditionalClaims, AsyncHttpClient, AuthenticationFlow, AuthorizationCode, Client, ClientId,
    ClientSecret, CsrfToken, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdTokenFields, IssuerUrl, Nonce, RedirectUrl, Scope, StandardErrorResponse,
    StandardTokenResponse,
};
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

use super::Session;

#[derive(Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PlsClaims {
    #[serde(rename = "pls_meta-tv")]
    pls_meta_tv: Vec<String>,
}
impl AdditionalClaims for PlsClaims {}

#[derive(thiserror::Error, Debug)]
pub enum OidcInitError<E: 'static + std::error::Error> {
    #[error("invalid OIDC issuer URL: {0}")]
    InvalidIssuerUrl(#[from] openidconnect::url::ParseError),
    #[error("OIDC provider metadata discovery failure: {0}")]
    DiscoveryFailure(#[from] openidconnect::DiscoveryError<E>),
}

#[derive(thiserror::Error, Debug)]
pub enum OidcAuthenticationError {
    #[error("invalid OIDC issuer URL: {0}")]
    InvalidRedirectUrl(#[from] openidconnect::url::ParseError),
    #[error("OIDC issuer does not publish a user info endpoint")]
    NoUserInfoUrl,
    #[error("OIDC issuer returned a CSRF token `{0}` that does not match ours")]
    BadCsrfToken(String),
    #[error("failed to exchange code with token from OIDC issuer")]
    CodeTokenExchangeFailure,
    #[error("OIDC issuer did not return an ID token")]
    NoIdToken,
    #[error("OIDC issuer returned an ID token, but it failed verification")]
    BadIdToken(#[from] openidconnect::ClaimsVerificationError),
    #[error("OIDC issuer did not return any `name` claim for the subject")]
    NoNameClaim,
}

// must be persisted between start of login flow and OIDC callback (end of flow)
#[derive(Serialize, Deserialize)]
pub struct OidcAuthenticationContext {
    redirect_url: RedirectUrl,
    csrf_state: CsrfToken,
    nonce: Nonce,
}

type ReqwestError<'c> = <openidconnect::reqwest::Client as AsyncHttpClient<'c>>::Error;

type PlsIdTokenFields = IdTokenFields<
    PlsClaims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

type PlsClient = Client<
    PlsClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<PlsIdTokenFields, CoreTokenType>,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,      // has auth URL
    EndpointNotSet,   // has device auth URL
    EndpointNotSet,   // has introspection URL
    EndpointNotSet,   // has revocation URL
    EndpointMaybeSet, // has token URL
    EndpointMaybeSet, // has user info URL
>;

// wrapper for simplicity and impl'ing
pub struct OidcClient {
    client: PlsClient,
    http_client: openidconnect::reqwest::Client,
    pub redirect_url: Option<String>,
}

impl OidcClient {
    pub async fn new<'c>(config: OidcConfig) -> Result<Self, OidcInitError<ReqwestError<'c>>> {
        let issuer_url = IssuerUrl::new(config.issuer_url)?;

        let http_client = openidconnect::reqwest::ClientBuilder::new()
            .redirect(openidconnect::reqwest::redirect::Policy::none()) // prevent SSRF attacks
            .build()
            .expect("reqwest client"); // there should be no reason for it to fail

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;

        let client_id = ClientId::new(config.client_id);
        let client_secret = ClientSecret::new(config.client_secret);

        let client =
            Client::from_provider_metadata(provider_metadata, client_id, Some(client_secret));

        Ok(Self {
            client,
            http_client,
            redirect_url: config.redirect_url,
        })
    }

    pub(super) async fn begin_authentication(
        &self,
        redirect_url: String,
    ) -> Result<(String, OidcAuthenticationContext), OidcAuthenticationError> {
        let redirect_url = RedirectUrl::new(redirect_url)?;

        let (authorize_url, csrf_state, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".to_owned()))
            .add_scope(Scope::new("pls_meta-tv".to_owned()))
            .set_redirect_uri(Cow::Borrowed(&redirect_url))
            .url();

        let context = OidcAuthenticationContext {
            redirect_url,
            csrf_state,
            nonce,
        };

        Ok((authorize_url.to_string(), context))
    }

    pub(super) async fn finish_authentication(
        &self,
        context: OidcAuthenticationContext,
        code: &str,
        state: &str,
    ) -> Result<Session, AppError> {
        if CsrfToken::new(state.to_owned()) != context.csrf_state {
            return Err(OidcAuthenticationError::BadCsrfToken(state.to_owned()).into());
        }

        // trade code for a token
        let response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|_| OidcAuthenticationError::NoUserInfoUrl)?
            .set_redirect_uri(Cow::Borrowed(&context.redirect_url))
            .request_async(&self.http_client)
            .await
            .inspect_err(|e| error!("OIDC code exchange error: {e:?}"))
            .map_err(|_| OidcAuthenticationError::CodeTokenExchangeFailure)?;

        let id_token_verifier = self.client.id_token_verifier();
        let claims = response
            .extra_fields()
            .id_token()
            .ok_or(OidcAuthenticationError::NoIdToken)?
            .claims(&id_token_verifier, &context.nonce)
            .map_err(OidcAuthenticationError::from)?;

        let perms = &claims.additional_claims().pls_meta_tv;

        // if !perms.iter().any(|p| p == "post") {
        //     return Err(AppError::LoginUnauthorized);
        // }

        let session = Session {
            username: claims.subject().to_string(),
            is_admin: perms.iter().any(|p| p == "admin"),
            expiration: claims.expiration(),
        };

        Ok(session)
    }
}

pub struct OidcInitializer;

#[rocket::async_trait]
impl Fairing for OidcInitializer {
    fn info(&self) -> Info {
        Info {
            name: "OIDC Manager",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let oidc_config: OidcConfig = match rocket.figment().focus("oidc").extract() {
            Ok(dir) => dir,
            Err(e) => {
                error!("oidc configuration incomplete: {}", e);
                return Err(rocket);
            }
        };

        let oidc_client = match OidcClient::new(oidc_config).await {
            Ok(client) => client,
            Err(e) => {
                error!("failed to initialize oidc client: {}", e);
                return Err(rocket);
            }
        };

        Ok(rocket.manage(oidc_client))
    }
}
