use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DOM_ID: AtomicUsize = AtomicUsize::new(1);

pub fn next_dom_id(prefix: &str) -> String {
    let id = NEXT_DOM_ID.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{id}")
}
