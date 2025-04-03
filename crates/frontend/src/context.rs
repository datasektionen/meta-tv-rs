use common::dtos::SlideGroupDto;
use leptos::prelude::*;

use crate::api::AppError;

#[derive(Clone)]
pub struct SlideGroupOptionsContext {
    pub slide_group: LocalResource<Result<SlideGroupDto, AppError>>,
}
