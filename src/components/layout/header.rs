/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::icon::{IconAdjustmentsHorizontal, IconServer, IconUserCircle},
    core::url::UrlBuilder,
    pages::config::edit::DEFAULT_SETTINGS_URL,
};
use web_sys::wasm_bindgen::JsCast;

#[component]
pub fn Header(is_admin: MaybeSignal<bool>) -> impl IntoView {
    view! {
        <header class="sticky top-0 inset-x-0 flex flex-wrap sm:justify-start sm:flex-nowrap z-[48] w-full bg-white border-b text-sm py-2.5 sm:py-4 lg:ps-64 dark:bg-gray-800 dark:border-gray-700">
            <nav class="flex basis-full items-center w-full mx-auto px-4 sm:px-6 md:px-8">

                <div class="me-5 lg:me-0 lg:hidden">
                    <img src="/logo.svg"/>
                </div>

                <div class="w-full flex items-center justify-end ms-auto sm:justify-between sm:gap-x-3 sm:order-3">

                    <Show when=move || is_admin.get()>
                        <div class="sm:hidden">
                            <button
                                type="button"
                                class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            >
                                <svg
                                    class="flex-shrink-0 size-4"
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="24"
                                    height="24"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <circle cx="11" cy="11" r="8"></circle>
                                    <path d="m21 21-4.3-4.3"></path>
                                </svg>
                            </button>
                        </div>

                        <div class="hidden sm:block">
                            <div class="relative">
                                <div class="absolute inset-y-0 start-0 flex items-center pointer-events-none z-20 ps-4">
                                    <svg
                                        class="flex-shrink-0 size-4 text-gray-400"
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="24"
                                        height="24"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    >
                                        <circle cx="11" cy="11" r="8"></circle>
                                        <path d="m21 21-4.3-4.3"></path>
                                    </svg>
                                </div>
                                <input
                                    type="text"
                                    class="py-2 px-4 ps-11 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                    placeholder="Search settings"
                                    on:keyup=move |ev| {
                                        let key_code = ev
                                            .unchecked_ref::<web_sys::KeyboardEvent>()
                                            .key_code();
                                        if key_code == 13 {
                                            let filter = event_target_value(&ev);
                                            let query = filter.trim();
                                            if !query.is_empty() {
                                                use_navigate()(
                                                    &UrlBuilder::new("/settings/search")
                                                        .with_parameter("query", query)
                                                        .finish(),
                                                    Default::default(),
                                                );
                                            }
                                        }
                                    }
                                />

                            </div>
                        </div>

                    </Show>

                    <div class="flex flex-row items-center justify-end gap-2">
                        <a
                            href="/manage/directory/accounts"
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            title="Management"
                            class:hidden=move || { !is_admin.get() }
                        >
                            <IconServer/>
                        </a>

                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            href=DEFAULT_SETTINGS_URL
                            title="Settings"
                            class:hidden=move || { !is_admin.get() }
                        >
                            <IconAdjustmentsHorizontal/>

                        </a>
                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            href="/account/crypto"
                            title="Account"
                        >
                            <IconUserCircle/>

                        </a>
                    </div>
                </div>
            </nav>
        </header>
    }
}
