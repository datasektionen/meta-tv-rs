use crate::{
    api,
    components::{error::ErrorList, slide_group::SlideGroupOverview, utils::ForVecMemo},
    context::ScreenContext,
};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups = LocalResource::new(async move || api::list_slide_groups().await);
    let slide_groups_memo = Memo::new(move |_| {
        slide_groups
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
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
        <Transition fallback=|| view! { <div>Loading...</div> }.into_any()>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>
                {move || Suspend::new(async move { slide_groups.await.map(|_| ()) })}
                {move || Suspend::new(async move { screens.await.map(|_| ()) })}
                <div class="container m-auto my-4">
                    <div class="text-right">
                        <a class="btn" href="/new">
                            "Create New"
                        </a>
                    </div>
                    <ForVecMemo
                        vec=slide_groups_memo
                        key=|slide_group| slide_group.id
                        children=move |group| {
                            view! {
                                <div class="card my-8">
                                    <SlideGroupOverview slide_group=group />
                                </div>
                            }
                                .into_any()
                        }
                    />
                </div>
            </ErrorBoundary>
        </Transition>
    }
    .into_any()
}
