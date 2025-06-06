use std::hash::Hash;

use leptos::{prelude::*, tachys::view::keyed::SerializableKey};

/// Enhanced For component to loop over Vec while memoizing each item.
#[component]
pub fn ForVecMemo<T, K, KF, EF, N>(
    #[prop(into)] vec: Signal<Vec<T>>,
    key: KF,
    #[prop(optional, into)] fallback: ViewFn,
    children: EF,
) -> impl IntoView
where
    T: Send + Sync + Clone + PartialEq + Default + 'static,
    KF: Send + Sync + Fn(&T) -> K + Send + Clone + 'static,
    K: Eq + Hash + SerializableKey + 'static,
    EF: Fn(Memo<T>) -> N + Send + Clone + 'static,
    N: IntoView + 'static,
{
    let fallback = StoredValue::new(fallback);

    view! {
        <ForEnumerate
            each=move || vec.get()
            key=move |el| key(el)
            children=move |index, _| {
                let el_memo = Memo::new(move |_| {
                    vec.with(|v| v.get(index.get()).cloned().unwrap_or_default())
                });
                children(el_memo)
            }
        />
        <Show when=move || vec.read().is_empty()>{move || { fallback.read_value().run() }}</Show>
    }
}
