use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use leptos::*;

use crate::core::schema::{Source, Type};

use super::FormElement;

#[component]
pub fn Select(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let options = create_memo(move |_| {
        let data = element.data.get_untracked();

        match &data.schema.fields.get(element.id).unwrap().typ_ {
            Type::Select(Source::Static(options)) => options
                .iter()
                .map(|(value, label)| (value.to_string(), label.to_string()))
                .collect::<Vec<_>>(),
            Type::Select(Source::Dynamic {
                schema,
                field,
                filter,
            }) => {
                let filter = filter.eval(&data);

                data.external_sources
                    .get(&format!("{}_{}", schema.id, field.id))
                    .map(|source| {
                        source
                            .iter()
                            .filter_map(|(id, value)| {
                                if filter.map_or(true, |values| values.contains(&value.as_str())) {
                                    (
                                        id.to_string(),
                                        format!("{} ({})", field.typ_.label(value), id),
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
    });
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
