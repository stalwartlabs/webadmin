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
    let show_action_dropdown = RwSignal::new(false);
    let show_account_dropdown = RwSignal::new(false);
    let auth_token = use_context::<RwSignal<AccessToken>>().unwrap();

    view! {
        <header class="sticky top-0 inset-x-0 flex flex-wrap sm:justify-start sm:flex-nowrap z-[48] w-full bg-white border-b text-sm py-2.5 sm:py-4 lg:ps-64 dark:bg-gray-800 dark:border-gray-700">
            <nav class="flex basis-full items-center w-full mx-auto px-4 sm:px-6 md:px-8">

                <div class="me-5 lg:me-0 lg:hidden">
                    <img src="/logo.svg" title=VERSION_NAME/>
                </div>

                <div class="w-full flex items-center justify-end sm:justify-between sm:gap-x-3 sm:order-3">

                    <Show when=move || {
                        permissions.get().is_some_and(|p| p.has_access(Permission::SettingsList))
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

                    <div class="flex flex-row items-center justify-end gap-2 ms-auto">

                        <div class="flex flex-row items-center justify-end gap-1">
                            <div class="hs-dropdown relative inline-flex">

                                <button
                                    type="button"
                                    class="size-[38px] relative inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-neutral-700 dark:focus:bg-neutral-700"
                                    on:click=move |_| {
                                        show_account_dropdown.set(false);
                                        show_action_dropdown
                                            .update(|v| {
                                                *v = !*v;
                                            });
                                    }
                                >

                                    <svg
                                        class="shrink-0 size-4"
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
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            d="M15.59 14.37a6 6 0 0 1-5.84 7.38v-4.8m5.84-2.58a14.98 14.98 0 0 0 6.16-12.12A14.98 14.98 0 0 0 9.631 8.41m5.96 5.96a14.926 14.926 0 0 1-5.841 2.58m-.119-8.54a6 6 0 0 0-7.381 5.84h4.8m2.581-5.84a14.927 14.927 0 0 0-2.58 5.84m2.699 2.7c-.103.021-.207.041-.311.06a15.09 15.09 0 0 1-2.448-2.448 14.9 14.9 0 0 1 .06-.312m-2.24 2.39a4.493 4.493 0 0 0-1.757 4.306 4.493 4.493 0 0 0 4.306-1.758M16.5 9a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Z"
                                        ></path>
                                    </svg>
                                    <span class="sr-only">Actions</span>
                                </button>

                                <div
                                    role="menu"
                                    class=move || {
                                        if show_action_dropdown.get() {
                                            "hs-dropdown-menu transition-[opacity,margin] absolute top-full right-0 duration opacity-100 open block divide-y divide-gray-200 min-w-40 z-50 bg-white shadow-2xl rounded-lg p-2 mt-2 dark:divide-neutral-700 dark:bg-neutral-800 dark:border dark:border-neutral-700"
                                        } else {
                                            "hs-dropdown-menu transition-[opacity,margin] duration hs-dropdown-open:opacity-100 opacity-0 hidden divide-y divide-gray-200 min-w-40 z-20 bg-white shadow-2xl rounded-lg p-2 mt-2 dark:divide-neutral-700 dark:bg-neutral-800 dark:border dark:border-neutral-700"
                                        }
                                    }
                                >

                                    <div class="py-3 px-5 bg-gray-100 rounded-t-lg dark:bg-neutral-700">
                                        <p class="text-sm text-gray-500 dark:text-neutral-500">
                                            Actions
                                        </p>
                                    </div>
                                    <div class="p-1.5 space-y-0.5">
                                        <a
                                            class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700 dark:focus:text-neutral-300"
                                            href=move || {
                                                permissions.get().map(|p| { p.default_url(false) })
                                            }

                                            class:hidden=move || {
                                                permissions
                                                    .get()
                                                    .map_or(true, |p| { !p.has_admin_access() })
                                            }
                                        >

                                            <IconServer/>
                                            Manage
                                        </a>

                                        <a
                                            class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700 dark:focus:text-neutral-300"
                                            href=DEFAULT_SETTINGS_URL
                                            class:hidden=move || {
                                                permissions
                                                    .get()
                                                    .map_or(true, |p| !p.has_access(Permission::SettingsList))
                                            }
                                        >

                                            <IconAdjustmentsHorizontal/>
                                            Configure
                                        </a>

                                        <a
                                            class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700 dark:focus:text-neutral-300"
                                            href="https://github.com/sponsors/stalwartlabs"
                                            target="_blank"
                                        >
                                            <IconHeart/>
                                            Sponsor
                                        </a>
                                    </div>

                                </div>
                            </div>

                            <div class="hs-dropdown relative inline-flex">

                                <button
                                    type="button"
                                    id="hs-dropdown-account"
                                    class="size-[38px] relative inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-neutral-700 dark:focus:bg-neutral-700"
                                    on:click=move |_| {
                                        show_action_dropdown.set(false);
                                        show_account_dropdown
                                            .update(|v| {
                                                *v = !*v;
                                            });
                                    }
                                >

                                    <svg
                                        class="shrink-0 size-4"
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
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
                                        ></path>
                                    </svg>
                                    <span class="sr-only">Account</span>
                                </button>

                                <div
                                    role="menu"
                                    class=move || {
                                        if show_account_dropdown.get() {
                                            "hs-dropdown-menu transition-[opacity,margin] absolute top-full right-0 duration opacity-100 open block divide-y divide-gray-200 min-w-40 z-50 bg-white shadow-2xl rounded-lg p-2 mt-2 dark:divide-neutral-700 dark:bg-neutral-800 dark:border dark:border-neutral-700"
                                        } else {
                                            "hs-dropdown-menu transition-[opacity,margin] duration hs-dropdown-open:opacity-100 opacity-0 hidden divide-y divide-gray-200 min-w-40 z-20 bg-white shadow-2xl rounded-lg p-2 mt-2 dark:divide-neutral-700 dark:bg-neutral-800 dark:border dark:border-neutral-700"
                                        }
                                    }
                                >

                                    <div class="py-3 px-5 bg-gray-100 rounded-t-lg dark:bg-neutral-700">
                                        <p class="text-sm text-gray-500 dark:text-neutral-500">
                                            Signed in as
                                        </p>
                                        <p class="text-sm font-medium text-gray-800 dark:text-neutral-200">
                                            {move || { auth_token.get().username.as_str().to_string() }}

                                        </p>
                                    </div>
                                    <div class="p-1.5 space-y-0.5">
                                        <a
                                            class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700 dark:focus:text-neutral-300"
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
                                            Account
                                        </a>
                                        <a
                                            class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700 dark:focus:text-neutral-300"
                                            on:click=move |_| {
                                                SessionStorage::delete(STATE_STORAGE_KEY);
                                                use_authorization().set(AccessToken::default());
                                                use_navigate()("/login", Default::default());
                                            }
                                        >

                                            <IconPower/>
                                            Logout
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>

                    </div>
                </div>
            </nav>
        </header>
    }
}
