/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
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
    } else if cols == 4 {
        "grid md:grid-cols-4 border border-gray-200 shadow-sm rounded-xl overflow-hidden dark:border-gray-700"
    } else {
        "grid md:grid-cols-5 border border-gray-200 shadow-sm rounded-xl overflow-hidden dark:border-gray-700"
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
            <h3 class="mt-1 text-xl sm:text-1xl font-semibold text-black dark:text-white">
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

#[component]
pub fn CardSimple(children: Children) -> impl IntoView {
    let nodes = children().nodes;
    let cols = nodes.len();
    let children = nodes.into_iter().collect_view();

    // Workaround for the fact that passing strings as a class prop doesn't work
    let class = if cols == 3 {
        "grid sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6"
    } else if cols == 4 {
        "grid sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6"
    } else {
        "grid sm:grid-cols-2 lg:grid-cols-5 gap-4 sm:gap-6"
    };

    view! {
        <div class="max-w-[85rem] px-4 py-2 sm:px-6 lg:px-8 lg:py-2 mx-auto">
            <div class=class>{children}</div>
        </div>
    }
}

#[component]
pub fn CardSimpleItem(
    #[prop(into)] title: MaybeSignal<String>,
    #[prop(into)] contents: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-white border shadow-sm rounded-xl dark:bg-neutral-900 dark:border-neutral-800">
            <div class="p-4 md:p-5 flex gap-x-4">
                <div class="shrink-0 flex justify-center items-center size-[46px] bg-gray-100 rounded-lg dark:bg-neutral-800">
                    {children()}
                </div>

                <div class="grow">
                    <div class="flex items-center gap-x-2">
                        <p class="text-xs uppercase tracking-wide text-gray-500 dark:text-neutral-500">
                            {title}
                        </p>
                    </div>
                    <div class="mt-1 flex items-center gap-x-2">
                        <h3 class="text-xl font-medium text-gray-800 dark:text-neutral-200">
                            {contents}
                        </h3>
                    </div>
                </div>
            </div>
        </div>
    }
}
