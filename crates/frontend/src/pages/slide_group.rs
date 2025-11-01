use crate::{
    api,
    components::{error::ErrorList, slide_group_options::SlideGroupOptions},
    context::{ScreenContext, SlideGroupOptionsContext},
};
use common::dtos::GroupDto;
use leptos::{logging, prelude::*};
use leptos_router::{hooks::use_params, params::Params};

#[derive(Params, PartialEq)]
struct EditSlideGroupParams {
    id: Option<i32>,
}

/// Page to edit slide group
#[component]
pub fn EditSlideGroup() -> impl IntoView {
    let params = use_params::<EditSlideGroupParams>();
    let (is_editing_options, set_editing_options) = signal(false);

    let slide_group = LocalResource::new(move || {
        let id = params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.id)
            .unwrap_or_default();
        async move { api::get_slide_group(id).await }
    });
    let slide_group_memo = Memo::new(move |_| {
        slide_group
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });
    let user_memberships = LocalResource::new(move || async move {
        api::user_memberships().await.unwrap_or_else(|error| {
            logging::error!("Error /auth/user_memberships: {}", error);
            // Is it ok to just return an empty list here?
            Vec::new()
        })
    });
    let user_memberships_memo = Memo::new(move |_| {
        user_memberships
            .get()
            .unwrap_or_default()
            .into_iter()
            .map(GroupDto::from)
            .collect::<Vec<_>>()
    });
    let refresh_action = Action::new(move |_: &()| {
        slide_group.refetch();
        async {}
    });

    provide_context(SlideGroupOptionsContext {
        refresh_group: refresh_action,
    });

    let screens = LocalResource::new(move || async move { api::list_screens().await });
    let screens_memo = Memo::new(move |_| {
        screens
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });
    provide_context(ScreenContext {
        screens: screens_memo,
    });

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }.into_any()
            }>
                {move || Suspend::new(async move { screens.await.map(|_| ()) })}
                {move || Suspend::new(async move { slide_group.await.map(|_| ()) })}
                <div class="container m-auto px-10">
                    <SlideGroupOptions
                        slide_group=slide_group_memo
                        user_memberships=user_memberships_memo
                        is_editing_options=is_editing_options
                        set_editing_options=set_editing_options
                    />
                </div>
            </ErrorBoundary>
        </Transition>
    }
    .into_any()
}
