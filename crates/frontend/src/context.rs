use common::dtos::ScreenDto;
use leptos::prelude::*;

#[derive(Clone)]
pub struct ScreenContext {
    pub screens: Memo<Vec<ScreenDto>>,
}
