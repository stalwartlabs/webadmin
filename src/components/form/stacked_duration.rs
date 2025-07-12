/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::FormElement;
use crate::components::form::input::Duration;
use crate::{
    components::icon::{IconPlus, IconXMark},
    core::form::FormErrorType,
};
use leptos::*;
use std::str::FromStr;

#[component]
pub fn StackedDuration(add_button_text: String, element: FormElement) -> impl IntoView {
    // TODO: Abstract all stacked components
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
                    let value_ = Duration::from_str(&item).unwrap_or_default();
                    let mut value = value_.clone();
                    let contents = value_.value.clone();
                    let unit = value_.unit.clone();
                    view! {
                        <div class="space-y-3">
                            <div class="relative">

                                <input
                                    type="text"
                                    class=move || {
                                        if !is_err {
                                            "py-2 px-3 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        } else {
                                            "py-2 px-3 block w-full border-red-500 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        }
                                    }

                                    prop:value=contents

                                    on:change=move |ev| {
                                        let value = value_.clone();
                                        element
                                            .data
                                            .update(|data| {
                                                match event_target_value(&ev).trim().parse::<u64>() {
                                                    Ok(new_value) if new_value > 0 => {
                                                        data.array_update(
                                                            element.id,
                                                            idx,
                                                            value.value(new_value).to_string(),
                                                        );
                                                    }
                                                    _ => {
                                                        data.new_error(element.id, "Invalid duration".to_string());
                                                    }
                                                }
                                            });
                                    }
                                />

                                <div class="absolute inset-y-0 end-0 flex items-center text-gray-500 pe-px">
                                    <select
                                        class="block text-xs w-full border-transparent rounded-lg focus:ring-blue-600 focus:border-blue-600 dark:bg-gray-800"
                                        on:change=move |ev| {
                                            element
                                                .data
                                                .update(|data| {
                                                    let unit = event_target_value(&ev);
                                                    let new_value = if !unit.is_empty() {
                                                        if value.value.is_empty() {
                                                            value.value = "1".to_string();
                                                        }
                                                        value.unit = unit;
                                                        value.to_string()
                                                    } else {
                                                        "false".to_string()
                                                    };
                                                    data.array_update(element.id, idx, new_value);
                                                });
                                        }
                                    >

                                        <option selected=unit.is_empty() value="">
                                            None
                                        </option>
                                        <option selected=unit == "ms" value="ms">
                                            ms
                                        </option>
                                        <option selected=unit == "s" value="s">
                                            seconds
                                        </option>
                                        <option selected=unit == "m" value="m">
                                            minutes
                                        </option>
                                        <option selected=unit == "h" value="h">
                                            hours
                                        </option>
                                        <option selected=unit == "d" value="d">
                                            days
                                        </option>
                                    </select>
                                </div>

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
                    if values.get().last().is_none_or(|(_, v, _)| !v.is_empty()) {
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
