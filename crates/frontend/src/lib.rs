use components::layout::Layout;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use pages::{
    create_slide_group::CreateSlideGroup, screen_feed::ScreenFeed, slide_group::EditSlideGroup,
};

// Modules
mod api;
mod components;
mod context;
mod pages;
mod utils;

// Top-Level pages
use crate::pages::home::Home;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" />

        // sets the document title
        <Title text="Welcome to Leptos CSR" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router>
            <Routes fallback=|| view! { NotFound }>
                <ParentRoute path=path!("/") view=Layout>
                    <Route path=path!("") view=Home />
                    <Route path=path!("new") view=CreateSlideGroup />
                    <Route path=path!("slides/:id") view=EditSlideGroup />
                </ParentRoute>
                <Route path=path!("/feed/:id") view=ScreenFeed />
            </Routes>
        </Router>
    }
}
