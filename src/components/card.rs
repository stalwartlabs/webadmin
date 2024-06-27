/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

#[slot]
pub struct CardItem {
    children: Children,
}

#[component]
pub fn Card(children: Children) -> impl IntoView {
    let nodes = children().nodes;
    let cols = nodes.len();
    let children = nodes
    .into_iter()
    .map(|child| view! {
        <a
            class="block p-4 md:p-5 relative bg-white hover:bg-gray-50 before:absolute before:top-0 before:start-0 before:w-full before:h-px md:before:w-px md:before:h-full before:bg-gray-200 before:first:bg-transparent dark:bg-slate-900 dark:hover:bg-slate-800 dark:before:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
            href="#"
        >
            <div class="flex md:grid lg:flex gap-y-3 gap-x-5">{child}</div>
        </a>
    })
    .collect_view();

    // Workaround for the fact that passing strings as a class prop doesn't work
    let class = if cols == 3 {
        "grid md:grid-cols-3 border border-gray-200 shadow-sm rounded-xl overflow-hidden dark:border-gray-700"
    } else {
        "grid md:grid-cols-4 border border-gray-200 shadow-sm rounded-xl overflow-hidden dark:border-gray-700"
    };

    view! {
        <div class="max-w-[85rem] px-4 py-2 sm:px-6 lg:px-8 lg:py-2 mx-auto">
            <div class=class>{children}</div>
        </div>
    }
}

#[component]
pub fn CardItem(
    #[prop(into)] title: MaybeSignal<String>,
    #[prop(into)] contents: MaybeSignal<String>,
    #[prop(into, optional)] subcontents: MaybeSignal<String>,
    #[prop(into, optional)] subcontents_bold: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        {children()}

        <div class="grow">
            <p class="text-xs uppercase tracking-wide font-medium text-gray-800 dark:text-gray-200">
                {title}
            </p>
            <h3 class="mt-1 text-xl sm:text-1xl font-semibold text-blue-600 dark:text-blue-500">
                {contents}
            </h3>
            <div class="mt-1 flex justify-between items-center">
                <p class="text-sm text-gray-500">
                    {subcontents}
                    <span class="font-semibold text-gray-800 dark:text-gray-200">
                        {subcontents_bold}
                    </span>
                </p>

            </div>
        </div>
    }
}
