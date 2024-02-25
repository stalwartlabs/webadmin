use std::collections::HashSet;

use leptos::*;

#[component]
pub fn SelectItem(item_id: String) -> impl IntoView {
    let selected = use_context::<RwSignal<HashSet<String>>>().unwrap();
    let item_id_ = item_id.clone();

    view! {
        <input
            type="checkbox"
            class="shrink-0 border-gray-300 rounded text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-600 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
            prop:checked=move || selected.get().contains(&item_id_)
            on:input=move |_| {
                selected
                    .update(|t| {
                        if !t.remove(&item_id) {
                            t.insert(item_id.to_string());
                        }
                    })
            }
        />
    }
}
