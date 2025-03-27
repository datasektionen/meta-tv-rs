use common::dtos::{AppErrorDto, CreateSlideGroupDto, CreatedDto, SlideGroupDto};
use gloo_net::http::{Request, Response};
use leptos::{logging, server_fn::serde::de::DeserializeOwned};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    ConnectionError(String),
    #[error("Error: {0}")]
    ApiError(#[from] AppErrorDto),
}

impl From<gloo_net::Error> for AppError {
    fn from(err: gloo_net::Error) -> Self {
        logging::log!("Gloo error: {:?}", err);
        AppError::ConnectionError(format!("{}", err))
    }
}

#[inline]
async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    if response.status() >= 400 {
        Err(response.json::<AppErrorDto>().await?)?;
    }
    Ok(response.json().await?)
}

pub async fn list_slide_groups() -> Result<Vec<SlideGroupDto>, AppError> {
    handle_response(Request::get("/api/slide-group").send().await?).await
}

pub async fn create_slide_group(slide_group: &CreateSlideGroupDto) -> Result<CreatedDto, AppError> {
    handle_response(
        Request::post("/api/slide-group")
            .json(slide_group)?
            .send()
            .await?,
    )
    .await
}

pub async fn get_slide_group(id: i32) -> Result<SlideGroupDto, AppError> {
    handle_response(
        Request::get(&format!("/api/slide-group/{id}"))
            .send()
            .await?,
    )
    .await
}
