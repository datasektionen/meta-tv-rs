use common::dtos::{OwnerDto, SlideGroupDto, UserInfoDto};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::{
    api::AppError,
    components::{alert::Alert, slide::SlideList},
    utils::{
        bool::fmt_if,
        datetime::{fmt_datetime, fmt_datetime_opt},
    },
};

/// Show overview of slide group, to be used on the home page as a summary of
/// each slide group.
/// Should not include a lot of data.
#[component]
pub fn SlideGroupOverview(#[prop(into)] slide_group: Signal<SlideGroupDto>) -> impl IntoView {
    let user_info = use_context::<LocalResource<Result<UserInfoDto, AppError>>>()
        .expect("User info has been provided");
    let is_owner = move || {
        user_info
            .get()
            .and_then(|info| info.ok())
            .map(|info| slide_group.get().created_by.is_owner(&info))
            .unwrap_or(false)
    };
    view! {
        <div>
            {move || {
                let group = slide_group.get();
                view! {
                    <div class="flex flex-wrap-reverse items-center justify-between gap-2 mb-6">
                        <h1 class="card-title text-4xl">
                            <Show when=move || is_owner()>
                                <a href=format!("/slides/{}", group.id)>{slide_group.get().title}</a>
                            </Show>
                            <Show when=move || !is_owner()>
                                {slide_group.get().title}
                            </Show>
                        </h1>
                        <div class="text-right grow">
                            <div
                                class="tooltip-error"
                                class:tooltip=!is_owner()
                                data-tip=match &slide_group.get().created_by {
                                    OwnerDto::User(username) => format!("Only {} can edit this slide group", username),
                                    OwnerDto::Group(group) => format!("Only members of {} can edit this slide group", group.name),
                                }
                            >
                                <a
                                    href=format!("/slides/{}", group.id)
                                    class="btn btn-primary"
                                    class:btn-disabled=!is_owner()
                                >
                                    "View Details"
                                    <Icon icon=i::MdiArrowRight width="1.5em" height="1.5em" />
                                </a>
                            </div>
                        </div>
                    </div>
                    <Show when=move || group.archive_date.is_some()>
                        <Alert icon=i::MdiDeleteAlert class="alert-error">
                            "These slides have been deleted on "
                            {move || fmt_datetime_opt(group.archive_date.as_ref(), "None")}
                            " and can't be edited further"
                        </Alert>
                    </Show>
                    <div class="grid grid-cols-3 gap-4">
                        {move || match group.created_by.clone() {
                            common::dtos::OwnerDto::User(username) => {
                                view! {
                                    <PropertyDisplay icon=i::MdiAccount>{username}</PropertyDisplay>
                                }
                            }
                            common::dtos::OwnerDto::Group(group) => {
                                view! {
                                    <PropertyDisplay icon=i::MdiAccountGroup>
                                        {group.name}
                                    </PropertyDisplay>
                                }
                            }
                        }}
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
                                "Hidden from others",
                                "Shown to everyone",
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
            }} <SlideList slide_group=slide_group editable=false/>
        </div>
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
