use leptos::prelude::*;
use reactive_stores::{Field, Store};

use crate::{
    components::{content::ContentItem, dialog::Dialog},
    context::ScreenContext,
    utils::edit_slide_group::{
        EditSlide, EditSlideGroup, EditSlideGroupStoreFields, EditSlideStoreFields,
    },
};

#[component]
pub fn SlideList(
    #[prop(into)] slide_group: Store<EditSlideGroup>,
    #[prop(into)] editable: Signal<bool>,
) -> impl IntoView {
    let add_slide = move || {
        slide_group.update(|slide_group| {
            let max_position = slide_group
                .slides
                .iter()
                .map(|s| s.position)
                .max()
                .unwrap_or(-1);
            slide_group.slides.push(EditSlide {
                existing: None,
                position: max_position + 1,
                content: Vec::new(),
            });
        });
    };

    view! {
        <For
            each=move || slide_group.slides()
            key=|slide| slide.key()
            children=move |slide| {
                view! {
                    <SlideRow
                        slide=slide
                        on_delete=move || {
                            let slide = slide.get_untracked();
                            slide_group
                                .update(move |slide_group| {
                                    slide_group.slides.retain(|iter_slide| iter_slide != &slide);
                                });
                        }
                        editable=editable
                    />
                }
                    .into_any()
            }
        />
        <Show when=move || slide_group.slides().get().len() == 0>
            <div class="h-60 text-center content-center bg-base-200 rounded-lg my-4">
                "There are currently no slides"
            </div>
        </Show>
        {move || {
            view! {
                <Show when=move || editable.get()>
                    <button
                        class="btn"
                        on:click=move |_| {
                            add_slide();
                        }
                    >
                        "Add Slide"
                    </button>
                </Show>
            }
                .into_any()
        }}
    }
    .into_any()
}

#[component]
fn SlideRow(
    #[prop(into)] slide: Field<EditSlide>,
    on_delete: impl Fn() + 'static + Send,
    editable: Signal<bool>,
) -> impl IntoView {
    let screens = use_context::<ScreenContext>()
        .expect("expected screen context")
        .screens;

    let is_delete_dialog_open = RwSignal::new(false);

    view! {
        <DeleteDialog open=is_delete_dialog_open on_delete=on_delete />
        <div class="my-6 @container">
            <div class="flex gap-[1rem] flex-wrap flex-col justify-center items-center @min-[56rem]:flex-row">
                <For
                    each=move || screens.get()
                    key=|screen| screen.id
                    children=move |screen| {
                        let content = Memo::new(move |_| {
                            slide
                                .with(|slide| {
                                    slide.content.iter().find(|c| c.screen == screen.id).cloned()
                                })
                        });
                        let screen_copy = screen.clone();
                        view! {
                            <ContentItem
                                attr:class="w-[18rem] grow"
                                screen=screen
                                content=content
                                on_submit=move |new_content| {
                                    let slide_content = slide.content();
                                    slide_content
                                        .update(|slide_content| {
                                            match slide_content
                                                .iter_mut()
                                                .find(move |content| content.screen == screen_copy.id)
                                            {
                                                Some(content) => *content = new_content,
                                                None => slide_content.push(new_content),
                                            }
                                        });
                                }
                                editable
                            />
                        }
                            .into_any()
                    }
                />
            </div>
            {move || {
                view! {
                    <Show when=move || editable.get()>
                        <div class="flex flex-row justify-center md:justify-start">
                            <button
                                class="btn btn-soft btn-choose btn-error my-3"
                                on:click=move |_| is_delete_dialog_open.set(true)
                            >
                                "Delete Slide"
                            </button>
                        </div>
                    </Show>
                }
                    .into_any()
            }}
        </div>
    }
    .into_any()
}

#[component]
pub fn DeleteDialog(open: RwSignal<bool>, on_delete: impl Fn() + 'static + Send) -> impl IntoView {
    view! {
        <Dialog open=open>
            <div class="card space-y-6 p-4">
                <div class="w-2xs">
                    <p>Are you sure you want to delete this slide</p>
                </div>
                <div class="mt-6 flex gap-3">
                    <button
                        class="btn btn-error"
                        type="button"
                        on:click=move |_| {
                            on_delete();
                            open.set(false);
                        }
                    >
                        "Delete"
                    </button>
                    <button class="btn" type="button" on:click=move |_| open.set(false)>
                        "Cancel"
                    </button>
                </div>
            </div>
        </Dialog>
    }
    .into_any()
}
