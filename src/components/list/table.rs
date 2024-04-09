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
pub fn Table(#[prop(into)] headers: MaybeSignal<Vec<String>>, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-white">
            <div class="-m-1.5 overflow-x-auto">
                <div class="p-1.5 min-w-full inline-block align-middle">
                    <div class="border rounded-lg shadow overflow-hidden dark:border-gray-700 dark:shadow-gray-900">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            <thead>
                                <tr>
                                    <For
                                        each=move || {
                                            headers.get().into_iter().collect::<Vec<_>>()
                                        }

                                        key=|header| header.clone()
                                        children=move |header| {
                                            view! {
                                                <th
                                                    scope="col"
                                                    class="px-6 py-3 text-start text-xs font-medium text-gray-500 uppercase"
                                                >
                                                    {header}
                                                </th>
                                            }
                                        }
                                    />

                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                {children()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TableRow(children: Children) -> impl IntoView {
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            view! {
                <td class="px-6 py-2 whitespace-nowrap text-sm text-gray-800 dark:text-gray-200">
                    {child}
                </td>
            }
        })
        .collect_view();

    view! { <tr>{children}</tr> }
}
