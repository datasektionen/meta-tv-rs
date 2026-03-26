use crate::{
    api::{self, AppError},
    components::{error::ErrorList, start_end_date_input::StartEndDateInput},
};

use chrono::Utc;
use common::dtos::{CreateSlideGroupDto, GroupDto, UserInfoDto};
use leptos::{html, logging, prelude::*};
use leptos_router::components::Redirect;

/// Page with form to create a new slide group
#[component]
pub fn CreateSlideGroup() -> impl IntoView {
    let title = RwSignal::new("".to_string());

    let now = Utc::now();
    let start_date = RwSignal::new(now.clone());
    let end_date = RwSignal::new((
        true,
        now.with_timezone(&chrono_tz::Europe::Stockholm)
            .checked_add_days(chrono::Days::new(
                // If time is between 00:00 and 03:00, don't add day
                if now.time() < chrono::NaiveTime::from_hms_opt(3, 0, 0).expect("03:00 is a time") {
                    0
                } else {
                    1
                },
            ))
            .unwrap_or_else(|| {
                now.with_timezone(&chrono_tz::Europe::Stockholm) + chrono::Duration::hours(24)
            })
            .with_time(
                chrono::NaiveTime::from_hms_opt(3, 0, 0)
                    .expect("Rasmus thinks time exist (same as before)"),
            ) // Make endtime 03:00 so things dissapear before people are back in META
            .earliest()
            .expect("Time is proably before Year 2038")
            .with_timezone(&chrono::Utc),
    ));

    let user_info = use_context::<LocalResource<Result<UserInfoDto, AppError>>>()
        .expect("User info has been provided");
    let available_owners = Memo::new(move |_| {
        [None]
            .into_iter()
            .chain(
                user_info
                    .get()
                    .and_then(|info| info.map(|info| info.memberships).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(GroupDto::from)
                    .map(Some),
            )
            .collect::<Vec<_>>()
    });
    let username = move || {
        user_info
            .get()
            .map(|result| {
                result
                    .map(|info| info.username)
                    .inspect_err(|err| logging::error!("Failed fetching user info: {}", err))
                    .unwrap_or_else(|_| "Logged in user".to_owned())
            })
            .unwrap_or_else(|| "Loading username...".to_owned())
    };

    let select = NodeRef::<html::Select>::new();
    let selected_owner = move || {
        let index = select
            .get()
            .expect("select element has been initialized")
            .value()
            .parse::<usize>()
            .expect("Option's values are indices");

        available_owners.get()[index].clone()
    };

    let submit_action = Action::new_local(move |title: &String| {
        let title = title.clone();
        async move {
            api::create_slide_group(&CreateSlideGroupDto {
                title,
                priority: 0,
                hidden: false,
                start_date: start_date.get(),
                end_date: Some(end_date.get().1),
                owner: selected_owner(),
            })
            .await
        }
    });

    let is_submitting = submit_action.pending();
    let response = move || {
        submit_action
            .value()
            .get()
            .map(|res| res.map(|_| view! { <Redirect path="/" /> }.into_any()))
    };

    view! {
        <div class="container m-auto flex min-h-[80vh]">
            <div class="card m-auto w-[41rem] max-w-full">
                <form
                    class="card-body"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        submit_action.dispatch(title.read().to_string());
                    }
                >
                    <h1 class="card-title mb-2">"Create slide group"</h1>
                    <fieldset disabled=is_submitting>
                        <label class="input mb-4">
                            <span class="label">"Name"</span>
                            <input
                                type="text"
                                placeholder="My amazing slideshow"
                                bind:value=title
                            />
                        </label>

                        <label class="select mb-4">
                            <span class="label">"Owner"</span>
                            <select node_ref=select>
                                <ForEnumerate
                                    each=move || available_owners.get()
                                    key=|owner| owner.clone()
                                    children=move |index, owner| {
                                        let owner_copy = owner.clone();
                                        view! {
                                            <option selected=move || owner_copy.is_none() value=index>
                                                {move || {
                                                    owner
                                                        .clone()
                                                        .map(|owner| owner.name)
                                                        .unwrap_or_else(username)
                                                }}
                                            </option>
                                        }
                                    }
                                />
                            </select>
                        </label>

                        <fieldset>
                            <legend class="text-base/7 font-semibold">Active Timespan</legend>
                            <StartEndDateInput
                                class="mt-2"
                                start_date=start_date
                                end_date=end_date
                                disable_end_date_removal_reason=Some(
                                    "The end date can be removed after the slide group has been created.",
                                )
                            />
                        </fieldset>

                        <div class="card-actions justify-end">
                            <button type="submit" class="btn btn-primary">
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
