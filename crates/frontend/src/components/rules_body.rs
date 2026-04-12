use leptos::prelude::*;

/// Shows the upload rules body.
#[component]
pub fn RulesBody() -> impl IntoView {
    view! {
        <ul
            role="list"
            class="list-outside list-disc ps-[2ch] marker:content-['§_'] marker:font-bold marker:text-[1.2em] flex flex-col gap-1"
        >
            <li>
                "Uploaded slide media must follow the chapters "
                <a href="https://styrdokument.datasektionen.se/policies/uppforandepolicy">
                    "Code of Conduct"
                </a> "."
            </li>
            <li>"Animated media must not contain blinking lights."</li>
            <li>
                "Slides may only advertise information related to the data or media chapters."
                <p class="ps-8 italic">
                    "This does not apply for slides only shown during an event."
                </p>
            </li>
        </ul>
    }
}
