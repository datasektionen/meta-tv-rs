use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
use leptos::prelude::*;

use crate::{api, components::error::ErrorList, context::SlideGroupOptionsContext};

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
        if response().is_some() {
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
                    hidden: slide_group.read().hidden,
                    start_date: slide_group.read().start_date,
                    end_date: slide_group.read().end_date,
                });
        }>
            <fieldset disabled=is_submitting>
                <button on:click=move |_| set_editing_options.set(false)>Cancel</button>
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

            <h1>{move || slide_group.get().title}</h1>
        </div>
    }
}
