/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

use crate::components::Color;

#[component]
pub fn Button(
    #[prop(into)] text: MaybeSignal<String>,
    #[prop(into)] color: MaybeSignal<Color>,
    #[prop(into)] on_click: Callback<(), ()>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=move || {
                match color.get() {
                    Color::Blue => {
                        "py-2 px-3 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    }
                    Color::Gray => {
                        "py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    }
                    Color::Red => {
                        "py-2 px-3 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-red-600 text-white hover:bg-red-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    }
                    _ => unimplemented!(),
                }
            }

            disabled=move || disabled.get()
            on:click=move |_| on_click.call(())
            {..attrs}
        >
            {children.map(|children| children())}
            {text.get()}
        </button>
    }
}
