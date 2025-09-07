use common::dtos::{
    AppErrorDto, CreateContentDto, CreateSlideDto, CreateSlideGroupDto, CreatedDto, ScreenDto,
    SessionDto, SlideGroupDto,
};
use gloo_net::http::{Request, Response};
use leptos::{logging, server_fn::serde::de::DeserializeOwned};
use thiserror::Error;
use web_sys::wasm_bindgen::JsValue;
use web_sys::{File, FormData};

#[derive(Error, Clone, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Connection(String),
    #[error("Error ({1}): {0}")]
    Api(AppErrorDto, u16),
    #[error("Internal error: {0}")]
    Js(String),
    #[error("Internal error: {0}")]
    Serde(String),
}

impl From<gloo_net::Error> for AppError {
    fn from(err: gloo_net::Error) -> Self {
        logging::log!("Gloo error: {:?}", err);
        AppError::Connection(format!("{}", err))
    }
}

impl From<JsValue> for AppError {
    fn from(err: JsValue) -> Self {
        logging::log!("JS error: {:?}", err);
        AppError::Js(format!("{:?}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        logging::log!("Serde error: {:?}", err);
        AppError::Serde(format!("{:?}", err))
    }
}

#[inline]
async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    let status = response.status();
    if status >= 400 {
        Err(AppError::Api(response.json::<AppErrorDto>().await?, status))?;
    }
    Ok(response.json().await?)
}

#[inline]
async fn handle_blank_response(response: Response) -> Result<(), AppError> {
    let status = response.status();
    if status >= 400 {
        Err(AppError::Api(response.json::<AppErrorDto>().await?, status))?;
    }
    Ok(())
}

pub async fn user_info() -> Result<SessionDto, AppError> {
    handle_response(Request::get("/auth/user").send().await?).await
}

pub async fn list_screens() -> Result<Vec<ScreenDto>, AppError> {
    handle_response(Request::get("/api/screen").send().await?).await
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

pub async fn update_slide_group(
    id: i32,
    slide_group: &CreateSlideGroupDto,
) -> Result<(), AppError> {
    handle_blank_response(
        Request::put(&format!("/api/slide-group/{id}"))
            .json(slide_group)?
            .send()
            .await?,
    )
    .await
}

pub async fn publish_slide_group(id: i32) -> Result<(), AppError> {
    handle_blank_response(
        Request::put(&format!("/api/slide-group/{id}/publish"))
            .send()
            .await?,
    )
    .await
}

pub async fn create_slide(slide: &CreateSlideDto) -> Result<CreatedDto, AppError> {
    handle_response(Request::post("/api/slide").json(slide)?.send().await?).await
}

pub async fn upload_content(data: &CreateContentDto, file: &File) -> Result<CreatedDto, AppError> {
    let form_data = FormData::new()?;
    form_data.set_with_str("data", &serde_json::to_string(data)?)?;
    form_data.set_with_blob("file", file)?;
    handle_response(
        Request::post("/api/content")
            .body(form_data)?
            .send()
            .await?,
    )
    .await
}

pub async fn archive_slide_group(id: i32) -> Result<(), AppError> {
    handle_blank_response(
        Request::delete(&format!("/api/slide-group/{id}"))
            .send()
            .await?,
    )
    .await
}

pub async fn delete_slide_row(id: i32) -> Result<(), AppError> {
    handle_blank_response(Request::delete(&format!("/api/slide/{id}")).send().await?).await
}

pub fn get_screen_feed_url(screen_id: i32) -> String {
    format!("/api/feed/{screen_id}")
}
