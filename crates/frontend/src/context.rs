use common::dtos::{ScreenDto, SlideGroupDto};
use leptos::prelude::*;

use crate::api::AppError;

#[derive(Clone)]
pub struct SlideGroupOptionsContext {
    pub slide_group: LocalResource<Result<SlideGroupDto, AppError>>,
}

#[derive(Clone)]
pub struct ScreenContext {
    pub screens: LocalResource<Result<Vec<ScreenDto>, AppError>>,
}
