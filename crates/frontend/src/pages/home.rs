use crate::{api, components::error::ErrorList};
use leptos::prelude::*;
use leptos_router::components::A;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups = LocalResource::new(async move || api::list_slide_groups().await);

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>

                <div class="container">
                    {move || Suspend::new(async move {
                        slide_groups
                            .await
                            .map(|groups| {
                                groups
                                    .iter()
                                    .map(|group| view! { <p>{group.title.clone()}</p> })
                                    .collect_view()
                            })
                    })} <A href="/new">Create new</A>

                </div>
            </ErrorBoundary>
        </Transition>
    }
}
