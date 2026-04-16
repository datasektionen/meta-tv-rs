use leptos::prelude::*;

/// Shows the upload rules body.
#[component]
pub fn RulesBody() -> impl IntoView {
    view! {
        <ul
            role="list"
            class="list-outside list-disc ps-[2ch] marker:content-['§_'] marker:font-bold marker:text-[1.2em] flex flex-col gap-1 max-w-[60ch]"
        >
            <li>
                "Uploaded slide media must follow the "
                <a
                    class="link hover:text-current/80"
                    href="https://styrdokument.datasektionen.se/policies/uppforandepolicy"
                >
                    "data chapter's Code of Conduct"
                </a> " as well as the "
                <a
                    class="link hover:text-current/80"
                    href="https://storage.googleapis.com/medieteknik-static/documents/2024/3/19/Uppförandepolicy%20EN.pdf"
                >
                    "media chapter's Code of Conduct"
                </a> "."
            </li>
            <li>"Animated media must not contain blinking lights."</li>
            <li>
                "Slides may only advertise information related to the data or media chapters."
                <p class="ps-8 italic">
                    "This does not apply for slides only shown during an event."
                </p>
            </li>
            <li>
                "Per the "
                <a
                    class="link hover:text-current/80"
                    href="https://styrdokument.datasektionen.se/pm/pm_informationsspridning#7-användning-av-generativ-ai"
                >
                    "data chapter's PM for Information Exchange"
                </a>
                " as well as the media chapter's Policy for Officials, slide media must not use "
                "images or video created with generative AI."
            </li>
        </ul>
    }
}
