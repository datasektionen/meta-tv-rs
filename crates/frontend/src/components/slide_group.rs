use chrono::Utc;
use common::dtos::{EditSlideGroupDto, OwnerDto, UserInfoDto};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use reactive_stores::Store;

use crate::{
    api::{self, AppError},
    components::{
        alert::Alert, dialog::Dialog, error::ErrorList, owner_select::OwnerSelect,
        slide::SlideList, start_end_date_input::StartEndDateInput,
    },
    utils::{
        bool::fmt_if,
        datetime::{fmt_datetime, fmt_datetime_opt},
        edit_slide_group::{EditSlideGroup, EditSlideGroupStoreFields},
    },
};

/// Displays slide group, with inputs for editing it inline.
#[component]
pub fn SlideGroup(
    #[prop(into)] slide_group: Store<EditSlideGroup>,
    /// Is called if the user has deleted this slide group (the slide group will already have been
    /// removed server side at this point).
    on_delete: impl Fn() + 'static,
) -> impl IntoView {
    let user_info = use_context::<LocalResource<Result<UserInfoDto, AppError>>>()
        .expect("User info has been provided");
    let is_owner = move || {
        user_info
            .get()
            .and_then(|info| info.ok())
            .map(|info| slide_group.get().created_by.is_owner(&info))
            .unwrap_or(false)
    };
    let is_editing = RwSignal::new(false);
    let saved_slide_group = RwSignal::new(slide_group.get_untracked());

    let update_slide_group = |slide_group: EditSlideGroupDto| {
        async move {
            api::update_slide_group(slide_group.id, &slide_group).await?;
            // Refetch slide group with (potentially) new slide IDs.
            api::get_slide_group(slide_group.id).await
        }
    };
    let save_action = Action::new_local(move |slide_group: &EditSlideGroupDto| {
        update_slide_group(slide_group.clone())
    });
    // Saves slide group and toggles its published state.
    let publish_toggle_action = Action::new_local(move |slide_group: &EditSlideGroupDto| {
        let mut slide_group = slide_group.clone();
        slide_group.published = !slide_group.published;

        update_slide_group(slide_group)
    });
    Effect::new(move || {
        if let Some(Ok(new_slide_group)) = save_action.value().get() {
            slide_group.set(new_slide_group.into());
            is_editing.set(false);
        }
    });
    Effect::new(move || {
        if let Some(Ok(new_slide_group)) = publish_toggle_action.value().get() {
            slide_group.set(new_slide_group.into());
            is_editing.set(false);
        }
    });

    // If all inputs should be disabled.
    let disabled = Signal::derive(move || {
        save_action.pending().get() || publish_toggle_action.pending().get()
    });

    let delete_dialog_open = RwSignal::new(false);

    view! {
        <div>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }.into_any()
            }>
                {move || save_action.value().get().map(|_| ())}
                {move || publish_toggle_action.value().get().map(|_| ())}
            </ErrorBoundary>
            <DeleteDialog
                slide_group_id=slide_group.get_untracked().id
                open=delete_dialog_open
                on_delete
            />
            <div class="flex flex-wrap items-center justify-between gap-2 mb-6">
                <h2 class="card-title text-4xl items-baseline">
                    <span class:italic=move || {
                        !slide_group.get().published
                    }>{move || slide_group.get().title}</span>
                    <Show when=move || !slide_group.get().published>
                        <span class="text-2xl text-gray-600 font-normal">"(draft)"</span>
                    </Show>
                </h2>
                <div class="grid grid-cols-1 items-center justify-items-end grow">
                    <div
                        class="pop-in flex-wrap gap-2 col-span-full row-span-full"
                        class:pop-in-open=is_editing
                    >
                        <button
                            class="pop-in-item btn btn-primary m-right-2"
                            class:btn-soft=move || slide_group.get().published
                            on:click=move |_| {
                                publish_toggle_action.dispatch(slide_group.get_untracked().into());
                            }
                            disabled=disabled
                        >
                            {move || {
                                if slide_group.get().published { "Unpublish" } else { "Publish" }
                            }}
                        </button>
                        <button
                            class="pop-in-item btn btn-primary"
                            on:click=move |_| {
                                save_action.dispatch(slide_group.get_untracked().into());
                            }
                            disabled=disabled
                        >
                            "Save"
                        </button>
                        <button
                            class="pop-in-item btn btn-soft"
                            on:click=move |_| {
                                slide_group.set(saved_slide_group.get_untracked());
                                is_editing.set(false);
                            }
                            disabled=disabled
                        >
                            "Cancel"
                        </button>
                    </div>
                    <div
                        class="tooltip-error tooltip-left pop-in col-span-full row-span-full"
                        class:tooltip=move || !is_owner()
                        class:pop-in-open=move || !is_editing.get()
                        data-tip=move || match &slide_group.get().created_by {
                            OwnerDto::User(username) => {
                                format!("Only {} can edit this slide group", username)
                            }
                            OwnerDto::Group(group) => {
                                format!("Only members of {} can edit this slide group", group.name)
                            }
                        }
                    >
                        <button
                            class="btn btn-primary btn-ghost btn-circle pop-in-item"
                            class:btn-disabled=move || !is_owner()
                            disabled=move || { disabled.get() || !is_owner() }
                            aria-label="Edit"
                            on:click=move |_| {
                                saved_slide_group.set(slide_group.get_untracked());
                                is_editing.set(true);
                            }
                        >
                            <Icon icon=i::MdiPencil width="1.8em" height="1.8em" />
                        </button>
                    </div>
                </div>
            </div>
            <Show when=move || slide_group.get().archive_date.is_some()>
                <Alert icon=i::MdiDeleteAlert class="alert-error">
                    "These slides have been deleted on "
                    {move || fmt_datetime_opt(slide_group.get().archive_date.as_ref(), "None")}
                    " and can't be edited further"
                </Alert>
            </Show>
            <div class="grid gap-4 grid-cols-1 sm:grid-cols-2 md:grid-cols-3 items-center">
                <Show
                    when=move || is_editing.get()
                    fallback=move || view! { <SlideGroupPropertiesDisplay slide_group /> }
                >
                    <SlideGroupPropertiesEditor slide_group disabled />
                </Show>
            </div>
            <SlideList slide_group=slide_group editable=is_editing />
            <div
                class="flex justify-end pop-in pop-in-collapse-layout"
                class:pop-in-open=is_editing
            >
                <button
                    class="btn btn-error btn-soft pop-in-item"
                    disabled=disabled
                    on:click=move |_| delete_dialog_open.set(true)
                >
                    "Delete Group"
                </button>
            </div>
        </div>
    }
    .into_any()
}

#[component]
fn SlideGroupPropertiesDisplay(#[prop(into)] slide_group: Store<EditSlideGroup>) -> impl IntoView {
    view! {
        {move || match slide_group.get().created_by.clone() {
            common::dtos::OwnerDto::User(username) => {
                view! { <PropertyDisplay icon=i::MdiAccount>{username}</PropertyDisplay> }
            }
            common::dtos::OwnerDto::Group(group) => {
                view! { <PropertyDisplay icon=i::MdiAccountGroup>{group.name}</PropertyDisplay> }
            }
        }}
        <PropertyDisplay icon=Signal::derive(move || fmt_if(
            slide_group.get().priority > 0,
            i::MdiPin,
            i::MdiPinOff,
        ))>{move || fmt_if(slide_group.get().priority > 0, "Pinned", "Unpinned")}</PropertyDisplay>
        <PropertyDisplay icon=Signal::derive(move || fmt_if(
            slide_group.get().hidden,
            i::MdiEyeOff,
            i::MdiEye,
        ))>
            {move || fmt_if(slide_group.get().hidden, "Hidden from others", "Shown to everyone")}
        </PropertyDisplay>
        <PropertyDisplay icon=i::MdiClock class="col-span-full">
            <div class="flex gap-2 items-center">
                {move || fmt_datetime(&slide_group.get().start_date)} <span class="shrink-0">
                    <Icon icon=i::MdiArrowRightBold />
                </span> {move || fmt_datetime_opt(slide_group.get().end_date.as_ref(), "Forever")}
            </div>
        </PropertyDisplay>
    }
    .into_any()
}

#[component]
fn SlideGroupPropertiesEditor(
    slide_group: Store<EditSlideGroup>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let owner = slide_group.created_by();
    let priority = slide_group.priority();
    let hidden = slide_group.hidden();

    let start_date = RwSignal::new(slide_group.get_untracked().start_date);
    let end_date = RwSignal::new(
        slide_group
            .get_untracked()
            .end_date
            .map(|date| (true, date))
            .unwrap_or_else(|| (false, Utc::now())),
    );

    Effect::new(move || {
        // This is not a good way to achieve reactivity, but I'm not sure how to do it better.
        slide_group.start_date().set(start_date.get());
        slide_group.end_date().set(match end_date.get() {
            (true, date) => Some(date),
            (false, _) => None,
        });
    });

    view! {
        <label class="select">
            <span class="label">"Owner"</span>
            <OwnerSelect owner attr:disabled=disabled />
        </label>
        <label class="label">
            <input
                type="checkbox"
                class="checkbox"
                prop:checked=move || { priority.get() > 0 }
                on:input:target=move |ev| {
                    priority.set(if ev.target().checked() { 1 } else { 0 });
                }
                disabled=disabled
            />
            "Pinned"
            <span
                class="tooltip"
                data-tip="Make this slide group the only one shown on the TV.\nPins are cleared every night at 03:00 (Swedish time)."
            >
                <Icon icon=i::MdiInformationOutline width="1.1em" height="1.1em" />
            </span>
        </label>
        <label class="label">
            <input
                type="checkbox"
                class="checkbox"
                prop:checked=move || hidden.get()
                on:input:target=move |ev| {
                    hidden.set(ev.target().checked());
                }
                disabled=disabled
            />
            "Hidden"
            <span class="tooltip" data-tip="Do not show this slide group on the TV.">
                <Icon icon=i::MdiInformationOutline width="1.1em" height="1.1em" />
            </span>
        </label>

        <StartEndDateInput
            start_date
            end_date
            disable_end_date_removal_reason=None
            class="col-span-full"
            disabled=disabled
        />
    }
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

#[component]
pub fn DeleteDialog(
    #[prop()] slide_group_id: i32,
    open: RwSignal<bool>,
    on_delete: impl Fn() + 'static,
) -> impl IntoView {
    let submitting = RwSignal::new(false);

    let delete_action =
        Action::new_local(
            move |_: &()| async move { api::archive_slide_group(slide_group_id).await },
        );

    let response = move || delete_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            open.set(false);
            submitting.set(false);
            // Make sure that caller doesn't subscribe this effect to any signals.
            untrack(|| on_delete());
        }
    });

    view! {
        <Dialog open=open>
            <div class="card space-y-6 p-4">
                <div class="w-2xs">
                    <p>"Are you sure you want to delete this slide group?"</p>
                </div>
                <div class="mt-6 flex gap-3">
                    <button
                        class="btn btn-error"
                        on:click=move |_| {
                            submitting.set(true);
                            delete_action.dispatch(());
                        }
                        disabled=submitting
                    >
                        "Delete"
                    </button>
                    <button
                        class="btn"
                        type="button"
                        on:click=move |_| open.set(false)
                        disabled=submitting
                    >
                        "Cancel"
                    </button>
                </div>
            </div>
        </Dialog>
    }
    .into_any()
}
