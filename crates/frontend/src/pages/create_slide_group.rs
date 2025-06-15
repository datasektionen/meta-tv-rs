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
        async move {
            api::create_slide_group(&CreateSlideGroupDto {
                title,
                priority: 0,
                hidden: false,
                start_date: chrono::Utc::now(),
                end_date: None,
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
        <div class="container">
            <form on:submit=move |ev| {
                ev.prevent_default();
                submit_action.dispatch(title.read().to_string());
            }>
                <fieldset disabled=is_submitting>
                    <input
                        class="border disabled:bg-gray-50 disabled:text-gray-500"
                        type="text"
                        bind:value=title
                    />

                    <button type="submit" class="border disabled:text-gray-500">
                        Create
                    </button>
                </fieldset>
            </form>

            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }.into_any()
            }>{response}</ErrorBoundary>

        </div>
    }
    .into_any()
}
