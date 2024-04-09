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

#[component]
pub fn ReportView(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(into, optional)] hide: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class="max-w-[85rem] px-4 py-5 sm:px-6 lg:px-8 lg:py-7 mx-auto"
            class:hidden=move || hide.get()
        >
            <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">

                {children()}

            </div>
        </div>
    }
}

#[component]
pub fn ReportSection(
    #[prop(into)] title: String,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class="grid sm:grid-cols-12 gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent"
        >

            <div class="sm:col-span-12">
                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">{title}</h2>
            </div>

            {children()}
        </div>
    }
}

#[component]
pub fn ReportItem(
    #[prop(into)] label: String,
    #[prop(optional)] hide: bool,
    children: Children,
) -> impl IntoView {
    if !hide {
        Some(view! {
            <div class="sm:col-span-3">
                <label class="inline-block text-sm font-medium text-gray-500 mt-2.5">{label}</label>
            </div>

            <div class="sm:col-span-9">{children()}</div>
        }.into_view())
    } else {
        None
    }
}

#[component]
pub fn ReportTextValue(#[prop(into)] value: MaybeSignal<String>) -> impl IntoView {
    view! {
        <label class="inline-block text-sm font-semibold text-gray-500 mt-2.5">
            {move || value.get()}
        </label>
    }
}
