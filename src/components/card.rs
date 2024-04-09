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
