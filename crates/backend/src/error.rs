use common::dtos::AppErrorDto;
use rocket::{
    error,
    http::Status,
    response::{self, Responder},
    serde::json::Json,
    Request, Response,
};
use sea_orm::DbErr;
use thiserror::Error;

use crate::auth::oidc::OidcAuthenticationError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("you must login to perform this action")]
    Unauthenticated,
    #[error("uploaded file exceeds maximum allowed size (max is {0} bytes)")]
    FileTooBig(u64),
    #[error("screen not found")]
    ScreenNotFound,
    #[error("slide group not found")]
    SlideGroupNotFound,
    #[error("slide group is archived and can't be edited")]
    SlideGroupArchived,
    #[error("slide not found")]
    SlideNotFound,
    #[error("slide is archived and can't be edited")]
    SlideArchived,
    #[error("database error: {0}")]
    DatabaseError(#[from] DbErr),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("internal error: {0}")]
    InternalError(&'static str),

    #[error("you do not have permission to login")]
    LoginUnauthorized,

    #[error("internal OIDC authentication error: {0}")] // not for login failures! just 500
    OidcAuthenticationError(#[from] OidcAuthenticationError),
    #[error("failed to serialize internal state for storage: {0}")]
    StateSerializationError(#[source] serde_json::Error),
    #[error("failed to deserialize internal state from secure storage: {0}")]
    StateDeserializationError(#[source] serde_json::Error), // not from client-controlled
    #[error("failed to complete internal request: {0}")]
    InternalRequestFailure(#[from] reqwest::Error),
    #[error("authentication flow expired and can no longer be completed")]
    AuthenticationFlowExpired,
}

impl AppError {
    pub fn status(&self) -> Status {
        match self {
            AppError::Unauthenticated => Status::Unauthorized,
            AppError::FileTooBig(_) => Status::PayloadTooLarge,
            AppError::ScreenNotFound => Status::NotFound,
            AppError::SlideGroupNotFound => Status::NotFound,
            AppError::SlideGroupArchived => Status::Forbidden,
            AppError::SlideNotFound => Status::NotFound,
            AppError::SlideArchived => Status::Forbidden,
            AppError::DatabaseError(_) => Status::InternalServerError,
            AppError::IoError(_) => Status::InternalServerError,
            AppError::InternalError(_) => Status::InternalServerError,
            AppError::LoginUnauthorized => Status::Forbidden,
            AppError::OidcAuthenticationError(_) => Status::InternalServerError,
            AppError::StateSerializationError(_) => Status::InternalServerError,
            AppError::StateDeserializationError(_) => Status::InternalServerError,
            AppError::InternalRequestFailure(_) => Status::InternalServerError,
            AppError::AuthenticationFlowExpired => Status::Gone,
        }
    }
}

impl From<AppError> for AppErrorDto {
    fn from(err: AppError) -> Self {
        Self {
            msg: err.to_string(),
        }
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status();
        if status.code >= 500 {
            // debug prints enum variant name, display shows thiserror message
            error!("While handling [{req}], encountered {self:?}: {self}");
        }

        let base = Json(AppErrorDto::from(self)).respond_to(req)?;

        Ok(Response::build_from(base).status(status).finalize())
    }
}
