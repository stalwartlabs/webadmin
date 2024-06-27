/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod header;
pub mod pagination;
pub mod row;
pub mod table;
pub mod toolbar;

use leptos::*;

use crate::components::{icon::IconPlus, messages::alert::Alerts};

#[slot]
pub struct Toolbar {
    children: Children,
}

#[slot]
pub struct Footer {
    children: Children,
}

#[component]
pub fn ListTable(
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] subtitle: MaybeSignal<String>,
    children: Children,
    toolbar: Toolbar,
    footer: Footer,
) -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <div class="-m-1.5 overflow-x-auto">
                <div class="p-1.5 min-w-full inline-block align-middle">
                    <div class="bg-white border border-gray-200 rounded-xl shadow-sm overflow-hidden dark:bg-slate-900 dark:border-gray-700">
                        <div class="px-6 py-4 grid gap-3 md:flex md:justify-between md:items-center border-b border-gray-200 dark:border-gray-700">

                            {move || {
                                let title = title.get();
                                if !title.is_empty() {
                                    Some(
                                        view! {
                                            <div>
                                                <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200">
                                                    {title}
                                                </h2>
                                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                                    {subtitle.get()}
                                                </p>
                                            </div>
                                        },
                                    )
                                } else {
                                    None
                                }
                            }}
                            <div>
                                <div class="inline-flex gap-x-2">{(toolbar.children)()}</div>
                            </div>

                        </div>

                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            {children()}
                        </table>

                        {(footer.children)()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ListSection(children: Children) -> impl IntoView {
    view! {
        <div class="max-w-[85rem] px-4 py-10 sm:px-6 lg:px-8 lg:py-14 mx-auto">
            <Alerts/>
            {children()}
        </div>
    }
}

#[component]
pub fn ListItem(
    #[prop(into, optional)] class: Option<String>,
    #[prop(into, optional)] subclass: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <td class=class.unwrap_or_else(|| "size-px whitespace-nowrap".to_string())>
            <div class=subclass.unwrap_or_else(|| "ps-6 py-3".to_string())>{children()}</div>
        </td>
    }
}

#[component]
pub fn ListTextItem(children: Children) -> impl IntoView {
    view! {
        <td class="size-px whitespace-nowrap">
            <div class="ps-6 py-3">
                <span class="text-sm text-gray-500">{children()}</span>
            </div>
        </td>
    }
}

#[component]
pub fn ZeroResults(
    #[prop(into)] title: MaybeSignal<String>,
    #[prop(into)] subtitle: MaybeSignal<String>,
    #[prop(into, optional)] button_text: MaybeSignal<String>,
    #[prop(into, optional)] button_action: Option<Callback<(), ()>>,
) -> impl IntoView {
    let has_button = button_action.is_some();

    view! {
        <div class="max-w-sm w-full min-h-[400px] flex flex-col justify-center mx-auto px-6 py-4">
            <div class="flex justify-center items-center size-[46px] bg-gray-100 rounded-lg dark:bg-gray-800">
                <svg
                    class="size-4 text-gray-400"
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    fill="currentColor"
                    viewBox="0 0 16 16"
                >
                    <path d=concat!(
                        "M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.",
                        "115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 ",
                        "1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 ",
                        "0 5.5 5.5 0 0 1 11 0z",
                    )></path>
                </svg>

            </div>

            <h2 class="mt-5 font-semibold text-gray-800 dark:text-white">{title}</h2>
            <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">{subtitle}</p>

            <Show when=move || { has_button }>

                <div class="mt-5 grid sm:flex gap-2">
                    <button
                        type="button"
                        class="py-2 px-3 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        on:click=move |_| button_action.as_ref().unwrap().call(())
                    >

                        <IconPlus/>
                        {button_text.get()}
                    </button>

                </div>

            </Show>
        </div>
    }
}
