use common::dtos::{CreateSlideDto, ScreenDto, SlideDto, SlideGroupDto};
use leptos::prelude::*;

use crate::{
    api,
    components::{content::ContentItem, error::ErrorList},
    context::SlideGroupOptionsContext,
};

#[component]
pub fn SlideList(slide_group: Signal<SlideGroupDto>) -> impl IntoView {
    let screens = LocalResource::new(move || async move { api::list_screens().await });

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>
                {move || Suspend::new(async move {
                    screens
                        .await
                        .map(|screens| {
                            view! { <SlideListInner screens=screens slide_group=slide_group /> }
                        })
                })}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn SlideListInner(
    #[prop()] screens: Vec<ScreenDto>,
    slide_group: Signal<SlideGroupDto>,
) -> impl IntoView {
    let group_id = slide_group.get_untracked().id;
    let max_position = move || {
        slide_group
            .get()
            .slides
            .iter()
            .map(|s| s.position)
            .max()
            .unwrap_or(-1)
    };

    view! {
        <For
            each=move || slide_group.get().slides.into_iter().enumerate()
            key=|(_, slide)| slide.id
            children=move |(index, _)| {
                let slide = Memo::new(move |_| {
                    slide_group.with(|slide_group| slide_group.slides.get(index).cloned())
                });
                slide
                    .get()
                    .map(|slide| {
                        view! {
                            <SlideRow
                                screens=screens.clone()
                                slide=Signal::derive(move || slide.clone())
                            />
                        }
                    })
            }
        />
        <AddSlideButton group_id=group_id max_position=Signal::derive(max_position) />
    }
}

#[component]
fn AddSlideButton(#[prop()] group_id: i32, max_position: Signal<i32>) -> impl IntoView {
    let create_action = Action::new_local(move |position: &i32| {
        let data = CreateSlideDto {
            position: *position,
            slide_group: group_id,
        };
        async move { api::create_slide(&data).await }
    });

    let page_context =
        use_context::<SlideGroupOptionsContext>().expect("to have found the context");

    let is_submitting = create_action.pending();
    let response = move || create_action.value().get().map(|r| r.map(|_| ()));
    Effect::new(move || {
        if response().map(|res| res.is_ok()).unwrap_or_default() {
            page_context.slide_group.refetch();
        }
    });

    view! {
        {response}
        <button
            disabled=is_submitting
            on:click=move |_| {
                create_action.dispatch(max_position.get() + 1);
            }
        >
            "Add Slide"
        </button>
    }
}

#[component]
fn SlideRow(#[prop()] screens: Vec<ScreenDto>, slide: Signal<SlideDto>) -> impl IntoView {
    view! {
        <div>
            <For
                each=move || screens.clone()
                key=|screen| screen.id
                children=move |screen| {
                    let content = Memo::new(move |_| {
                        slide
                            .with(|slide| {
                                slide.content.iter().find(|c| c.screen == screen.id).cloned()
                            })
                    });
                    view! {
                        <ContentItem
                            screen_id=screen.id
                            slide_id=slide.get_untracked().id
                            content=Signal::from(content)
                        />
                    }
                }
            />
        </div>
    }
}
