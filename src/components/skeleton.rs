/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

#[component]
pub fn Skeleton() -> impl IntoView {
    view! {
        <div class="flex animate-pulse">
            <div class="ms-4 mt-2 w-full">
                <h3 class="h-4 bg-gray-200 rounded-full dark:bg-gray-700" style="width: 40%;"></h3>

                <ul class="mt-5 space-y-3">
                    <li class="w-full h-4 bg-gray-200 rounded-full dark:bg-gray-700"></li>
                    <li class="w-full h-4 bg-gray-200 rounded-full dark:bg-gray-700"></li>
                    <li class="w-full h-4 bg-gray-200 rounded-full dark:bg-gray-700"></li>
                    <li class="w-full h-4 bg-gray-200 rounded-full dark:bg-gray-700"></li>
                </ul>
            </div>
        </div>
    }
}
