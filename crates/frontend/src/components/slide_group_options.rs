use chrono::Utc;
use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
use leptos::prelude::*;

use crate::{
    api,
    components::{error::ErrorList, slide::SlideList},
    context::SlideGroupOptionsContext,
    utils::{
        bool::fmt_bool,
        datetime::{datetime_to_input, fmt_datetime, fmt_datetime_opt, input_to_datetime},
    },
};

/// Show options of slide group, keeping track if they have been saved or not.
/// Should force data to be refreshed when edited.
#[component]
pub fn SlideGroupOptions(
    slide_group: Signal<SlideGroupDto>,
    is_editing_options: ReadSignal<bool>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        {move || {
            if is_editing_options.get() {
                view! {
                    <SlideGroupEditOptions
                        slide_group=slide_group
                        set_editing_options=set_editing_options
                    />
                }
                    .into_any()
            } else {
                view! {
                    <SlideGroupViewOptions
                        slide_group=slide_group
                        set_editing_options=set_editing_options
                    />
                }
                    .into_any()
            }
        }}
    }
}

#[component]
fn SlideGroupEditOptions(
    slide_group: Signal<SlideGroupDto>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    let title = RwSignal::new(slide_group.get().title);
    let priority = RwSignal::new(slide_group.get().priority);
    let hidden = RwSignal::new(slide_group.get().hidden);
    let start_date = RwSignal::new(slide_group.get().start_date);
    let end_date = RwSignal::new(slide_group.get().end_date);

    let submit_action = Action::new_local(move |data: &CreateSlideGroupDto| {
        let id = slide_group.get().id;
        let data = data.clone();
        async move { api::update_slide_group(id, &data).await }
    });

    let page_context =
        use_context::<SlideGroupOptionsContext>().expect("to have found the context");

    let is_submitting = submit_action.pending();
    let response = move || submit_action.value().get();
    Effect::new(move || {
        // only go back if submitting has succeeded
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.slide_group.refetch();
            set_editing_options.set(false);
        }
    });

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            submit_action
                .dispatch(CreateSlideGroupDto {
                    title: title.read().to_string(),
                    priority: *priority.read(),
                    hidden: *hidden.read(),
                    start_date: *start_date.read(),
                    end_date: *end_date.read(),
                });
        }>
            <fieldset disabled=is_submitting>
                <button type="button" on:click=move |_| set_editing_options.set(false)>
                    Cancel
                </button>
                <button type="submit" class="border disabled:text-gray-500">
                    Save
                </button>

                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorList errors=errors /> }
                }>{response}</ErrorBoundary>
                <input
                    class="border disabled:bg-gray-50 disabled:text-gray-500"
                    type="text"
                    bind:value=title
                />
                <input
                    class="border disabled:bg-gray-50 disabled:text-gray-500"
                    type="checkbox"
                    prop:checked=move || { priority.get() > 0 }
                    on:change:target=move |ev| {
                        priority.set(if ev.target().checked() { 1 } else { 0 })
                    }
                />
                <input
                    class="border disabled:bg-gray-50 disabled:text-gray-500"
                    type="checkbox"
                    bind:checked=hidden
                />
                <p>Dates are in your local timezone</p>
                <input
                    class="border disabled:bg-gray-50 disabled:text-gray-500"
                    type="datetime-local"
                    step=1
                    prop:value=move || { datetime_to_input(&start_date.get()) }
                    on:change:target=move |ev| {
                        if let Some(dt) = input_to_datetime(&ev.target().value()) {
                            start_date.set(dt);
                        }
                    }
                />
                {move || match end_date.get() {
                    Some(end_date_value) => {
                        view! {
                            <p>Dates are in your local timezone</p>
                            <input
                                class="border disabled:bg-gray-50 disabled:text-gray-500"
                                type="datetime-local"
                                step=1
                                prop:value=move || { datetime_to_input(&end_date_value) }
                                on:change:target=move |ev| {
                                    if let Some(dt) = input_to_datetime(&ev.target().value()) {
                                        end_date.set(Some(dt));
                                    }
                                }
                            />
                            <button type="button" on:click=move |_| { end_date.set(None) }>
                                Remove end date
                            </button>
                        }
                            .into_any()
                    }
                    None => {

                        view! {
                            <button
                                type="button"
                                on:click=move |_| { end_date.set(Some(Utc::now())) }
                            >
                                Add end date
                            </button>
                        }
                            .into_any()
                    }
                }}
            </fieldset>
        </form>
    }
}

#[component]
fn SlideGroupViewOptions(
    slide_group: Signal<SlideGroupDto>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div>
            <button on:click=move |_| set_editing_options.set(true)>Edit</button>

            {move || {
                let group = slide_group.get();
                (!group.published)
                    .then(|| {
                        view! { <SlideGroupPublishButton group_id=group.id /> }
                    })
            }}

            {move || {
                let group = slide_group.get();
                view! {
                    <h1>{group.title}</h1>
                    <p>"Priority: " {move || fmt_bool(group.priority > 0)}</p>
                    <p>"Hidden: " {move || fmt_bool(group.hidden)}</p>
                    <p>"Created by: " {move || group.created_by.clone()}</p>
                    <p>"Start date: " {move || fmt_datetime(&group.start_date)}</p>
                    <p>"End date: " {move || fmt_datetime_opt(group.end_date.as_ref())}</p>
                    <p>"Archive date: " {move || fmt_datetime_opt(group.archive_date.as_ref())}</p>
                    <p>"Published: " {move || fmt_bool(group.published)}</p>
                }
            }}

            <SlideList slide_group=slide_group />
        </div>
    }
}

#[component]
fn SlideGroupPublishButton(#[prop()] group_id: i32) -> impl IntoView {
    let publish_action =
        Action::new_local(move |_: &()| async move { api::publish_slide_group(group_id).await });

    let page_context =
        use_context::<SlideGroupOptionsContext>().expect("to have found the context");

    let is_submitting = publish_action.pending();
    let response = move || publish_action.value().get();
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.slide_group.refetch();
        }
    });

    view! {
        {response}
        <button
            disabled=is_submitting
            on:click=move |_| {
                publish_action.dispatch(());
            }
        >
            Publish
        </button>
    }
}
