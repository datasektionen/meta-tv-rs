//! Widget which includes two date inputs, meant for setting the start and end dates of a slide
//! group.

use crate::{
    components::utils::{If, Otherwise, Then},
    utils::datetime::{datetime_to_input, input_to_datetime},
};

use chrono::{DateTime, Utc};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn StartEndDateInput(
    start_date: RwSignal<DateTime<Utc>>,
    end_date: RwSignal<(bool, DateTime<Utc>)>,
    disable_end_date_removal_reason: Option<&'static str>,
    #[prop(into, default = "".into())] class: Signal<&'static str>,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let end_date_text = move || {
        if end_date.get().0 {
            "Remove end date"
        } else {
            "Add end date"
        }
    };

    view! {
        <div class=move || format!("{} @container max-w-[41rem]", class.get())>
            <div class="grid items-center gap-x-4 gap-y-4 grid-rows-auto grid-cols-1 @min-[34rem]:grid-cols-[minmax(min-content,19rem)_1em_minmax(min-content,19rem)]">
                <div>
                    <label class="input">
                        <span class="label">Start</span>
                        <input
                            type="datetime-local"
                            step=60
                            prop:value=move || { datetime_to_input(&start_date.get()) }
                            on:change:target=move |ev| {
                                if let Some(dt) = input_to_datetime(&ev.target().value()) {
                                    start_date.set(dt);
                                }
                            }
                            disabled=disabled
                        />
                    </label>
                </div>
                <div class="shrink-0 @max-[34rem]:hidden">
                    <Icon icon=i::MdiArrowRightBold />
                </div>
                <div>
                    <If cond=Signal::derive(move || end_date.get().0)>
                        <Then slot>
                            <label class="input">
                                <span class="label">
                                    "End"
                                    <Show when=move || {
                                        disable_end_date_removal_reason.is_some()
                                    }>
                                        <span
                                            class="tooltip focus:tooltip-open"
                                            data-tip=disable_end_date_removal_reason.unwrap_or_default()
                                            tabindex=0
                                        >
                                            <Icon
                                                icon=i::MdiInformationOutline
                                                width="1.1em"
                                                height="1.1em"
                                            />
                                        </span>
                                    </Show>
                                </span>
                                <input
                                    type="datetime-local"
                                    step=60
                                    prop:value=move || { datetime_to_input(&end_date.get().1) }
                                    on:change:target=move |ev| {
                                        if let Some(date) = input_to_datetime(
                                            &ev.target().value(),
                                        ) {
                                            end_date.set((true, date));
                                        }
                                    }
                                    disabled=disabled
                                />
                            </label>
                        </Then>
                        <Otherwise slot>
                            <span class="italic">"Forever"</span>
                        </Otherwise>
                    </If>
                </div>
            </div>
            <div class="flex justify-between items-top mt-3">
                <p class="text-sm/6 text-gray-600">Dates are in Swedish time</p>
                <Show when=move || disable_end_date_removal_reason.is_none()>
                    <button
                        class="btn"
                        type="button"
                        on:click=move |_| { end_date.update(|(enabled, _)| *enabled = !*enabled) }
                        disabled=disabled
                    >
                        {end_date_text}
                    </button>
                </Show>
            </div>
        </div>
    }
}
