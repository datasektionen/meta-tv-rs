//! Widget which includes two date inputs, meant for setting the start and end dates of a slide
//! group.

use crate::{
    components::utils::{If, Otherwise, Then},
    utils::datetime::{datetime_to_input, input_to_datetime},
};

use chrono::{DateTime, Utc};
use icondata as i;
use leptos::{html::Div, prelude::*};
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
    let container = NodeRef::<Div>::new();
    let first_input_container = NodeRef::<Div>::new();
    let second_input_container = NodeRef::<Div>::new();
    let container_size = leptos_use::use_element_size(container);

    // If the second input has wrapped and is below the first.
    // Note: We have to track this using code since the arrow between the inputs should be hidden if
    //   they have been wrapped on separate lines, which is not something which we can select for
    //   using CSS.
    let layout_wrapped = Memo::new(move |_| {
        let Some(first_input_container) = first_input_container.get() else {
            return false;
        };
        let Some(second_input_container) = second_input_container.get() else {
            return false;
        };

        // Rerun calculation if the container changes width, as the wrapping of its flex layout
        // should only change if its width changes.
        container_size.width.track();

        let first_bounds = first_input_container.get_bounding_client_rect();
        let second_bounds = second_input_container.get_bounding_client_rect();
        return first_bounds.bottom() <= second_bounds.top();
    });

    view! {
        <div class=move || format!("{} max-w-[41rem]", class.get()) node_ref=container>
            // We avoid using flex-gap, since that would leave an additional gaps between the two
            // inputs whenever the arrow has wrapped to its own line.
            <div class="flex flex-wrap items-center gap-x-4 -mt-4">
                <div class="mt-4" node_ref=first_input_container>
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
                <div
                    class="shrink-0 overflow-hidden"
                    class:mt-4=move || !layout_wrapped.get()
                    class:h-0=layout_wrapped
                >
                    <Icon icon=i::MdiArrowRightBold />
                </div>
                <div class="mt-4" node_ref=second_input_container>
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
                <p class="text-sm/6 text-current/60">Dates are in Swedish time</p>
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
