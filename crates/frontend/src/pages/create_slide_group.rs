use crate::{api, components::error::ErrorList};
use common::dtos::CreateSlideGroupDto;
use leptos::prelude::*;
use leptos_router::components::Redirect;

/// Page with form to create a new slide group
#[component]
pub fn CreateSlideGroup() -> impl IntoView {
    let title = RwSignal::new("".to_string());

    let submit_action = Action::new_local(|title: &String| {
        let title = title.clone();
        let now = chrono::Utc::now();
        async move {
            api::create_slide_group(&CreateSlideGroupDto {
                title,
                priority: 0,
                hidden: false,
                start_date: now.clone(),
                end_date: Some(
                    now.with_timezone(&chrono_tz::Europe::Stockholm)
                        .checked_add_days(chrono::Days::new(
                            // If time is between 00:00 and 03:00, don't add day
                            if now.time()
                                < chrono::NaiveTime::from_hms_opt(3, 0, 0).expect("03:00 is a time")
                            {
                                0
                            } else {
                                1
                            },
                        ))
                        .unwrap_or_else(|| {
                            now.with_timezone(&chrono_tz::Europe::Stockholm)
                                + chrono::Duration::hours(24)
                        })
                        .with_time(
                            chrono::NaiveTime::from_hms_opt(3, 0, 0)
                                .expect("Rasmus thinks time exist (same as before)"),
                        ) // Make endtime 03:00 so things dissapear before people are back in META
                        .earliest()
                        .expect("Time is proably before Year 2038")
                        .with_timezone(&chrono::Utc),
                ),
            })
            .await
        }
    });

    let is_submitting = submit_action.pending();
    let response = move || {
        submit_action.value().get().map(|res| {
            res.map(|created| {
                view! { <Redirect path=format!("/slides/{}", created.id) /> }.into_any()
            })
        })
    };

    view! {
        <div class="container m-auto flex min-h-[80vh]">
            <div class="card max-w-100 m-auto">
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit_action.dispatch(title.read().to_string());
                }>
                    <h1 class="text-4xl mb-4">"Create slide group"</h1>
                    <fieldset disabled=is_submitting>
                        <label class="label mb-4">
                            "Name"
                            <input
                                class="input"
                                type="text"
                                placeholder="My amazing slideshow"
                                bind:value=title
                            />
                        </label>

                        <div class="flex justify-end">
                            <button type="submit" class="btn">
                                Create
                            </button>
                        </div>
                    </fieldset>
                </form>
            </div>

            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }.into_any()
            }>{response}</ErrorBoundary>

        </div>
    }
    .into_any()
}
