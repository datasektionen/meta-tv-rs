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
    pinned: bool,
    published: bool,
    visible: bool,
    has_end_date: bool,
}
impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            pinned: false,
            published: false,
            visible: false,
            has_end_date: false,
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
    // used as source of truth instead of the resource, to not redundantly refetch all slides if only one change
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
                        <div class="flex flex-row gap-3 border-1 border-base-300 rounded-sm bg-base-200 p-2">
                            <button
                                class="flex flex-row items-center gap-1"
                                on:click=move |_| {
                                    show_filters.set(!show_filters.get());
                                }
                            >
                                <span>"Filters"</span>
                                <div
                                    class=move || {
                                        if show_filters.get() {
                                            "w-fit h-fit rotate-180"
                                        } else {
                                            "w-fit h-fit rotate-90"
                                        }
                                    }
                                >
                                    <Icon icon=icondata::MdiTriangle width="0.5rem" height="0.5rem"/>
                                </div>
                            </button>
                            <Show when=move || show_filters.get()>
                                <div class="flex flex-row gap-1">
                                    <input
                                        type="checkbox"
                                        class="checkbox"
                                        prop:checked=move || filters.get().pinned
                                        on:input:target=move |ev| {
                                            filters.update(|filters| {filters.pinned = ev.target().checked();});
                                        }
                                    />
                                    <span>"Pinned"</span>
                                </div>
                                <div class="flex flex-row gap-1">
                                    <input
                                        type="checkbox"
                                        class="checkbox"
                                        prop:checked=move || filters.get().published
                                        on:input:target=move |ev| {
                                            filters.update(|filters| {filters.published = ev.target().checked();});
                                        }
                                    />
                                    <span>"Published"</span>
                                </div>
                                <div class="flex flex-row gap-1">
                                    <input
                                        type="checkbox"
                                        class="checkbox"
                                        prop:checked=move || filters.get().visible
                                        on:input:target=move |ev| {
                                            filters.update(|filters| {filters.visible = ev.target().checked();});
                                        }
                                    />
                                    <span>"Visible"</span>
                                </div>
                                <div class="flex flex-row gap-1">
                                    <input
                                        type="checkbox"
                                        class="checkbox"
                                        prop:checked=move || filters.get().has_end_date
                                        on:input:target=move |ev| {
                                            filters.update(|filters| {filters.has_end_date = ev.target().checked();});
                                        }
                                    />
                                    <span>"Has end date"</span>
                                </div>
                            </Show>
                        </div>

                        <a class="btn" href="/new">
                            "Create New"
                        </a>
                    </div>
                    <For
                        // filter out the outfiltered ones
                        each=move || {slide_groups.rows().into_iter()
                                .filter(move |slide_group| {
                                    let filter_options = filters.get();
                                    if filter_options.pinned && slide_group.read().priority <= 0 {
                                        return false;
                                    }
                                    if filter_options.published && !slide_group.read().published {
                                        return false;
                                    }
                                    if filter_options.visible && slide_group.read().hidden {
                                        return false;
                                    }
                                    if filter_options.has_end_date && !slide_group.read().end_date.is_some() {
                                        return false;
                                    }
                                    true
                                })}
                        key=|slide_group| slide_group.get().id
                        children={move |group: AtKeyed<Store<SlideGroups>, SlideGroups, i32, Vec<SlideGroupDto>>| {
                            view! {
                                <div class="card my-8">
                                    <div class="card-body">
                                        <SlideGroup
                                            slide_group=group
                                            on_delete=move || {
                                                slide_groups
                                                    .update(move |slide_groups| {
                                                        let id = group.get().id;
                                                        slide_groups.rows.retain(|slide_group| slide_group.id != id);
                                                    });
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
