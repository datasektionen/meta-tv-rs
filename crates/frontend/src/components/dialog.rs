use leptos::{html, prelude::*};

#[component]
pub fn Dialog(open: RwSignal<bool>, children: Children) -> impl IntoView {
    let dialog_ref: NodeRef<html::Dialog> = NodeRef::new();
    Effect::new(move || {
        if let Some(dialog) = dialog_ref.get() {
            if open.get() {
                let _ = dialog.show_modal();
            } else {
                dialog.close();
            }
        }
    });

    view! {
        <dialog node_ref=dialog_ref on:close=move |_| { open.set(false) }>
            {children()}
        </dialog>
    }
}
