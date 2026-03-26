use crate::{
    api,
    components::{error::ErrorList, slide_group::SlideGroup, utils::ForVecMemo},
    context::ScreenContext,
    utils::edit_slide_group::EditSlideGroup,
};
use leptos::prelude::*;
use reactive_stores::Store;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups_resource = LocalResource::new(async move || api::list_slide_groups().await);
    let slide_groups = RwSignal::new(Vec::new());
    Effect::new(move || {
        if let Some(result) = slide_groups_resource.get() {
            slide_groups.set(result.unwrap_or_default())
        }
    });

    let screens_resource = LocalResource::new(move || async move { api::list_screens().await });
    let screens = Memo::new(move |_| {
        screens_resource
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });
    provide_context(ScreenContext { screens });

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }.into_any()>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>
                {move || Suspend::new(async move { slide_groups_resource.await.map(|_| ()) })}
                {move || Suspend::new(async move { screens_resource.await.map(|_| ()) })}
                <div class="container m-auto my-4">
                    <div class="text-right">
                        <a class="btn" href="/new">
                            "Create New"
                        </a>
                    </div>
                    <ForVecMemo
                        vec=slide_groups
                        key=|slide_group| slide_group.id
                        children=move |group| {
                            let group = Store::new(EditSlideGroup::from(group.get()));
                            view! {
                                <div class="card my-8">
                                    <div class="card-body">
                                        <SlideGroup
                                            slide_group=group
                                            on_delete=move || {
                                                slide_groups
                                                    .update(move |slide_groups| {
                                                        let id = group.get().id;
                                                        slide_groups.retain(|slide_group| slide_group.id != id);
                                                    });
                                            }
                                        />
                                    </div>
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
