use common::dtos::SlideGroupDto;
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::{
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
pub fn SlideGroupOverview(slide_group: Signal<SlideGroupDto>) -> impl IntoView {
    view! {
        <div>
            {move || {
                let group = slide_group.get();
                view! {
                    <div class="flex flex-wrap-reverse items-center justify-between gap-2 mb-6">
                        <h1 class="text-6xl">
                            <a href=format!("/slides/{}", group.id)>{group.title}</a>
                        </h1>
                        <div class="text-right grow">
                            <a
                                href=format!("/slides/{}", group.id)
                                class="btn inline-flex gap-2 items-center"
                            >
                                "View Details"
                                <Icon icon=i::MdiArrowRight width="1.5em" height="1.5em" />
                            </a>
                        </div>
                    </div>
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
            }} <SlideList slide_group=slide_group />
        </div>
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
}
