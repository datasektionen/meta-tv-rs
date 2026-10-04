use crate::{
    api,
    components::{error::ErrorList, slide_group::SlideGroup},
    context::ScreenContext,
};
use common::dtos::SlideGroupDto;
use leptos::prelude::*;
use leptos_icons::Icon;
use reactive_stores::{AtKeyed, Store};

#[derive(Clone)]
struct FilterOptions {
    pinned: Option<bool>,
    published: Option<bool>,
    visible: Option<bool>,
    has_end_date: Option<bool>,
}
impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            pinned: None,
            published: None,
            visible: None,
            has_end_date: None,
        }
    }
}

#[derive(Store, Debug, Clone)]
pub struct SlideGroups {
    #[store(key: i32 = |row| row.id)]
    rows: Vec<SlideGroupDto>,
}

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let slide_groups_resource = LocalResource::new(async move || api::list_slide_groups().await);
    // used as source of truth for displaying slides instead of the resource, to not redundantly refetch all slides if only one changes,
    // effectively fetching only on page reload
    let slide_groups = Store::new(SlideGroups { rows: Vec::new() });
    Effect::new(move || {
        if let Some(Ok(list)) = slide_groups_resource.get() {
            slide_groups.rows().set(list);
        }
    });

    let show_filters = RwSignal::new(false);
    let filters = RwSignal::new(FilterOptions::default());

    let screens_resource = LocalResource::new(move || async move { api::list_screens().await });
    let screens = Memo::new(move |_| {
        screens_resource
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });
    provide_context(ScreenContext { screens });

    view! {
        <Transition fallback=|| view! { <div>Loading...</div> }.into_any()>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorList errors=errors /> }
            }>
                {move || Suspend::new(async move { slide_groups_resource.await.map(|_| ()) })}
                {move || Suspend::new(async move { screens_resource.await.map(|_| ()) })}
                <div class="container m-auto my-4">
                    <div class="flex flex-row items-center justify-between gap-4">
                        // filters
                        <div class="flex flex-row gap-3 border-1 border-base-300 rounded-sm bg-base-200 p-2 flex-wrap">
                            <button
                                class="flex flex-row items-center gap-1"
                                on:click=move |_| {
                                    show_filters.set(!show_filters.get());
                                }
                            >
                                <span>"Filters"</span>
                                <div class=move || {
                                    if show_filters.get() {
                                        "w-fit h-fit rotate-180"
                                    } else {
                                        "w-fit h-fit rotate-90"
                                    }
                                }>
                                    <Icon
                                        icon=icondata::MdiTriangle
                                        width="0.5rem"
                                        height="0.5rem"
                                    />
                                </div>
                            </button>
                            <div class="flex flex-row flex-wrap gap-1">
                                <Show when=move || show_filters.get()>
                                    <label class="label select-none">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || {
                                                filters.get().pinned.is_some_and(|filter| filter)
                                            }
                                            prop:indeterminate=move || {
                                                filters.get().pinned.is_some_and(|filter| !filter)
                                            }
                                            on:click=move |_| {
                                                filters
                                                    .update(|filters| {
                                                        filters.pinned = match filters.pinned {
                                                            Some(true) => Some(false),
                                                            Some(false) => None,
                                                            None => Some(true),
                                                        };
                                                    });
                                            }
                                        />
                                        "Pinned"
                                    </label>
                                    <label class="label select-none">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || {
                                                filters.get().published.is_some_and(|filter| filter)
                                            }
                                            prop:indeterminate=move || {
                                                filters.get().published.is_some_and(|filter| !filter)
                                            }
                                            on:click=move |_| {
                                                filters
                                                    .update(|filters| {
                                                        filters.published = match filters.published {
                                                            Some(true) => Some(false),
                                                            Some(false) => None,
                                                            None => Some(true),
                                                        };
                                                    });
                                            }
                                        />
                                        "Published"
                                    </label>
                                    <label class="label select-none">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || {
                                                filters.get().visible.is_some_and(|filter| filter)
                                            }
                                            prop:indeterminate=move || {
                                                filters.get().visible.is_some_and(|filter| !filter)
                                            }
                                            on:click=move |_| {
                                                filters
                                                    .update(|filters| {
                                                        filters.visible = match filters.visible {
                                                            Some(true) => Some(false),
                                                            Some(false) => None,
                                                            None => Some(true),
                                                        };
                                                    });
                                            }
                                        />
                                        "Visible"
                                    </label>
                                    <label class="label select-none">
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            prop:checked=move || {
                                                filters.get().has_end_date.is_some_and(|filter| filter)
                                            }
                                            prop:indeterminate=move || {
                                                filters.get().has_end_date.is_some_and(|filter| !filter)
                                            }
                                            on:click=move |_| {
                                                filters
                                                    .update(|filters| {
                                                        filters.has_end_date = match filters.has_end_date {
                                                            Some(true) => Some(false),
                                                            Some(false) => None,
                                                            None => Some(true),
                                                        };
                                                    });
                                            }
                                        />
                                        "Has end date"
                                    </label>
                                </Show>
                            </div>
                        </div>

                        <a class="btn" href="/new">
                            "Create New"
                        </a>
                    </div>
                    <For
                        // filter out the outfiltered ones
                        each=move || {
                            slide_groups
                                .rows()
                                .into_iter()
                                .filter(move |slide_group| {
                                    let mut should_display = true;
                                    let filter_options = filters.get();
                                    if let Some(pinned) = filter_options.pinned {
                                        if slide_group.read().priority <= 0 {
                                            should_display &= !pinned;
                                        } else {
                                            should_display &= pinned;
                                        };
                                    }
                                    if let Some(published) = filter_options.published {
                                        if slide_group.read().published {
                                            should_display &= published;
                                        } else {
                                            should_display &= !published;
                                        };
                                    }
                                    if let Some(visible) = filter_options.visible {
                                        if slide_group.read().hidden {
                                            should_display &= !visible;
                                        } else {
                                            should_display &= visible;
                                        };
                                    }
                                    if let Some(has_end_date) = filter_options.has_end_date {
                                        if slide_group.read().end_date.is_some() {
                                            should_display &= has_end_date;
                                        } else {
                                            should_display &= !has_end_date;
                                        };
                                    }
                                    should_display
                                })
                        }
                        key=|slide_group| slide_group.get().id
                        children={move |
                            group: AtKeyed<
                                Store<SlideGroups>,
                                SlideGroups,
                                i32,
                                Vec<SlideGroupDto>,
                            >|
                        {
                            view! {
                                <div class="card my-8">
                                    <div class="card-body">
                                        <SlideGroup
                                            slide_group=group
                                            on_delete=move || {
                                                let id = group.key();
                                                slide_groups
                                                    .rows()
                                                    .update(move |rows| rows.retain(|r| r.id != id));
                                            }
                                        />
                                    </div>
                                </div>
                            }
                                .into_any()
                        }}
                    />
                </div>
            </ErrorBoundary>
        </Transition>
    }
    .into_any()
}
