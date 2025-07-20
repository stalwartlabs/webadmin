/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;
use leptos_i18n::t;
use leptos_meta::*;

use crate::components::icon::IconArrowLeft;
use crate::i18n::use_i18n;

#[component]
pub fn NotFound() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <Html lang="en" class="h-full" />
        <Body class="dark:bg-slate-900 bg-gray-100 flex h-full items-center py-16" />
        <div class="max-w-[50rem] flex flex-col mx-auto size-full">
            <header class="mb-auto flex justify-center z-50 w-full py-4">
                <nav class="px-4 sm:px-6 lg:px-8" aria-label="Global">
                    <a
                        class="flex-none text-xl font-semibold sm:text-3xl dark:text-white"
                        href="#"
                        aria-label=t!(i18n, brand)
                    >
                        {t!(i18n, stalwart)}
                    </a>
                </nav>
            </header>

            <div class="text-center py-10 px-4 sm:px-6 lg:px-8">
                <h1 class="block text-7xl font-bold text-gray-800 sm:text-9xl dark:text-white">
                    404
                </h1>
                <h1 class="block text-2xl font-bold text-white"></h1>
                <p class="mt-3 text-gray-600 dark:text-gray-400">
                    {t!(i18n, oops_something_went_wrong)}
                </p>
                <p class="text-gray-600 dark:text-gray-400">
                    {t!(i18n,sorry_we_could_not_find_your_page)}
                </p>
                <div class="mt-5 flex flex-col justify-center items-center gap-2 sm:flex-row sm:gap-3">
                    <a
                        class="w-full sm:w-auto py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent text-blue-600 hover:text-blue-800 disabled:opacity-50 disabled:pointer-events-none dark:text-blue-500 dark:hover:text-blue-400 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        href="/manage/directory/accounts"
                    >
                        <IconArrowLeft />
                        {t!(i18n,back_to_manage)}
                    </a>
                </div>
            </div>

            <footer class="mt-auto text-center py-5">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <p class="text-sm text-gray-500">{t!(i18n,c_stalwart_labs_ltd)}</p>
                </div>
            </footer>
        </div>
    }
}
