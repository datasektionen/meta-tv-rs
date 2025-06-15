use chrono::Utc;
use common::dtos::{ContentType, FeedEntryDto};
use gloo_timers::callback::Timeout;
use leptos::prelude::*;

#[component]
/// Handle the feed data and change slides based on time
pub fn ScreenFeedSlideshow(feed: Signal<Vec<FeedEntryDto>>) -> impl IntoView {
    let refresh_slide_signal = RwSignal::new(0);
    let timeout_handle_signal = RwSignal::new_local(None::<Timeout>);
    let (slide, set_slide) = signal(None);

    Effect::new(move || {
        refresh_slide_signal.track();
        let feed = feed.read();
        timeout_handle_signal.update(|handle| {
            if let Some(handle) = handle.take() {
                handle.cancel();
            }
        });
        if let Some((timeout, entry)) = calculate_next_slide(&feed) {
            set_slide.set(Some(entry.clone()));
            let handle = Timeout::new(timeout, move || {
                *refresh_slide_signal.write() += 1;
            });
            timeout_handle_signal.set(Some(handle));
        }
    });
    on_cleanup(move || {
        timeout_handle_signal.update(|handle| {
            if let Some(handle) = handle.take() {
                handle.cancel();
            }
        });
    });

    view! {
        {move || {
            slide
                .read()
                .as_ref()
                .map(|entry| {
                    match entry.content_type {
                        ContentType::Image => {
                            view! {
                                <img
                                    class="object-contain h-screen w-screen"
                                    src=format!("/uploads/{}", entry.file_path)
                                />
                            }
                                .into_any()
                        }
                        ContentType::Video => {
                            view! {
                                <video
                                    autoplay
                                    loop
                                    muted
                                    preload="auto"
                                    playsinline
                                    class="object-contain h-screen w-screen"
                                    src=format!("/uploads/{}", entry.file_path)
                                    on:loadedmetadata:target=move |ev| {
                                        let target = ev.target();
                                        target.set_muted(true);
                                        target.set_loop(true);
                                        let _ = target.play();
                                    }
                                />
                            }
                                .into_any()
                        }
                        ContentType::Html => {
                            view! {
                                <iframe
                                    sandbox="allow-scripts"
                                    class="object-contain h-screen w-screen"
                                    src=format!("/uploads/{}", entry.file_path)
                                />
                            }
                                .into_any()
                        }
                    }
                })
        }}
    }
    .into_any()
}

/// Get a feed and calculate the slide that should be displayed, along with how long until the next
/// slide should be displayed.
///
/// Slides are aligned with time, and this calculated is therefore deterministic for a given
/// timestamp (this function automatically gets the current time).
fn calculate_next_slide(feed: &[FeedEntryDto]) -> Option<(u32, &FeedEntryDto)> {
    let total_time: i32 = Some(feed.iter().map(|e| e.duration).sum::<i32>()).filter(|t| *t > 0)?;

    let now = Utc::now().timestamp_millis();
    let offset = (now % total_time as i64) as i32;

    let (ellapsed_duration, entry) = feed
        .iter()
        .scan(0, |ellapsed_duration, entry| {
            *ellapsed_duration += entry.duration;
            Some((*ellapsed_duration, entry))
        })
        .find(|(ellapsed_duration, _)| *ellapsed_duration > offset)?;

    let remaining_time: u32 = (ellapsed_duration - offset).try_into().ok()?;
    Some((remaining_time, entry))
}
