use leptos::prelude::*;

/// Show a list of errors.
/// Useful when combined with ErrorBoundary.
#[component]
pub fn ErrorList(errors: ArcRwSignal<Errors>) -> impl IntoView {
    view! {
        <h1>"Uh oh! Something went wrong!"</h1>

        <p>"Errors: "</p>
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
    .into_any()
}
