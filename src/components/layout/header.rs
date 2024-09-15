/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::icon::{
        IconAdjustmentsHorizontal, IconHeart, IconPower, IconServer, IconUserCircle,
    },
    core::{oauth::use_authorization, url::UrlBuilder, AccessToken, Permission, Permissions},
    pages::config::edit::DEFAULT_SETTINGS_URL,
    STATE_STORAGE_KEY, VERSION_NAME,
};
use web_sys::wasm_bindgen::JsCast;

#[component]
pub fn Header(permissions: Memo<Option<Permissions>>) -> impl IntoView {
    view! {
        <header class="sticky top-0 inset-x-0 flex flex-wrap sm:justify-start sm:flex-nowrap z-[48] w-full bg-white border-b text-sm py-2.5 sm:py-4 lg:ps-64 dark:bg-gray-800 dark:border-gray-700">
            <nav class="flex basis-full items-center w-full mx-auto px-4 sm:px-6 md:px-8">

                <div class="me-5 lg:me-0 lg:hidden">
                    <img src="/logo.svg" title=VERSION_NAME/>
                </div>

                <div class="w-full flex items-center justify-end ms-auto sm:justify-between sm:gap-x-3 sm:order-3">

                    <Show when=move || {
                        permissions.get().map_or(false, |p| p.has_access(Permission::SettingsList))
                    }>
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
                            href=move || { permissions.get().map(|p| { p.default_url(false) }) }

                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            title="Management"
                            class:hidden=move || {
                                permissions.get().map_or(true, |p| { !p.has_admin_access() })
                            }
                        >

                            <IconServer/>
                        </a>

                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            href=DEFAULT_SETTINGS_URL
                            title="Settings"
                            class:hidden=move || {
                                permissions
                                    .get()
                                    .map_or(true, |p| !p.has_access(Permission::SettingsList))
                            }
                        >

                            <IconAdjustmentsHorizontal/>

                        </a>
                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            href=move || {
                                permissions
                                    .get()
                                    .map(|p| {
                                        if p.has_access(Permission::ManageEncryption) {
                                            "/account/crypto"
                                        } else {
                                            "/account/password"
                                        }
                                    })
                            }

                            title="Account"
                            class:hidden=move || {
                                permissions
                                    .get()
                                    .map_or(
                                        true,
                                        |p| {
                                            !p
                                                .has_access_any(
                                                    &[Permission::ManageEncryption, Permission::ManagePasswords],
                                                )
                                        },
                                    )
                            }
                        >

                            <IconUserCircle/>

                        </a>
                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            href="https://github.com/sponsors/stalwartlabs"
                            target="_blank"
                            title="Sponsor Stalwart open source"
                        >
                            <IconHeart/>

                        </a>
                        <a
                            class="w-[2.375rem] h-[2.375rem] inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            title="Logout"
                            on:click=move |_| {
                                SessionStorage::delete(STATE_STORAGE_KEY);
                                use_authorization().set(AccessToken::default());
                                use_navigate()("/login", Default::default());
                            }
                        >

                            <IconPower/>

                        </a>
                    </div>
                </div>
            </nav>
        </header>
    }
}
