use std::collections::HashSet;

use leptos::*;

#[component]
pub fn ColumnList(
    #[prop(into)] headers: MaybeSignal<Vec<String>>,
    #[prop(into, optional)] select_all: Option<Callback<(), Vec<String>>>,
    children: Children,
) -> impl IntoView {
    let headers_ = headers.clone();
    let total_columns = create_memo(move |_| headers_.get().len());
    let selected = use_context::<RwSignal<HashSet<String>>>().unwrap();
    let has_select_all = select_all.is_some();

    view! {
        <thead class="bg-gray-50 dark:bg-slate-800">
            <tr>
                <Show when=move || { has_select_all }>
                    <th scope="col" class="ps-6 py-3 text-start">
                        <label for="hs-at-with-checkboxes-main" class="flex">
                            <input
                                type="checkbox"
                                class="shrink-0 border-gray-300 rounded text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-600 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                on:change=move |ev| {
                                    selected
                                        .update(|t| {
                                            let items = select_all.unwrap().call(());
                                            if event_target_checked(&ev) {
                                                t.extend(items);
                                            } else {
                                                for item in items {
                                                    t.remove(&item);
                                                }
                                            }
                                        });
                                }
                            />

                            <span class="sr-only">Checkbox</span>
                        </label>
                    </th>
                </Show>

                <For
                    each=move || { headers.get().into_iter().enumerate().collect::<Vec<_>>() }

                    key=|(idx, header)| format!("{header}-{idx}")
                    children=move |(idx, header)| {
                        let class = if idx == 0 && has_select_all {
                            "ps-6 lg:ps-3 xl:ps-0 pe-6 py-3 text-start"
                        } else if idx == total_columns.get() - 1 {
                            "px-6 py-3 text-end"
                        } else {
                            "px-6 py-3 text-start"
                        };
                        view! {
                            <th scope="col" class=class>
                                <div class="flex items-center gap-x-2">
                                    <span class="text-xs font-semibold uppercase tracking-wide text-gray-800 dark:text-gray-200">
                                        {header}
                                    </span>
                                </div>
                            </th>
                        }
                    }
                />

            </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">

            {children()}
        </tbody>
    }
}
