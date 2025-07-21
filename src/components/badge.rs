/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
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
