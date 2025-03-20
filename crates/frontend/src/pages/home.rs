use crate::{api, components::counter_btn::Button};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups = LocalResource::new(async move || api::list_slide_groups().await);

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Uh oh! Something went wrong!"</h1>

                    <p>"Errors: "</p>
                    // Render a list of errors as strings - good for development purposes
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect_view()
                        }}

                    </ul>
                }
            }>

                <div class="container">

                    <picture>
                        <source
                            srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_pref_dark_RGB.svg"
                            media="(prefers-color-scheme: dark)"
                        />
                        <img
                            src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg"
                            alt="Leptos Logo"
                            height="200"
                            width="400"
                        />
                    </picture>

                    <h1>"Welcome to Leptos"</h1>

                    <div class="buttons">
                        <Button />
                        <Button increment=5 />
                    </div>

                    {move || Suspend::new(async move {
                        slide_groups
                            .await
                            .map(|groups| {
                                groups
                                    .iter()
                                    .map(|group| view! { <p>{group.title.clone()}</p> })
                                    .collect_view()
                            })
                    })}

                </div>
            </ErrorBoundary>
        </Transition>
    }
}
