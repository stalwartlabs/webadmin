pub mod button;
pub mod input;
pub mod select;
pub mod stacked_badge;
pub mod stacked_input;

use leptos::*;

use crate::{components::messages::alert::Alerts, core::form::FormData};

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
    #[prop(optional, into)] title: Option<String>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(optional, into)] hide: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let title = title.map(|title| {
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
        <div {..attrs} class=class class:hide=move || hide.get()>

            {title}

            {children()}
        </div>
    }
}

#[component]
pub fn FormItem(
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(optional, into)] hide: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="sm:col-span-3" class:hide=move || hide.get()>
            <label class="inline-block text-sm text-gray-800 mt-2.5 dark:text-gray-200">
                {label}
            </label>
        </div>
        <div class="sm:col-span-9" class:hide=move || hide.get()>
            {children()}
        </div>
    }
}

impl FormElement {
    pub fn new(id: &'static str, data: RwSignal<FormData>) -> Self {
        FormElement { id, data }
    }
}
