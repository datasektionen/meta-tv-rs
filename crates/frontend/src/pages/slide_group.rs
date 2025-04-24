use crate::{
    api,
    components::{error::ErrorList, slide_group_options::SlideGroupOptions},
    context::SlideGroupOptionsContext,
};
use leptos::prelude::*;
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

    let context = SlideGroupOptionsContext { slide_group };

    provide_context(context);

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>

                <div class="container m-auto px-10">
                    {move || Suspend::new(async move {
                        slide_group
                            .await
                            .map(|group| {
                                view! {
                                    <SlideGroupOptions
                                        slide_group=Signal::derive(move || group.clone())
                                        is_editing_options=is_editing_options
                                        set_editing_options=set_editing_options
                                    />
                                }
                            })
                    })}
                </div>
            </ErrorBoundary>
        </Transition>
    }
}
