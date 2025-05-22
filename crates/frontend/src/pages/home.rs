use crate::{
    api,
    components::{error::ErrorList, slide_group::SlideGroupOverview},
    context::ScreenContext,
};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups = LocalResource::new(async move || api::list_slide_groups().await);

    let screens = LocalResource::new(move || async move { api::list_screens().await });
    provide_context(ScreenContext { screens });

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>

                <div class="container m-auto my-4">
                    <div class="text-right">
                        <a class="btn" href="/new">
                            "Create New"
                        </a>
                    </div>
                    {move || Suspend::new(async move {
                        slide_groups
                            .await
                            .map(|groups| {
                                groups
                                    .iter()
                                    .map(|group| {
                                        view! {
                                            <div class="card my-8">
                                                <SlideGroupOverview slide_group=RwSignal::new(group.clone())
                                                    .into() />
                                            </div>
                                        }
                                    })
                                    .collect_view()
                            })
                    })}

                </div>
            </ErrorBoundary>
        </Transition>
    }
}
