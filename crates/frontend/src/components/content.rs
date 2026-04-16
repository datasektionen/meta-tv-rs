use common::dtos::{ContentDto, ContentType, CreateContentDto, ScreenDto};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use web_sys::File;

use crate::{
    api,
    components::{dialog::Dialog, error::ErrorList},
    utils::dom_id::next_dom_id,
};

#[component]
pub fn ContentItem(
    #[prop()] screen: ScreenDto,
    #[prop(into)] content: Signal<Option<ContentDto>>,
    // Function which is called with the resulting content ID after the user has uploaded content.
    on_submit: impl Fn(ContentDto) -> () + 'static,
    #[prop(into)] editable: Signal<bool>,
) -> impl IntoView {
    let is_upload_dialog_open = RwSignal::new(false);
    let content_description_id = next_dom_id("content-description");

    let display_content = |content: ContentDto| match content.content_type {
        ContentType::Image => {
            view! { <img class="object-contain h-full w-full" src=&content.url /> }.into_any()
        }
        ContentType::Video => view! {
            <video
                controls
                muted
                preload="metadata"
                class="object-contain h-full w-full"
                src=&content.url
            />
        }
        .into_any(),
        ContentType::Html => view! {
            <iframe
                sandbox="allow-scripts allow-same-origin"
                class="object-contain h-full w-full"
                src=&content.url
            />
        }
        .into_any(),
    };

    view! {
        <div>
            <p
                class="uppercase text-current/80 font-bold text-sm"
                id=content_description_id.clone()
            >
                {screen.name}
            </p>
            <div class="aspect-16/9 border">
                <UploadContentDialog
                    screen_id=screen.id
                    open=is_upload_dialog_open
                    on_submit=on_submit
                />
                {move || {
                    match (content.get(), editable.get()) {
                        (Some(content), true) => {
                            view! {
                                <button
                                    class="block h-full w-full"
                                    aria-label="Upload image"
                                    aria-describedby=content_description_id.clone()
                                    on:click=move |_| is_upload_dialog_open.set(true)
                                >
                                    {display_content(content)}
                                </button>
                            }
                                .into_any()
                        }
                        (Some(content), false) => display_content(content),
                        (None, true) => {
                            view! {
                                <button
                                    class="w-full h-full bg-base-200 flex gap-2 flex-col text-xl justify-center items-center"
                                    on:click=move |_| is_upload_dialog_open.set(true)
                                >
                                    <Icon icon=i::MdiPlus width="2em" height="2em" />
                                    "Upload"
                                </button>
                            }
                                .into_any()
                        }
                        (None, false) => {
                            view! {
                                <div class="w-full h-full bg-base-200 flex gap-2 flex-col text-xl justify-center items-center">
                                    <Icon icon=i::MdiFileDocumentAlert width="2em" height="2em" />
                                    "Empty"
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn UploadContentDialog(
    #[prop()] screen_id: i32,
    open: RwSignal<bool>,
    // Function which is called with the resulting populated content after the user has uploaded
    // content.
    on_submit: impl Fn(ContentDto) + 'static,
) -> impl IntoView {
    let input_ref = NodeRef::new();

    let upload_action = Action::new_local(move |file: &File| {
        let file = file.clone();
        let mime_type = file.type_();
        let content_type = if mime_type.starts_with("image/") {
            ContentType::Image
        } else if mime_type.starts_with("video/") {
            ContentType::Video
        } else {
            ContentType::Html
        };
        let data = CreateContentDto {
            screen: screen_id,
            content_type,
        };
        async move { api::upload_content(&data, &file).await }
    });

    let is_submitting = upload_action.pending();
    let response = move || upload_action.value().get();
    Effect::new(move || {
        if let Some(Ok(created)) = response() {
            open.set(false);
            // Make sure that the caller doesn't accidentally subscribe this effect to other
            // dependencies.
            untrack(|| {
                on_submit(created);
            });
        }
    });

    view! {
        <Dialog open=open>
            <div class="card space-y-6 p-4">
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    if let Some(file) = input_ref
                        .get()
                        .and_then(|input| { input.files() })
                        .and_then(|filelist| filelist.item(0))
                    {
                        upload_action.dispatch_local(file);
                    }
                }>
                    <fieldset disabled=is_submitting>
                        <div class="w-2xs overflow-hidden">
                            <input
                                class="rounded-sm input"
                                node_ref=input_ref
                                type="file"
                                accept="image/*,video/*,text/html"
                                required="true"
                            />
                        </div>
                        <div class="mt-6 flex gap-3">
                            <button class="btn" type="submit">
                                "Upload"
                            </button>
                            <button class="btn" type="button" on:click=move |_| open.set(false)>
                                "Cancel"
                            </button>
                        </div>
                    </fieldset>
                </form>

                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorList errors=errors /> }.into_any()
                }>{move || response().map(|r| r.map(|_| ()))}</ErrorBoundary>
            </div>
        </Dialog>
    }
    .into_any()
}
