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
    #[error("database error: {0}")]
    DatabaseError(#[from] DbErr),
}

impl AppError {
    fn status(&self) -> Status {
        match self {
            AppError::DatabaseError(_) => Status::InternalServerError,
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
