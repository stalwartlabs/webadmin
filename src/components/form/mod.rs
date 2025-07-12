/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod button;
pub mod expression;
pub mod input;
pub mod select;
pub mod stacked_badge;
pub mod stacked_duration;
pub mod stacked_input;
pub mod tab;

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
    #[prop(optional)] stacked: bool,
    children: Children,
) -> impl IntoView {
    let title = title.filter(|s| !s.is_empty()).map(|title| {
        view! {
            <div class="sm:col-span-12">
                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">{title}</h2>
            </div>
        }
    });
    let class = if stacked {
        "mt-5 pt-2 relative z-10 bg-white rounded-xl sm:mt-5 md:pt-5"
    } else if title.is_some() {
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
    #[prop(optional)] stacked: bool,
    children: Children,
) -> impl IntoView {
    let tooltip = tooltip
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
    });

    let is_optional = move || {
        if is_optional.get() {
            Some(
                view! { <span class="text-sm text-gray-400 dark:text-gray-600">{" (Optional)"}</span> },
            )
        } else {
            None
        }
    };

    if !stacked {
        view! {
            <div class="sm:col-span-3" class:hidden=move || hide.get()>
                <label class="inline-block text-sm text-gray-800 mt-2.5 dark:text-gray-200">
                    {label}
                </label>
                {tooltip}
                {is_optional}

            </div>
            <div class="sm:col-span-9" class:hidden=move || hide.get()>
                {children()}
            </div>
        }
        .into_view()
    } else {
        view! {
            <div class="mb-4 sm:mb-8" class:hidden=move || hide.get()>
                <label class="block mb-2 text-sm font-medium dark:text-white">{label}</label>
                <div class="relative">{children()}</div>
            </div>
        }
        .into_view()
    }
}

impl FormElement {
    pub fn new(id: &'static str, data: RwSignal<FormData>) -> Self {
        FormElement { id, data }
    }
}
