use common::dtos::{ContentDto, ContentType, CreateContentDto};
use leptos::prelude::*;
use web_sys::File;

use crate::{api, components::dialog::Dialog, context::SlideGroupOptionsContext};

#[component]
pub fn ContentItem(
    #[prop()] screen_id: i32,
    #[prop()] slide_id: i32,
    content: Signal<Option<ContentDto>>,
) -> impl IntoView {
    let is_upload_dialog_open = RwSignal::new(false);

    view! {
        <div class="aspect-16/9 h-40">
            <UploadContentDialog screen_id=screen_id slide_id=slide_id open=is_upload_dialog_open />
            {move || {
                if let Some(content) = content.get() {
                    match content.content_type {
                        ContentType::Html => todo!(),
                        ContentType::Image => {
                            view! {
                                <img
                                    class="object-contain h-full w-full"
                                    src=format!("/uploads/{}", content.file_path)
                                />
                            }
                                .into_any()
                        }
                        ContentType::Video => todo!(),
                    }
                } else {
                    view! {
                        <button
                            class="w-full h-full border"
                            on:click=move |_| is_upload_dialog_open.set(true)
                        >
                            "+"
                        </button>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn UploadContentDialog(
    #[prop()] screen_id: i32,
    #[prop()] slide_id: i32,
    open: RwSignal<bool>,
) -> impl IntoView {
    let input_ref = NodeRef::new();

    let upload_action = Action::new_local(move |file: &File| {
        let file = file.clone();
        let mime_type = file.type_();
        let content_type = if mime_type.starts_with("image/") {
            ContentType::Image
        } else if mime_type.starts_with("video/*") {
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

    let page_context =
        use_context::<SlideGroupOptionsContext>().expect("to have found the context");

    let is_submitting = upload_action.pending();
    let response = move || upload_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.slide_group.refetch();
            open.set(false);
        }
    });

    view! {
        <Dialog open=open>
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
                    <input
                        node_ref=input_ref
                        type="file"
                        accept="image/*,video/*,text/html"
                        required="true"
                    />
                    <button type="submit">"Upload"</button>
                </fieldset>
            </form>
        </Dialog>
    }
}
