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

#[derive(Error, Debug)]
pub enum AppError {
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
}

impl AppError {
    pub fn status(&self) -> Status {
        match self {
            AppError::FileTooBig(_) => Status::PayloadTooLarge,
            AppError::ScreenNotFound => Status::NotFound,
            AppError::SlideGroupNotFound => Status::NotFound,
            AppError::SlideGroupArchived => Status::Forbidden,
            AppError::SlideNotFound => Status::NotFound,
            AppError::SlideArchived => Status::Forbidden,
            AppError::DatabaseError(_) => Status::InternalServerError,
            AppError::IoError(_) => Status::InternalServerError,
            AppError::InternalError(_) => Status::InternalServerError,
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
