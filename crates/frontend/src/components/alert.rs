use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn Alert(
    #[prop(into)] icon: Signal<icondata_core::Icon>,
    #[prop(into, optional)] class: MaybeProp<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div role="alert" class=["alert text-lg my-4", class.read().unwrap_or_default()].join(" ")>
            <Icon icon=icon width="1.5em" height="1.5em" />
            <span>{children()}</span>
        </div>
    }
    .into_any()
}
