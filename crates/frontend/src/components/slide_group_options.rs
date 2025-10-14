use chrono::Utc;
use common::dtos::{CreateSlideGroupDto, SlideGroupDto};
use icondata as i;
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_icons::Icon;

use crate::{
    api,
    components::{alert::Alert, dialog::Dialog, error::ErrorList, slide::SlideList},
    context::SlideGroupOptionsContext,
    utils::{
        bool::fmt_if,
        datetime::{datetime_to_input, fmt_datetime, fmt_datetime_opt, input_to_datetime},
    },
};

/// Show options of slide group, keeping track if they have been saved or not.
/// Should force data to be refreshed when edited.
#[component]
pub fn SlideGroupOptions(
    #[prop(into)] slide_group: Signal<SlideGroupDto>,
    is_editing_options: ReadSignal<bool>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <Show
            when=move || is_editing_options.get()
            fallback=move || {
                view! {
                    <SlideGroupViewOptions
                        slide_group=slide_group
                        set_editing_options=set_editing_options
                    />
                }
                    .into_any()
            }
        >
            <SlideGroupEditOptions
                slide_group=slide_group
                set_editing_options=set_editing_options
            />
        </Show>
    }
    .into_any()
}

#[component]
fn SlideGroupEditOptions(
    slide_group: Signal<SlideGroupDto>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    let title = RwSignal::new(slide_group.get_untracked().title);
    let priority = RwSignal::new(slide_group.get_untracked().priority);
    let hidden = RwSignal::new(slide_group.get_untracked().hidden);
    let start_date = RwSignal::new(slide_group.get_untracked().start_date);
    let end_date = RwSignal::new(slide_group.get_untracked().end_date);

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
            page_context.refresh_group.dispatch(());
            set_editing_options.set(false);
        }
    });

    let is_delete_dialog_open = RwSignal::new(false);

    view! {
        <DeleteDialog
            slide_group_id=slide_group.get().id
            open=is_delete_dialog_open
            set_editing_options=set_editing_options
        />
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
            <div class="space-y-12">
                <div class="border-b border-gray-100/10 pb-12">
                    <fieldset disabled=is_submitting>
                        <h2 class="text-base/7 font-semibold">General</h2>
                        <ErrorBoundary fallback=|errors| {
                            view! { <ErrorList errors=errors /> }.into_any()
                        }>{response}</ErrorBoundary>
                        <div class="sm:col-span-4 mt-6">
                          <label for="title" class="block text-sm/6 font-medium">Title</label>
                          <div class="mt-2">
                            <input
                                name="title"
                                class="input block w-full rounded-md px-3 py-1.5 outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6"
                                type="text"
                                bind:value=title
                            />
                          </div>
                        </div>
                    </fieldset>
                </div>
                <div class="border-b border-gray-100/10 pb-12">
                    <fieldset disabled=is_submitting>
                        <h2 class="text-base/7 font-semibold">Visibility</h2>
                        <div class="mt-6 space-y-6">
                            <div class="flex gap-3">
                                <div class="flex h-6 shrink-0 items-center">
                                    <div class="grid size-4 grid-cols-1">
                                        <input
                                            type="checkbox"
                                            name="priority"
                                            id="priority"
                                            aria-describedby="marked-priority"
                                            class="col-start-1 row-start-1 rounded-sm border border-gray-300 checked:border-indigo-600 checked:bg-indigo-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:border-gray-300 disabled:bg-gray-100 disabled:checked:bg-gray-100 forced-colors:appearance-auto" 
                                            prop:checked=move || { priority.get() > 0 }
                                            on:change:target=move |ev| {
                                                priority.set(if ev.target().checked() { 1 } else { 0 })
                                            }
                                        />
                                    </div>
                                </div>
                                <div class="text-sm/6">
                                    <label for="priority" class="label font-medium">Pin</label>
                                    <p class="text-gray-500">
                                        Make this slide group the only one shown on the TV.
                                        <em>Pins are cleared every night at 5 AM (Swedish time).</em>
                                    </p>
                                </div>
                            </div>
                        </div>
                        <div class="mt-3 space-y-6">
                            <div class="flex gap-3">
                                <div class="flex h-6 shrink-0 items-center">
                                    <div class="grid size-4 grid-cols-1">
                                        <input
                                            type="checkbox"
                                            name="hidden"
                                            id="hidden"
                                            aria-describedby="mark-hidden"
                                            class="col-start-1 row-start-1 rounded-sm border border-gray-300 checked:border-indigo-600 checked:bg-indigo-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:border-gray-300 disabled:bg-gray-100 disabled:checked:bg-gray-100 forced-colors:appearance-auto" 
                                            bind:checked=hidden
                                        />
                                    </div>
                                </div>
                                <div class="text-sm/6">
                                    <label for="hidden" class="label font-medium">Hidden</label>
                                    <p class="text-gray-500">Do not show this slide group on the TV.</p>
                                </div>
                            </div>
                        </div>
                    </fieldset>
                </div>
                <div class="border-b border-gray-900/10 pb-12">
                    <fieldset disabled=is_submitting>
                        <h2 class="text-base/7 font-semibold">Active Timespan</h2>
                        <div class="col-span-full mt-6">     
                            <label for="start-date" class="block text-sm/6 font-medium">Start</label>
                            <div class="mt-3 space-y-6">
                                <div class="flex gap-3">
                                    <div class="flex w-full items-center">
                                        <div class="grid w-full grid-cols-1">
                                            <input
                                                class="col-start-1 row-start-1 input border disabled:bg-gray-50 disabled:text-gray-500"
                                                type="datetime-local"
                                                step=1
                                                prop:value=move || { datetime_to_input(&start_date.get()) }
                                                on:change:target=move |ev| {
                                                    if let Some(dt) = input_to_datetime(&ev.target().value()) {
                                                        start_date.set(dt);
                                                    }
                                                }
                                            />
                                            <p class="mt-3 text-sm/6 text-gray-600">Dates are in Swedish time</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        {move || match end_date.get() {
                            Some(end_date_value) => {
                                view! {
                                    <div class="col-span-full mt-6">     
                                        <label for="end-date" class="block text-sm/6 font-medium">End</label>
                                        <div class="mt-3 space-y-6">
                                            <div class="flex gap-3">
                                                <div class="flex w-full items-center">
                                                    <div class="grid w-full grid-cols-1">
                                                        <input
                                                            class="col-start-1 row-start-1 input border disabled:bg-gray-50 disabled:text-gray-500"
                                                            type="datetime-local"
                                                            step=1
                                                            prop:value=move || { datetime_to_input(&end_date_value) }
                                                            on:change:target=move |ev| {
                                                                if let Some(dt) = input_to_datetime(&ev.target().value()) {
                                                                    end_date.set(Some(dt));
                                                                }
                                                            }
                                                        />
                                                        <p class="mt-3 text-sm/6 text-gray-600">Dates are in Swedish time</p>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <button class="btn" type="button" on:click=move |_| { end_date.set(None) }>
                                        Remove end date
                                    </button>
                                }
                                    .into_any()
                            }
                            None => {

                                view! {
                                    <button
                                        class="btn"
                                        type="button"
                                        on:click=move |_| { end_date.set(Some(Utc::now())) }
                                    >
                                        Add end date
                                    </button>
                                }
                                    .into_any()
                            }
                        }}
                        <div class="mt-6 flex gap-6">
                            <button type="submit" class="btn border disabled:text-gray-500">
                                Save
                            </button>
                            <button class="btn" type="button" on:click=move |_| set_editing_options.set(false)>
                                Cancel
                            </button>
                            <button class="btn btn-soft btn-error" type="button" on:click=move |_| is_delete_dialog_open.set(true)>
                                Delete Group
                            </button>
                        </div>
                    </fieldset>
                </div>
            </div>
        </form>
    }
    .into_any()
}

#[component]
fn SlideGroupViewOptions(
    slide_group: Signal<SlideGroupDto>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    let is_delete_dialog_open = RwSignal::new(false);

    view! {
        <div>
            {move || {
                view! {
                    <DeleteDialog
                        slide_group_id=slide_group.get().id
                        open=is_delete_dialog_open
                        set_editing_options=set_editing_options
                    />
                    <Show when=move || !slide_group.read().archive_date.is_some()>
                        <button on:click=move |_| set_editing_options.set(true) class="btn">Edit</button>

                        {move || {
                            view! {
                                <Show when=move || !slide_group.read().published>
                                    <SlideGroupPublishButton group_id=Signal::derive(move || {
                                        slide_group.read().id
                                    }) />
                                </Show>
                            }
                                .into_any()
                        }}

                        <button on:click=move |_| is_delete_dialog_open.set(true) class="btn btn-soft btn-error">Delete Group</button>
                    </Show>
                }.into_any()
            }}

            {move || {
                let group = slide_group.get();
                view! {
                    <h1 class="text-6xl mb-6">{group.title}</h1>
                    <Show when=move || group.archive_date.is_some()>
                        <Alert icon=i::MdiDeleteAlert class="bg-red-300">
                            "These slides have been deleted on "
                            {move || fmt_datetime_opt(group.archive_date.as_ref(), "None")}
                            " and can't be edited further"
                        </Alert>
                    </Show>
                    <div class="grid grid-cols-3 gap-4">
                        <PropertyDisplay icon=i::MdiAccount>
                            {move || group.created_by.clone()}
                        </PropertyDisplay>
                        <PropertyDisplay icon=fmt_if(
                            group.published,
                            i::MdiFileCheck,
                            i::MdiFileEdit,
                        )>{move || fmt_if(group.published, "Published", "Draft")}</PropertyDisplay>
                        <PropertyDisplay icon=fmt_if(
                            group.priority > 0,
                            i::MdiPin,
                            i::MdiPinOff,
                        )>
                            {move || fmt_if(group.priority > 0, "Pinned", "Unpinned")}
                        </PropertyDisplay>
                        <PropertyDisplay icon=fmt_if(
                            group.hidden,
                            i::MdiEyeOff,
                            i::MdiEye,
                        )>
                            {move || fmt_if(
                                group.hidden,
                                "Hidden",
                                "Shown",
                            )}
                        </PropertyDisplay>
                        <PropertyDisplay icon=i::MdiClock class="col-span-2">
                            <div class="flex gap-2 items-center">
                                {move || fmt_datetime(&group.start_date)}
                                <Icon icon=i::MdiArrowRightBold />
                                {move || fmt_datetime_opt(group.end_date.as_ref(), "Forever")}
                            </div>
                        </PropertyDisplay>
                    </div>
                }
                    .into_any()
            }}

            <SlideList slide_group=slide_group editeble=true />
        </div>
    }
    .into_any()
}

#[component]
pub fn DeleteDialog(
    #[prop()] slide_group_id: i32,
    open: RwSignal<bool>,
    set_editing_options: WriteSignal<bool>,
) -> impl IntoView {
    let delete_action =
        Action::new_local(
            move |_: &()| async move { api::archive_slide_group(slide_group_id).await },
        );

    let Some(page_context) = use_context::<SlideGroupOptionsContext>() else {
        // if context is not available, then hide button
        return ().into_any();
    };

    let response = move || delete_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.refresh_group.dispatch(());
            set_editing_options.set(false);
            open.set(false);
        }
    });

    view! {
        <Dialog open=open>
            <div class="card space-y-6 p-4">
                <div class="w-2xs">
                    <p>Are you sure you want to delete this slide group</p>
                </div>
                <div class="mt-6 flex gap-3">
                    <button class="btn btn-error" on:click=move |_| {delete_action.dispatch(());}>
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

#[component]
fn SlideGroupPublishButton(#[prop(into)] group_id: Signal<i32>) -> impl IntoView {
    let publish_action = Action::new_local(move |_: &()| {
        let id = group_id.get();
        async move { api::publish_slide_group(id).await }
    });

    let page_context =
        use_context::<SlideGroupOptionsContext>().expect("to have found the context");

    let is_submitting = publish_action.pending();
    let response = move || publish_action.value().get();
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
                publish_action.dispatch(());
            }
        >
            Publish
        </button>
    }
    .into_any()
}

#[component]
fn PropertyDisplay(
    #[prop(into)] icon: Signal<icondata_core::Icon>,
    #[prop(into, optional)] class: MaybeProp<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <p class=["flex gap-2 items-center text-lg", class.read().unwrap_or_default()].join(" ")>
            <Icon icon=icon width="1.5em" height="1.5em" />
            <span>{children()}</span>
        </p>
    }
    .into_any()
}
