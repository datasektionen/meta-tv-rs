use common::dtos::SlideGroupDto;
use leptos::prelude::*;

/// Show options of slide group, keeping track if they have been saved or not.
/// Should force data to be refreshed when edited.
#[component]
pub fn SlideGroupOptions(#[prop()] slide_group: SlideGroupDto) -> impl IntoView {
    view! { <h1>{slide_group.title}</h1> }
}
