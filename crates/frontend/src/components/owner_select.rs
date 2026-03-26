use common::dtos::{GroupDto, OwnerDto, UserInfoDto};
use leptos::{logging, prelude::*};
use reactive_stores::Field;

use crate::api::AppError;

#[component]
pub fn OwnerSelect(#[prop(into)] owner: Field<OwnerDto>) -> impl IntoView {
    let user_info = use_context::<LocalResource<Result<UserInfoDto, AppError>>>()
        .expect("User info has been provided");

    let username = move || {
        user_info.get().and_then(|result| {
            result
                .map(|info| info.username)
                .inspect_err(|err| logging::error!("Failed fetching user info: {}", err))
                .ok()
        })
    };

    let initial_owner = owner.get_untracked();

    let available_owners = Memo::new(move |_| {
        let logged_in_username = username();
        // Add the initially set value.
        let mut owners = [initial_owner.clone()]
            .into_iter()
            // Add currently logged in user.
            .chain(match logged_in_username.clone() {
                Some(username) if Some(username.as_str()) != initial_owner.username() => {
                    Some(OwnerDto::User(username))
                }
                _ => None,
            })
            // Add groups.
            .chain(
                user_info
                    .get()
                    .and_then(|info| info.map(|info| info.memberships).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(GroupDto::from)
                    .map(OwnerDto::Group)
                    .filter(|owner| owner != &initial_owner),
            )
            .collect::<Vec<_>>();
        owners.sort();
        owners
    });

    view! {
        <select
            prop:value=move || {
                let owner = owner.get();
                available_owners.get().iter().position(|element| element == &owner).unwrap_or(0)
            }
            on:change:target=move |ev| {
                owner
                    .set(
                        available_owners
                            .get()[ev
                                .target()
                                .value()
                                .parse::<usize>()
                                .expect("Option's values are indices")]
                            .clone(),
                    );
            }
        >
            <ForEnumerate
                each=move || available_owners.get()
                key=|owner| owner.clone()
                children=move |index, owner| {
                    view! { <option value=index>{move || owner.name().to_owned()}</option> }
                }
            />
        </select>
    }
}
