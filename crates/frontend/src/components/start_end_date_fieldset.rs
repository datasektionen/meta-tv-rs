use crate::utils::datetime::{datetime_to_input, input_to_datetime};

use chrono::{DateTime, Utc};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn StartEndDateFieldset(
    start_date: RwSignal<DateTime<Utc>>,
    end_date: RwSignal<(bool, DateTime<Utc>)>,
    disable_end_date_removal_reason: Option<&'static str>,
    #[prop(into, default = false.into())]
    disabled: Signal<bool>
) -> impl IntoView {
    let end_date_text = move || {
        if end_date.get().0 {
            "Remove end date"
        } else {
            "Add end date"
        }
    };
    view! {
        <fieldset disabled=disabled>
            <legend class="text-base/7 font-semibold">Active Timespan</legend>
            <div class="grid grid-cols-[repeat(auto-fit,_minmax(14rem,_1fr))] gap-x-8 gap-y-4 mt-4">
                <div>
                    <label for="start-date" class="block text-sm/6 font-medium">
                        Start
                    </label>
                    <div class="mt-3">
                        <input
                            class="col-start-1 row-start-1 input border disabled:bg-gray-50 disabled:text-gray-500"
                            id="start-date"
                            type="datetime-local"
                            step=60
                            prop:value=move || { datetime_to_input(&start_date.get()) }
                            on:change:target=move |ev| {
                                if let Some(dt) = input_to_datetime(&ev.target().value()) {
                                    start_date.set(dt);
                                }
                            }
                        />
                    </div>
                </div>
                <div>
                    {move || {
                        match end_date.get() {
                            (true, end_date_value) => {
                                Some(
                                    view! {
                                        <div class="text-sm/6 font-medium flex flex-horizontal items-center gap-x-[0.3em]">
                                            <label for="end-date">End</label>
                                            <Show when=move || {
                                                disable_end_date_removal_reason.is_some()
                                            }>
                                                <span
                                                    class="tooltip"
                                                    data-tip=disable_end_date_removal_reason.unwrap_or_default()
                                                >
                                                    <Icon
                                                        icon=i::MdiInformationOutline
                                                        width="1.1em"
                                                        height="1.1em"
                                                    />
                                                </span>
                                            </Show>
                                        </div>
                                        <div class="mt-3">
                                            <input
                                                class="col-start-1 row-start-1 input border disabled:bg-gray-50 disabled:text-gray-500"
                                                id="end-date"
                                                type="datetime-local"
                                                step=60
                                                prop:value=move || { datetime_to_input(&end_date_value) }
                                                on:change:target=move |ev| {
                                                    if let Some(date) = input_to_datetime(
                                                        &ev.target().value(),
                                                    ) {
                                                        end_date.set((true, date));
                                                    }
                                                }
                                            />
                                        </div>
                                    },
                                )
                            }
                            _ => None,
                        }
                            .into_any()
                    }}
                </div>
            </div>
            <div class="flex justify-between items-top mt-3">
                <p class="text-sm/6 text-gray-600">Dates are in Swedish time</p>
                <Show when=move || disable_end_date_removal_reason.is_none()>
                    <button
                        class="btn"
                        type="button"
                        on:click=move |_| { end_date.update(|(enabled, _)| *enabled = !*enabled) }
                    >
                        {end_date_text}
                    </button>
                </Show>
            </div>
        </fieldset>
    }
}
