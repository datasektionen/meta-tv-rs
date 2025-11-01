use crate::{api, components::feed::ScreenFeedSlideshow};
use codee::string::JsonSerdeCodec;
use common::dtos::FeedEntryDto;
use leptos::prelude::*;
use leptos_router::{hooks::use_params, params::Params};
use leptos_use::{use_event_source_with_options, ReconnectLimit, UseEventSourceOptions};

#[derive(Params, PartialEq)]
struct ScreenFeedParams {
    id: Option<i32>,
}

/// Page to display the slideshow on a given TV
#[component]
pub fn ScreenFeed() -> impl IntoView {
    let params = use_params::<ScreenFeedParams>();
    let id = params
        .read_untracked()
        .as_ref()
        .ok()
        .and_then(|params| params.id)
        .unwrap_or_default();

    let event_source = use_event_source_with_options::<Vec<FeedEntryDto>, JsonSerdeCodec>(
        &api::get_screen_feed_url(id),
        UseEventSourceOptions::default()
            .reconnect_limit(ReconnectLimit::Infinite)
            .reconnect_interval(10_000), // 10 seconds
    );

    let data = move || event_source.data.get().unwrap_or_default();

    view! {
        <Transition fallback=|| {
            view! { <div>Loading...</div> }
        }>
        <div class="bg-black">
            <ScreenFeedSlideshow feed=Signal::derive(data) />
        </div>
        </Transition>
    }
    .into_any()
}
