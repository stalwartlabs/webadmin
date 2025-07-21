/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

use crate::{
    components::icon::{IconPlus, IconXMark},
    core::form::FormErrorType,
};

use super::FormElement;

#[component]
pub fn StackedInput(
    add_button_text: String,
    element: FormElement,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
) -> impl IntoView {
    let values = create_memo(move |_| {
        let data = element.data.get();
        let error = data.error(element.id);

        data.array_value(element.id)
            .enumerate()
            .map(|(idx, value)| {
                (
                    idx,
                    value.to_string(),
                    error.as_ref().and_then(|e| {
                        if e.id == FormErrorType::Array(idx) {
                            Some(e.error.clone())
                        } else {
                            None
                        }
                    }),
                )
            })
            .collect::<Vec<_>>()
    });
    let error = create_memo(move |_| {
        element.data.get().error(element.id).and_then(|e| {
            if e.id == FormErrorType::None {
                Some(e.error.clone())
            } else {
                None
            }
        })
    });

    view! {
        <div class="space-y-3">

            <For
                each=move || { values.get().into_iter() }
                key=move |(idx, item, error)| {
                    format!(
                        "{idx}_{}_{}",
                        item.as_bytes().iter().map(|v| *v as usize).sum::<usize>(),
                        error.is_some(),
                    )
                }

                children=move |(idx, item, error)| {
                    let is_err = error.is_some();
                    let error = error.unwrap_or_default();
                    view! {
                        <div class="space-y-3">
                            <div class="relative">
                                <input
                                    type="text"
                                    class=move || {
                                        if !is_err {
                                            "py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm text-sm rounded-lg focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        } else {
                                            "py-2 px-3 pe-11 block w-full border-red-500 shadow-sm text-sm rounded-lg focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        }
                                    }

                                    prop:value=item
                                    placeholder=placeholder.clone().map(|p| move || p.get())
                                    on:change=move |ev| {
                                        element
                                            .data
                                            .update(|data| {
                                                data.array_update(element.id, idx, event_target_value(&ev));
                                            });
                                    }
                                />

                                <button
                                    type="button"
                                    class="absolute top-0 end-0 p-2.5 rounded-e-md dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                    on:click=move |_| {
                                        element
                                            .data
                                            .update(|data| {
                                                data.array_delete(element.id, idx);
                                            });
                                    }
                                >

                                    <IconXMark/>

                                </button>
                            </div>
                            <p class="text-xs text-red-600 mt-2" class:hidden=!is_err>
                                {error}
                            </p>
                        </div>
                    }
                }
            />

        </div>

        <p class="mt-3 text-end">
            <button
                type="button"
                class="py-1.5 px-2 inline-flex items-center gap-x-1 text-xs font-medium rounded-full border border-dashed border-gray-200 bg-white text-gray-800 hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-gray-800 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                on:click=move |_| {
                    if values.get().last().is_none_or( |(_, v, _)| !v.is_empty()) {
                        element
                            .data
                            .update(|data| {
                                data.array_push(element.id, "".to_string(), false);
                            });
                    }
                }
            >

                <IconPlus attr:class="flex-shrink-0 size-3.5"/>
                {add_button_text}
            </button>
        </p>

        {move || {
            error
                .get()
                .map(|error| {
                    view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                })
        }}
    }
}
