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
