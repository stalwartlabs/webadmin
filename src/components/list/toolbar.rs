/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;
use web_sys::wasm_bindgen::JsCast;

use crate::components::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonIcon {
    Add,
    Delete,
}

#[component]
pub fn ToolbarButton(
    #[prop(into)] text: MaybeSignal<String>,
    color: Color,
    #[prop(into)] on_click: Callback<(), ()>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = match color {
        Color::Blue => concat!("py-2 px-3 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg ","border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 ","disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"),
        Color::Red => concat!("py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg ","border border-gray-200 bg-white text-red-500 shadow-sm hover:bg-gray-50 disabled:opacity-50 ","disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:hover:bg-gray-800 ","dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"),
        Color::Gray => concat!("py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg ","border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 ","disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white ","dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"),
        _ => unimplemented!()
    };

    view! {
        <button class=class on:click=move |_| on_click.call(())>

            {children.map(|children| children())}

            {move || text.get()}
        </button>
    }
}

#[component]
pub fn SearchBox(
    #[prop(into)] value: MaybeSignal<Option<String>>,
    #[prop(into)] on_search: Callback<String, ()>,
) -> impl IntoView {
    let value_ = value.clone();
    view! {
        <div class="sm:col-span-1">
            <label for="hs-as-table-product-review-search" class="sr-only">
                Search
            </label>
            <div class="relative">
                <input
                    type="text"
                    class=concat!(
                        "py-2 px-3 ps-11 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 ",
                        "disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 ",
                        "dark:text-gray-400 dark:focus:ring-gray-600",
                    )

                    placeholder="Search"
                    prop:value=move || value_.get().unwrap_or_default()
                    on:keyup=move |ev| {
                        let key_code = ev.unchecked_ref::<web_sys::KeyboardEvent>().key_code();
                        if key_code == 13 {
                            let filter = event_target_value(&ev);
                            if filter != value.get().unwrap_or_default() {
                                on_search.call(filter);
                            }
                        }
                    }
                />

                <div class="absolute inset-y-0 start-0 flex items-center pointer-events-none ps-4">
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
            </div>
        </div>
    }
}
