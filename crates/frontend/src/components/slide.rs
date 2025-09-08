use common::dtos::{CreateSlideDto, SlideDto, SlideGroupDto};
use leptos::prelude::*;

use crate::{
    api,
    components::{content::ContentItem, dialog::Dialog, utils::ForVecMemo},
    context::{ScreenContext, SlideGroupOptionsContext},
};

#[component]
pub fn SlideList(slide_group: Signal<SlideGroupDto>, editeble: bool) -> impl IntoView {
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
            children=move |slide| { view! { <SlideRow slide=slide slide_group=slide_group editeble=editeble /> }.into_any() }
        />
        {move || {
            view! {
                <Show when=move || !slide_group.get().archive_date.is_some()>
                    <AddSlideButton
                        group_id=Signal::derive(group_id)
                        max_position=Signal::derive(max_position)
                    />
                </Show>
            }.into_any()
        }}
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
fn SlideRow(#[prop(into)] slide: Signal<SlideDto>, slide_group: Signal<SlideGroupDto>, editeble: bool) -> impl IntoView {
    let screens = use_context::<ScreenContext>()
        .expect("expected screen context")
        .screens;

    let is_delete_dialog_open = RwSignal::new(false);

    view! {
        <DeleteDialog
            slide_id=slide.get().id
            open=is_delete_dialog_open
        />
        <div class="my-6">
            <div class="flex gap-4 flex-col md:flex-row justify-center md:justify-start items-center md:items-start">
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
            {move || {
                view! {
                    <Show when=move || !slide_group.get().archive_date.is_some() && editeble>
                        <div class="flex flex-row justify-center md:justify-start">
                            <button class="btn my-3" on:click=move |_| is_delete_dialog_open.set(true)>
                                Delete
                            </button>
                        </div>
                    </Show>
                }.into_any()
            }}
        </div>
    }
    .into_any()
}

#[component]
pub fn DeleteDialog(#[prop()] slide_id: i32, open: RwSignal<bool>) -> impl IntoView {
    let delete_action =
        Action::new_local(move |_: &()| async move { api::delete_slide_row(slide_id).await });

    let Some(page_context) = use_context::<SlideGroupOptionsContext>() else {
        // if context is not available, then hide button
        return ().into_any();
    };

    let response = move || delete_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.refresh_group.dispatch(());
        }
    });

    view! {
        <Dialog open=open>
            <div class="card space-y-6 p-4">
                <div class="w-2xs">
                    <p>Are you sure you want to delete these slides</p>
                </div>
                <div class="mt-6 flex gap-3">
                    <button class="btn" on:click=move |_| {delete_action.dispatch(());}>
                        Delete
                    </button>
                    <button class="btn" type="button" on:click=move |_| open.set(false)>
                        "Cancel"
                    </button>
                </div>
            </div>
        </Dialog>
    }
    .into_any()
}
