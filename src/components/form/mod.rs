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

pub mod button;
pub mod expression;
pub mod input;
pub mod select;
pub mod stacked_badge;
pub mod stacked_input;

use leptos::*;

use crate::{
    components::{icon::IconInfo, messages::alert::Alerts},
    core::form::FormData,
};

#[derive(Debug, Clone, Copy)]
pub struct FormElement {
    pub id: &'static str,
    pub data: RwSignal<FormData>,
}

pub type ValidateCb = Callback<Result<String, String>, ()>;

#[component]
pub fn Form(
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] subtitle: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    let title_ = title.clone();

    view! {
        <div class="max-w-4xl px-4 py-10 sm:px-6 lg:px-8 lg:py-14 mx-auto">
            <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">
                <div class="mb-8" class:hidden=move || title_.get().is_empty()>
                    <h2 class="text-xl font-bold text-gray-800 dark:text-gray-200">
                        {move || title.get()}
                    </h2>
                    <p class="text-sm text-gray-600 dark:text-gray-400">{move || subtitle.get()}</p>
                </div>

                <Alerts/>

                <form>{children()}</form>
            </div>
        </div>
    }
}

#[component]
pub fn FormButtonBar(children: Children) -> impl IntoView {
    view! { <div class="mt-5 flex justify-end gap-x-2">{children()}</div> }
}

#[component]
pub fn FormSection(
    #[prop(optional)] title: Option<String>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(optional, into)] hide: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let title = title.filter(|s| !s.is_empty()).map(|title| {
        view! {
            <div class="sm:col-span-12">
                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">{title}</h2>
            </div>
        }
    });
    let class = if title.is_some() {
        "grid sm:grid-cols-12 gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent"
    } else {
        "grid sm:grid-cols-12 gap-2 sm:gap-6"
    };

    view! {
        <div {..attrs} class=class class:hidden=move || hide.get()>

            {title}

            {children()}
        </div>
    }
}

#[component]
pub fn FormItem(
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(optional)] tooltip: Option<&'static str>,
    #[prop(optional, into)] hide: MaybeSignal<bool>,
    #[prop(optional, into)] is_optional: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="sm:col-span-3" class:hidden=move || hide.get()>
            <label class="inline-block text-sm text-gray-800 mt-2.5 dark:text-gray-200">
                {label}
            </label>
            {tooltip
                .filter(|s| !s.is_empty())
                .map(|tooltip| {
                    let is_mouse_over = create_rw_signal(false);
                    view! {
                        <div class="hs-tooltip inline-block">
                            <button
                                type="button"
                                class="hs-tooltip-toggle ms-1"
                                on:mouseover=move |_| {
                                    is_mouse_over.set(true);
                                }

                                on:mouseleave=move |_| {
                                    is_mouse_over.set(false);
                                }
                            >

                                <IconInfo
                                    size=16
                                    attr:stroke-width="1"
                                    attr:class="inline-block size-3 text-gray-400 dark:text-gray-600"
                                />
                            </button>
                            <span
                                class="hs-tooltip-content hs-tooltip-shown:opacity-100 hs-tooltip-shown:visible opacity-70 transition-opacity inline-block absolute w-40 text-center z-10 py-1 px-2 bg-gray-900 text-xs font-medium text-white rounded shadow-sm dark:bg-slate-700"
                                role="tooltip"
                                class:hidden=move || !is_mouse_over.get()
                                class:show=move || is_mouse_over.get()
                            >
                                {tooltip}
                            </span>

                        </div>
                    }
                })}

            {move || {
                if is_optional.get() {
                    Some(
                        view! {
                            <span class="text-sm text-gray-400 dark:text-gray-600">
                                {" (Optional)"}
                            </span>
                        },
                    )
                } else {
                    None
                }
            }}

        </div>
        <div class="sm:col-span-9" class:hidden=move || hide.get()>
            {children()}
        </div>
    }
}

impl FormElement {
    pub fn new(id: &'static str, data: RwSignal<FormData>) -> Self {
        FormElement { id, data }
    }
}
