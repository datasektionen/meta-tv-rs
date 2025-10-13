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
                    now.clone()
                        .checked_add_months(chrono::Months::new(1))
                        // Adding one month may result in an invalid date time (e.g. due to daylight
                        // savings). Fallback to adding 30 days then to avoid the ambiguity.
                        .unwrap_or_else(|| now + chrono::Duration::days(30)),
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
