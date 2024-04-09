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

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use ahash::AHashSet;
use leptos::*;

use crate::core::{
    form::FormData,
    schema::{Source, Type},
};

use super::FormElement;

#[component]
pub fn Select(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let options = create_memo(move |_| element.data.get_untracked().select_sources(element.id));
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<String>(element.id)
            .unwrap_or_default()
    });
    let error = create_memo(move |_| {
        element
            .data
            .get()
            .error_string(element.id)
            .map(|s| s.to_string())
    });

    view! {
        <select
            class=move || {
                if error.get().is_none() {
                    "py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                } else {
                    "py-2 px-3 pe-9 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                }
            }

            on:change=move |ev| {
                element
                    .data
                    .update(|data| {
                        data.update(element.id, event_target_value(&ev));
                    });
            }

            disabled=move || disabled.get()
        >

            <For
                each=move || options.get()
                key=move |(id, _)| id.clone()
                children=move |(id, label)| {
                    let id_ = id.clone();
                    view! {
                        <option selected=move || value.get() == id value=id_>
                            {label}
                        </option>
                    }
                }
            />

        </select>

        {move || {
            error
                .get()
                .map(|error| {
                    view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                })
        }}
    }
}

#[component]
pub fn CheckboxGroup(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let options = create_memo(move |_| element.data.get_untracked().select_sources(element.id));
    let values = create_memo(move |_| {
        element
            .data
            .get()
            .array_value(element.id)
            .map(|s| s.to_string())
            .collect::<AHashSet<_>>()
    });
    let error = create_memo(move |_| {
        element
            .data
            .get()
            .error_string(element.id)
            .map(|s| s.to_string())
    });

    view! {
        <div class="space-y-2">

            <For
                each=move || {
                    options
                        .get()
                        .chunks(2)
                        .map(|v| [v.first().cloned(), v.get(1).cloned()])
                        .enumerate()
                        .collect::<Vec<_>>()
                }

                key=move |(idx, _)| *idx
                children=move |(_, options)| {
                    let options = options
                        .into_iter()
                        .flatten()
                        .map(|(id, label)| {
                            let id_ = id.clone();
                            view! {
                                <label class="max-w-xs flex p-3 w-full bg-white border border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400">
                                    <input
                                        type="checkbox"
                                        class="shrink-0 mt-0.5 border-gray-200 rounded text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-gray-800 dark:border-gray-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                        prop:checked=move || values.get().contains(&id_)
                                        disabled=move || disabled.get()
                                        on:input=move |_| {
                                            let mut values = values.get();
                                            if !values.remove(&id) {
                                                values.insert(id.clone());
                                            }
                                            let mut values = values.into_iter().collect::<Vec<_>>();
                                            values.sort();
                                            element
                                                .data
                                                .update(|data| {
                                                    data.update(element.id, values);
                                                })
                                        }
                                    />

                                    <span class="text-sm text-gray-500 ms-3 dark:text-gray-400">
                                        {label}
                                    </span>
                                </label>
                            }
                        })
                        .collect_view();
                    view! { <div class="grid sm:grid-cols-2 gap-2">{options}</div> }
                }
            />

        </div>

        {move || {
            error
                .get()
                .map(|error| {
                    view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                })
        }}
    }
}

#[component]
pub fn SelectCron(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<SimpleCron>(element.id)
            .unwrap_or_default()
    });
    let error = create_memo(move |_| {
        element
            .data
            .get()
            .error_string(element.id)
            .map(|s| s.to_string())
    });

    view! {
        <div class="space-y-3">
            <div class="relative">
                <div class="sm:flex rounded-lg shadow-sm">
                    <select
                        class=move || {
                            if error.get().is_none() {
                                "py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            } else {
                                "py-2 px-3 pe-9 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            }
                        }

                        on:change=move |ev| {
                            element
                                .data
                                .update(|data| {
                                    data.update(
                                        element.id,
                                        data
                                            .value::<SimpleCron>(element.id)
                                            .unwrap_or_default()
                                            .day(event_target_value(&ev))
                                            .to_string(),
                                    );
                                });
                        }

                        disabled=move || disabled.get()
                    >

                        <For
                            each=move || 0..=7
                            key=move |num| *num
                            children=move |num| {
                                let id = if num == 0 { "*".to_string() } else { num.to_string() };
                                let id_ = id.clone();
                                view! {
                                    <option selected=move || value.get().day == id value=id_>
                                        {match num {
                                            0 => "Every day",
                                            1 => "On Monday",
                                            2 => "On Tuesday",
                                            3 => "On Wednesday",
                                            4 => "On Thursday",
                                            5 => "On Friday",
                                            6 => "On Saturday",
                                            7 => "On Sunday",
                                            _ => unreachable!(),
                                        }}

                                    </option>
                                }
                            }
                        />

                    </select>
                    <select
                        class=move || {
                            if error.get().is_none() {
                                "py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            } else {
                                "py-2 px-3 pe-9 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            }
                        }

                        on:change=move |ev| {
                            element
                                .data
                                .update(|data| {
                                    data.update(
                                        element.id,
                                        data
                                            .value::<SimpleCron>(element.id)
                                            .unwrap_or_default()
                                            .hour(event_target_value(&ev))
                                            .to_string(),
                                    );
                                });
                        }

                        disabled=move || disabled.get()
                    >

                        <For
                            each=move || 0..=24
                            key=move |num| *num
                            children=move |num| {
                                let id = if num == 0 {
                                    "*".to_string()
                                } else {
                                    (num - 1).to_string()
                                };
                                let id_ = id.clone();
                                view! {
                                    <option selected=move || value.get().hour == id value=id_>
                                        {if num == 0 {
                                            "Every hour".to_string()
                                        } else {
                                            format!("at hour {}", num - 1)
                                        }}

                                    </option>
                                }
                            }
                        />

                    </select>
                    <select
                        class=move || {
                            if error.get().is_none() {
                                "py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            } else {
                                "py-2 px-3 pe-9 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            }
                        }

                        on:change=move |ev| {
                            element
                                .data
                                .update(|data| {
                                    data.update(
                                        element.id,
                                        data
                                            .value::<SimpleCron>(element.id)
                                            .unwrap_or_default()
                                            .minute(event_target_value(&ev))
                                            .to_string(),
                                    );
                                });
                        }

                        disabled=move || disabled.get()
                    >

                        <For
                            each=move || 0..=59
                            key=move |num| *num
                            children=move |num| {
                                view! {
                                    <option
                                        selected=move || value.get().minute == num.to_string()
                                        value=num
                                    >
                                        {format!("at minute {num}")}

                                    </option>
                                }
                            }
                        />

                    </select>
                </div>
            </div>
        </div>

        {move || {
            error
                .get()
                .map(|error| {
                    view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                })
        }}
    }
}

impl FormData {
    fn select_sources(&self, id: &str) -> Vec<(String, String)> {
        match &self.schema.fields.get(id).unwrap().typ_ {
            Type::Select {
                source: Source::Static(options),
                ..
            } => options
                .iter()
                .map(|(value, label)| (value.to_string(), label.to_string()))
                .collect::<Vec<_>>(),
            Type::Select {
                source:
                    Source::Dynamic {
                        schema,
                        field,
                        filter,
                    },
                ..
            } => {
                let filter = filter.eval(self);

                self.external_sources
                    .get(&format!("{}_{}", schema.id, field.id))
                    .map(|source| {
                        source
                            .iter()
                            .filter_map(|(id, value)| {
                                if filter.map_or(true, |values| values.contains(&value.as_str())) {
                                    (
                                        id.to_string(),
                                        if !value.is_empty() {
                                            format!("{} ({})", field.typ_.label(value), id)
                                        } else {
                                            id.to_string()
                                        },
                                    )
                                        .into()
                                } else {
                                    None
                                }
                            })
                            .chain([(String::new(), "-- None --".to_string())])
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            }
            _ => panic!("Invalid schema type for select"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleCron {
    hour: String,
    minute: String,
    day: String,
}

impl FromStr for SimpleCron {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        Ok(SimpleCron {
            minute: parts.next().unwrap_or("0").to_string(),
            hour: parts.next().unwrap_or("3").to_string(),
            day: parts.next().unwrap_or("*").to_string(),
        })
    }
}

impl SimpleCron {
    fn hour(mut self, hour: String) -> Self {
        self.hour = hour;
        self
    }

    fn minute(mut self, minute: String) -> Self {
        self.minute = minute;
        self
    }

    fn day(mut self, day: String) -> Self {
        self.day = day;
        self
    }
}

impl Default for SimpleCron {
    fn default() -> Self {
        SimpleCron {
            hour: "3".to_string(),
            minute: "0".to_string(),
            day: "*".to_string(),
        }
    }
}

impl Display for SimpleCron {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.minute, self.hour, self.day)
    }
}
