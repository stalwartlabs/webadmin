/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

#[component]
pub fn Tab(
    #[prop(into)] tabs: MaybeSignal<Vec<Option<String>>>,
    children: Children,
) -> impl IntoView {
    let selected = RwSignal::new(0);
    let buttons = tabs.get().into_iter().enumerate().filter_map(|(id, name)| {
        name.map(|name| {
            view! {
                <button
                    type="button"
                    class=move || {
                        if selected.get() == id {
                            "hs-tab-active:font-semibold hs-tab-active:border-blue-600 hs-tab-active:text-blue-600 py-4 px-1 inline-flex items-center gap-x-2 border-b-2 border-transparent text-sm whitespace-nowrap text-gray-500 hover:text-blue-600 focus:outline-none focus:text-blue-600 disabled:opacity-50 disabled:pointer-events-none dark:text-neutral-400 dark:hover:text-blue-500 active"
                        } else {
                            "hs-tab-active:font-semibold hs-tab-active:border-blue-600 hs-tab-active:text-blue-600 py-4 px-1 inline-flex items-center gap-x-2 border-b-2 border-transparent text-sm whitespace-nowrap text-gray-500 hover:text-blue-600 focus:outline-none focus:text-blue-600 disabled:opacity-50 disabled:pointer-events-none dark:text-neutral-400 dark:hover:text-blue-500"
                        }
                    }

                    id=move || format!("tabs-with-underline-item-{}", id)
                    aria-selected=move || selected.get() == id
                    data-hs-tab=move || { format!("#tabs-with-underline-{}", id) }
                    aria-controls=move || { format!("tabs-with-underline-{}", id) }
                    role="tab"
                    on:click=move |_| {
                        selected.set(id);
                    }
                >

                    {name}
                </button>
            }
        })

    }).collect_view();

    let children = children()
        .nodes
        .into_iter()
        .enumerate()
        .map(|(id, child)| {
            view! {
                <div
                    id=move || format!("tabs-with-underline-{id}")
                    role="tabpanel"
                    aria-labelledby=move || format!("tabs-with-underline-item-{id}")
                    class:hidden=move || selected.get() != id
                >
                    {child}
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="border-b border-gray-200 dark:border-neutral-700">
            <nav
                class="flex gap-x-1"
                aria-label="Tabs"
                role="tablist"
                aria-orientation="horizontal"
            >
                {buttons}
            </nav>
        </div>
        <div class="mt-3">{children}</div>
    }
}
