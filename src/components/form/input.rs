use std::str::FromStr;

use leptos::*;

use crate::components::icon::{IconClock, IconExclamationCircle, IconInfo};

use super::FormElement;

#[component]
pub fn InputText(
    element: FormElement,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
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
        <div class="relative">
            <input
                {..attrs}
                type="text"
                class=move || {
                    if error.get().is_none() {
                        "py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm text-sm rounded-lg focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    } else {
                        "py-2 px-3 pe-11 block w-full border-red-500 shadow-sm text-sm rounded-lg focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    }
                }

                placeholder=placeholder.map(|p| move || p.get())
                prop:value=move || value.get()
                disabled=move || disabled.get()
                on:change=move |ev| {
                    element
                        .data
                        .update(|data| {
                            data.update(element.id, event_target_value(&ev));
                        });
                }
            />

            <div
                class="absolute inset-y-0 end-0 flex items-center pointer-events-none pe-3"
                class:hidden=move || error.get().is_none()
            >
                <IconExclamationCircle attr:class="flex-shrink-0 size-4 text-red-500"/>
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

#[component]
pub fn InputPassword(
    element: FormElement,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
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
    let show_password = create_rw_signal(false);

    view! {
        <div class="relative">
            <input
                {..attrs}
                type=move || if show_password.get() { "text" } else { "password" }
                class=move || {
                    if error.get().is_none() {
                        "py-2 px-3 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    } else {
                        "py-2 px-3 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    }
                }

                placeholder=placeholder.map(|p| move || p.get())
                prop:value=move || value.get()
                disabled=move || disabled.get()
                on:change=move |ev| {
                    element
                        .data
                        .update(|data| {
                            data.update(element.id, event_target_value(&ev));
                        });
                }
            />

            <button
                type="button"
                class="absolute top-0 end-0 p-3.5 rounded-e-md dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                on:click=move |_| {
                    show_password.update(|v| *v = !*v);
                }
            >

                <svg
                    class=move || {
                        let color = if error.get().is_none() { "gray-400" } else { "red-500" };
                        format!("flex-shrink-0 size-3.5 text-{color} dark:text-neutral-600")
                    }

                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path
                        class="hs-password-active:hidden"
                        class:hidden=move || show_password.get()
                        d="M9.88 9.88a3 3 0 1 0 4.24 4.24"
                    ></path>
                    <path
                        class="hs-password-active:hidden"
                        class:hidden=move || show_password.get()
                        d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"
                    ></path>
                    <path
                        class="hs-password-active:hidden"
                        class:hidden=move || show_password.get()
                        d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"
                    ></path>
                    <line
                        class="hs-password-active:hidden"
                        class:hidden=move || show_password.get()
                        x1="2"
                        x2="22"
                        y1="2"
                        y2="22"
                    ></line>
                    <path
                        class:hidden=move || !show_password.get()
                        class="hs-password-active:block"
                        d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"
                    ></path>
                    <circle
                        class="hs-password-active:block"
                        class:hidden=move || !show_password.get()
                        cx="12"
                        cy="12"
                        r="3"
                    ></circle>
                </svg>
            </button>
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

const UNIT_GB: u64 = 1024 * 1024 * 1024;
const UNIT_MB: u64 = 1024 * 1024;

#[component]
pub fn InputSize(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<u64>(element.id)
            .unwrap_or_default()
    });

    let multiplier = create_memo(move |_| {
        let raw_value = value.get();
        if raw_value == 0 {
            0
        } else if raw_value % UNIT_GB == 0 {
            UNIT_GB
        } else if raw_value % UNIT_MB == 0 {
            UNIT_MB
        } else {
            1
        }
    });
    let display_value = create_memo(move |_| {
        let multiplier = multiplier.get();
        if multiplier != 0 {
            value.get() / multiplier
        } else {
            0u64
        }
    });
    let error = create_memo(move |_| {
        element
            .data
            .get()
            .error_string(element.id)
            .map(|s| s.to_string())
    });

    view! {
        <div class="relative">
            <input
                type="text"
                class="py-2 px-3 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                prop:value=move || {
                    match display_value.get() {
                        0 => String::new(),
                        v => v.to_string(),
                    }
                }

                on:change=move |ev| {
                    element
                        .data
                        .update(|data| {
                            match event_target_value(&ev).trim().parse::<u64>() {
                                Ok(new_value) if new_value > 0 => {
                                    data.update(
                                        element.id,
                                        (new_value * multiplier.get()).to_string(),
                                    );
                                }
                                _ => {
                                    data.new_error(element.id, "Invalid size".to_string());
                                }
                            }
                        });
                }

                {..attrs}
                disabled=move || { disabled.get() || display_value.get() == 0 }
            />

            <div class="absolute inset-y-0 end-0 flex items-center text-gray-500 pe-px">
                <select
                    class="block text-xs w-full border-transparent rounded-lg focus:ring-blue-600 focus:border-blue-600 dark:bg-gray-800"
                    on:change=move |ev| {
                        element
                            .data
                            .update(|data| {
                                match event_target_value(&ev).parse::<u64>().unwrap_or(0) {
                                    0 => {
                                        data.remove(element.id);
                                    }
                                    new_multiplier => {
                                        data.update(
                                            element.id,
                                            (std::cmp::max(display_value.get(), 1) * new_multiplier)
                                                .to_string(),
                                        );
                                    }
                                }
                            });
                    }
                >

                    <option selected=move || multiplier.get() == 0 value="0">
                        None
                    </option>
                    <option selected=move || multiplier.get() == 1 value="1">
                        bytes
                    </option>
                    <option selected=move || multiplier.get() == UNIT_MB value=UNIT_MB.to_string()>
                        MB
                    </option>
                    <option selected=move || multiplier.get() == UNIT_GB value=UNIT_GB.to_string()>
                        GB
                    </option>
                </select>
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

#[component]
pub fn InputDuration(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<Duration>(element.id)
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
        <div class="relative">
            <input
                type="text"
                class="py-2 px-3 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                prop:value=move || { value.get().value }

                on:change=move |ev| {
                    element
                        .data
                        .update(|data| {
                            match event_target_value(&ev).trim().parse::<u64>() {
                                Ok(new_value) if new_value > 0 => {
                                    data.update(
                                        element.id,
                                        value.get().value(new_value).to_string(),
                                    );
                                }
                                _ => {
                                    data.new_error(element.id, "Invalid duration".to_string());
                                }
                            }
                        });
                }

                {..attrs}
                disabled=move || { disabled.get() || value.get().unit.is_empty() }
            />

            <div class="absolute inset-y-0 end-0 flex items-center text-gray-500 pe-px">
                <select
                    class="block text-xs w-full border-transparent rounded-lg focus:ring-blue-600 focus:border-blue-600 dark:bg-gray-800"
                    on:change=move |ev| {
                        element
                            .data
                            .update(|data| {
                                let unit = event_target_value(&ev);
                                if !unit.is_empty() {
                                    let mut value = value.get();
                                    if value.value.is_empty() {
                                        value.value = "1".to_string();
                                    }
                                    value.unit = unit;
                                    data.update(element.id, value.to_string());
                                } else {
                                    data.remove(element.id);
                                }
                            });
                    }

                    disabled=move || disabled.get()
                >

                    <option selected=move || value.get().unit.is_empty() value="">
                        None
                    </option>
                    <option selected=move || value.get().unit == "ms" value="ms">
                        ms
                    </option>
                    <option selected=move || value.get().unit == "s" value="s">
                        seconds
                    </option>
                    <option selected=move || value.get().unit == "m" value="m">
                        minutes
                    </option>
                    <option selected=move || value.get().unit == "h" value="h">
                        hours
                    </option>
                    <option selected=move || value.get().unit == "d" value="d">
                        days
                    </option>
                </select>
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

#[component]
pub fn InputRate(
    element: FormElement,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<Rate>(element.id)
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
        <div class="relative">
            <div class="sm:flex rounded-lg shadow-sm">
                <input
                    type="text"
                    class="py-2 px-3 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    prop:value=move || { value.get().amount }
                    class:hidden=move || { disabled.get() || value.get().period.unit.is_empty() }
                    on:change=move |ev| {
                        element
                            .data
                            .update(|data| {
                                match event_target_value(&ev).trim().parse::<u64>() {
                                    Ok(new_value) if new_value > 0 => {
                                        data.update(
                                            element.id,
                                            value.get().amount(new_value).to_string(),
                                        );
                                    }
                                    _ => {
                                        data.new_error(
                                            element.id,
                                            "Invalid rate amount".to_string(),
                                        );
                                    }
                                }
                            });
                    }
                />

                <span
                    class="py-2 px-3 inline-flex items-center min-w-fit w-full border border-gray-200 bg-gray-50 text-sm text-gray-500 -mt-px -ms-px first:rounded-t-lg last:rounded-b-lg sm:w-auto sm:first:rounded-s-lg sm:mt-0 sm:first:ms-0 sm:first:rounded-se-none sm:last:rounded-es-none sm:last:rounded-e-lg dark:bg-gray-700 dark:border-gray-700 dark:text-gray-400"
                    class:hidden=move || { disabled.get() || value.get().period.unit.is_empty() }
                >

                    <IconClock attr:class="mx-auto size-4 text-gray-400"/>
                </span>
                <input
                    type="text"
                    class="py-2 px-3 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    prop:value=move || { value.get().period.value }

                    on:change=move |ev| {
                        element
                            .data
                            .update(|data| {
                                match event_target_value(&ev).trim().parse::<u64>() {
                                    Ok(new_value) if new_value > 0 => {
                                        data.update(
                                            element.id,
                                            value.get().duration_value(new_value).to_string(),
                                        );
                                    }
                                    _ => {
                                        data.new_error(element.id, "Invalid duration".to_string());
                                    }
                                }
                            });
                    }

                    {..attrs}
                    disabled=move || { disabled.get() || value.get().period.unit.is_empty() }
                />

                <div class="absolute inset-y-0 end-0 flex items-center text-gray-500 pe-px">
                    <select
                        class="block text-xs w-full border-transparent rounded-lg focus:ring-blue-600 focus:border-blue-600 dark:bg-gray-800"
                        on:change=move |ev| {
                            element
                                .data
                                .update(|data| {
                                    let unit = event_target_value(&ev);
                                    if !unit.is_empty() {
                                        let mut value = value.get();
                                        if value.period.value.is_empty() {
                                            value.period.value = "1".to_string();
                                        }
                                        if value.amount.is_empty() {
                                            value.amount = "1".to_string();
                                        }
                                        value.period.unit = unit;
                                        data.update(element.id, value.to_string());
                                    } else {
                                        data.remove(element.id);
                                    }
                                });
                        }

                        disabled=move || disabled.get()
                    >

                        <option selected=move || value.get().period.unit.is_empty() value="">
                            None
                        </option>
                        <option selected=move || value.get().period.unit == "ms" value="ms">
                            ms
                        </option>
                        <option selected=move || value.get().period.unit == "s" value="s">
                            seconds
                        </option>
                        <option selected=move || value.get().period.unit == "m" value="m">
                            minutes
                        </option>
                        <option selected=move || value.get().period.unit == "h" value="h">
                            hours
                        </option>
                        <option selected=move || value.get().period.unit == "d" value="d">
                            days
                        </option>
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

#[component]
pub fn InputSwitch(
    element: FormElement,
    #[prop(optional, into)] label: Option<MaybeSignal<String>>,
    #[prop(optional)] tooltip: Option<&'static str>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<bool>(element.id)
            .unwrap_or_default()
    });

    view! {
        <div class="flex items-center">
            <input
                type="checkbox"
                {..attrs}
                class="relative w-11 h-6 p-px bg-gray-100 border-transparent text-transparent rounded-full cursor-pointer transition-colors ease-in-out duration-200 focus:ring-blue-600 disabled:opacity-50 disabled:pointer-events-none checked:bg-none checked:text-blue-600 checked:border-blue-600 focus:checked:border-blue-600 dark:bg-gray-800 dark:border-gray-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-600 before:inline-block before:size-5 before:bg-white checked:before:bg-blue-200 before:translate-x-0 checked:before:translate-x-full before:rounded-full before:shadow before:transform before:ring-0 before:transition before:ease-in-out before:duration-200 dark:before:bg-gray-400 dark:checked:before:bg-blue-200"
                prop:checked=move || value.get()
                on:input=move |_| {
                    element
                        .data
                        .update(|data| {
                            let value = !data.value::<bool>(element.id).unwrap_or_default();
                            data.update(element.id, if value { "true" } else { "false" });
                        })
                }

                disabled=move || disabled.get()
            />
            {label
                .map(|label| {
                    view! {
                        <label class="text-sm text-gray-500 ms-3 dark:text-gray-400">
                            {label.get()}
                        </label>
                    }
                })}

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

        </div>
    }
}

#[component]
pub fn TextArea(
    element: FormElement,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
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
        <div class="relative">
            <textarea
                {..attrs}
                class=move || {
                    if error.get().is_none() {
                        "py-3 px-4 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    } else {
                        "py-2 px-3 block w-full border-red-500 rounded-lg text-sm focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    }
                }

                rows="5"
                placeholder=placeholder.map(|p| move || p.get())
                prop:value=move || value.get()
                disabled=move || disabled.get()
                on:change=move |ev| {
                    element
                        .data
                        .update(|data| {
                            data.update(element.id, event_target_value(&ev));
                        });
                }
            >
            </textarea>
            <div
                class="absolute top-0 end-0 flex items-center pointer-events-none p-3"
                class:hidden=move || error.get().is_none()
            >
                <IconExclamationCircle attr:class="flex-shrink-0 size-4 text-red-500"/>
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

#[derive(Default, PartialEq, Eq, Clone)]
pub struct Duration {
    pub value: String,
    pub unit: String,
}

impl FromStr for Duration {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut duration = Duration::default();

        for c in s.chars() {
            match c {
                '0'..='9' => duration.value.push(c),
                'a'..='z' | 'A'..='Z' => duration.unit.push(c.to_ascii_lowercase()),
                c => {
                    if !c.is_ascii_whitespace() {
                        return Err(());
                    }
                }
            }
        }

        if !duration.value.is_empty()
            && ["ms", "s", "m", "h", "d"].contains(&duration.unit.as_str())
        {
            Ok(duration)
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.value.is_empty() && !self.unit.is_empty() {
            write!(f, "{}{}", self.value, self.unit)
        } else {
            write!(f, "")
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
pub struct Rate {
    pub amount: String,
    pub period: Duration,
}

impl FromStr for Rate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (amount_, period) = s.split_once('/').ok_or(())?;
        let mut amount = String::with_capacity(amount_.len());

        for c in amount_.chars() {
            if c.is_ascii_digit() {
                amount.push(c);
            } else if !c.is_ascii_whitespace() {
                return Err(());
            }
        }

        if !amount.is_empty() {
            Ok(Rate {
                amount,
                period: period.parse()?,
            })
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.amount.is_empty() && !self.period.value.is_empty() {
            write!(f, "{}/{}", self.amount, self.period)
        } else {
            write!(f, "")
        }
    }
}

impl Duration {
    pub fn value(mut self, value: u64) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn format(&self) -> Option<String> {
        if !self.value.is_empty() && !self.unit.is_empty() {
            Some(format!(
                "{} {}{}",
                self.value,
                match self.unit.as_str() {
                    "ms" => "millisecond",
                    "s" => "second",
                    "m" => "minute",
                    "h" => "hour",
                    "d" => "day",
                    _ => "",
                },
                if self.value == "1" { "" } else { "s" }
            ))
        } else {
            None
        }
    }
}

impl Rate {
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount.to_string();
        self
    }

    pub fn duration_value(mut self, value: u64) -> Self {
        self.period.value = value.to_string();
        self
    }

    pub fn format(&self) -> Option<String> {
        if !self.amount.is_empty() && !self.period.value.is_empty() && !self.period.unit.is_empty()
        {
            Some(format!(
                "{} every {} {}{}",
                self.amount,
                self.period.value,
                match self.period.unit.as_str() {
                    "ms" => "millisecond",
                    "s" => "second",
                    "m" => "minute",
                    "h" => "hour",
                    "d" => "day",
                    _ => "",
                },
                if self.period.value == "1" { "" } else { "s" }
            ))
        } else {
            None
        }
    }
}
