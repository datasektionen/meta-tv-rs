use common::dtos::{CreateSlideDto, SlideDto, SlideGroupDto};
use leptos::prelude::*;

use crate::{
    api,
    components::{content::ContentItem, utils::ForVecMemo},
    context::{ScreenContext, SlideGroupOptionsContext},
};

#[component]
pub fn SlideList(slide_group: Signal<SlideGroupDto>) -> impl IntoView {
    let group_id = move || slide_group.read().id;
    let max_position = move || {
        slide_group
            .get()
            .slides
            .iter()
            .map(|s| s.position)
            .max()
            .unwrap_or(-1)
    };

    view! {
        <ForVecMemo
            vec=Signal::derive(move || slide_group.get().slides)
            key=|slide| slide.id
            fallback=move || {
                view! {
                    <div class="h-60 text-center content-center bg-base-200 rounded-lg my-4">
                        "There are currently no slides"
                    </div>
                }
                    .into_any()
            }
            children=move |slide| { view! { <SlideRow slide=slide /> }.into_any() }
        />
        <AddSlideButton
            group_id=Signal::derive(group_id)
            max_position=Signal::derive(max_position)
        />
    }
    .into_any()
}

#[component]
fn AddSlideButton(#[prop(into)] group_id: Signal<i32>, max_position: Signal<i32>) -> impl IntoView {
    let create_action = Action::new_local(move |position: &i32| {
        let data = CreateSlideDto {
            position: *position,
            slide_group: group_id.get(),
        };
        async move { api::create_slide(&data).await }
    });

    let Some(page_context) = use_context::<SlideGroupOptionsContext>() else {
        // if context is not available, then hide button
        return ().into_any();
    };

    let is_submitting = create_action.pending();
    let response = move || create_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.refresh_group.dispatch(());
        }
    });

    view! {
        {response}
        <button
            class="btn"
            disabled=is_submitting
            on:click=move |_| {
                create_action.dispatch(max_position.get() + 1);
            }
        >
            "Add Slide"
        </button>
    }
    .into_any()
}

#[component]
fn SlideRow(#[prop(into)] slide: Signal<SlideDto>) -> impl IntoView {
    let screens = use_context::<ScreenContext>()
        .expect("expected screen context")
        .screens;

    view! {
        <div class="flex gap-4 my-6">
            <For
                each=move || screens.get()
                key=|screen| screen.id
                children=move |screen| {
                    let content = Memo::new(move |_| {
                        slide
                            .with(|slide| {
                                slide.content.iter().find(|c| c.screen == screen.id).cloned()
                            })
                    });
                    view! {
                        <ContentItem
                            screen=screen
                            slide_id=slide.get_untracked().id
                            content=content
                        />
                    }
                        .into_any()
                }
            />
        </div>
    }
    .into_any()
}
