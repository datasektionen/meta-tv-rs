use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn Alert(
    #[prop(into)] icon: Signal<icondata_core::Icon>,
    #[prop(into, optional)] class: MaybeProp<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=[
            "flex gap-4 items-center text-lg rounded-lg px-4 py-2 my-4",
            class.read().unwrap_or_default(),
        ]
            .join(" ")>
            <Icon icon=icon width="1.5em" height="1.5em" />
            <div>{children()}</div>
        </div>
    }
    .into_any()
}
