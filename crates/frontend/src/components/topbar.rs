use leptos::prelude::*;

use crate::api::{self, AppError};

#[component]
pub fn Topbar() -> impl IntoView {
    let user_info = LocalResource::new(async move || api::user_info().await);

    view! {
        <div class="container m-auto min-h-10 flex justify-between py-2">
            <h1 class="text-3xl">
                <a href="/">"META-TV"</a>
            </h1>
            <div class="text-xl">
                <Transition fallback=move || {
                    view! { <div class="skeleton w-50 h-[1.2em]" /> }.into_any()
                }>
                    <ErrorBoundary fallback=move |_| {
                        view! { <div>"error :("</div> }.into_any()
                    }>
                        {move || Suspend::new(async move {
                            let data = user_info.await;
                            match data {
                                Ok(data) => {
                                    Ok(
                                        view! {
                                            <p>
                                                "Welcome, "{data.username}
                                                <a
                                                    href="/auth/logout"
                                                    rel="external"
                                                    class="btn text-base ml-1"
                                                >
                                                    Logout
                                                </a>
                                            </p>
                                        }
                                            .into_any(),
                                    )
                                }
                                Err(AppError::Api(_, 401)) => {
                                    Ok(
                                        view! {
                                            <a href="/auth/login" rel="external" class="btn text-base">
                                                "Login"
                                            </a>
                                        }
                                            .into_any(),
                                    )
                                }
                                Err(err) => Err(err),
                            }
                        })}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
    }
    .into_any()
}
