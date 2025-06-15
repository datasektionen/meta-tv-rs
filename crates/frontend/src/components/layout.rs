use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::components::topbar::Topbar;

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <Topbar />
        <Outlet />
    }
    .into_any()
}
