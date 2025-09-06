use common::dtos::{ContentDto, ContentType, CreateContentDto, ScreenDto};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use web_sys::File;

use crate::{api, components::dialog::Dialog, context::SlideGroupOptionsContext};

#[component]
pub fn ContentItem(
    #[prop()] screen: ScreenDto,
    #[prop()] slide_id: i32,
    #[prop(into)] content: Signal<Option<ContentDto>>,
) -> impl IntoView {
    let is_upload_dialog_open = RwSignal::new(false);

    let is_readonly = use_context::<SlideGroupOptionsContext>().is_none();

    view! {
        <div>
            <p class="uppercase text-current/80 font-bold text-sm">{screen.name}</p>
            <div class="aspect-16/9 h-40 border">
                <UploadContentDialog
                    screen_id=screen.id
                    slide_id=slide_id
                    open=is_upload_dialog_open
                />
                {move || {
                    if let Some(content) = content.get() {
                        match content.content_type {
                            ContentType::Image => {
                                view! {
                                    <img
                                        class="object-contain h-full w-full"
                                        src=format!("/uploads/{}", content.file_path)
                                    />
                                }
                                    .into_any()
                            }
                            ContentType::Video => {
                                view! {
                                    <video
                                        controls
                                        muted
                                        preload="metadata"
                                        class="object-contain h-full w-full"
                                        src=format!("/uploads/{}", content.file_path)
                                    />
                                }
                                    .into_any()
                            }
                            ContentType::Html => {
                                view! {
                                    <iframe
                                        sandbox=""
                                        class="object-contain h-full w-full"
                                        src=format!("/uploads/{}", content.file_path)
                                    />
                                }
                                    .into_any()
                            }
                        }
                    } else if is_readonly {
                        view! {
                            <div class="w-full h-full bg-base-200 flex gap-2 flex-col text-xl justify-center items-center">
                                <Icon icon=i::MdiFileDocumentAlert width="2em" height="2em" />
                                "Empty"
                            </div>
                        }
                            .into_any()
                    } else {
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
                }}
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn UploadContentDialog(
    #[prop()] screen_id: i32,
    #[prop()] slide_id: i32,
    open: RwSignal<bool>,
) -> impl IntoView {
    let input_ref = NodeRef::new();

    let Some(page_context) = use_context::<SlideGroupOptionsContext>() else {
        // if context is not available, then hide dialog
        return ().into_any();
    };

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
            slide: slide_id,
            screen: screen_id,
            content_type,
        };
        async move { api::upload_content(&data, &file).await }
    });

    let is_submitting = upload_action.pending();
    let response = move || upload_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.refresh_group.dispatch(());
            open.set(false);
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
            </div>
        </Dialog>
    }
    .into_any()
}
