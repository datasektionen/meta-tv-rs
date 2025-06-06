use common::dtos::ScreenDto;
use leptos::prelude::*;

#[derive(Clone)]
pub struct SlideGroupOptionsContext {
    pub refresh_group: Action<(), ()>,
}

#[derive(Clone)]
pub struct ScreenContext {
    pub screens: Memo<Vec<ScreenDto>>,
}
