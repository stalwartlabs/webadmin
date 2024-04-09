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

use super::Color;

#[component]
pub fn Badge(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    color: Color,
    #[prop(optional)] large: bool,
    children: Children,
) -> impl IntoView {
    let size = if large {
        "gap-x-1.5 py-1.5 ps-3 pe-2"
    } else {
        "gap-x-1 py-1 px-2"
    };

    let color = match color {
        Color::Blue => "bg-blue-100 text-blue-800 dark:bg-blue-500/10 dark:text-blue-500",
        Color::Red => "bg-red-100 text-red-800 dark:bg-red-500/10 dark:text-red-500",
        Color::Yellow => "bg-yellow-100 text-yellow-800 dark:bg-yellow-800/30 dark:text-yellow-500",
        Color::Green => "bg-teal-100 text-teal-800 dark:bg-teal-500/10 dark:text-teal-500",
        Color::Gray => "bg-gray-100 text-gray-800 dark:bg-gray-500/10 dark:text-gray-500",
    };

    let class = format!("{size} inline-flex items-center text-xs font-medium rounded-full {color}");

    view! {
        <span {..attrs} class=class>
            {children()}
        </span>
    }
}
