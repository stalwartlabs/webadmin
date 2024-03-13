use leptos::*;

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
                <svg
                    class="flex-shrink-0 size-4 text-red-500"
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" x2="12" y1="8" y2="12"></line>
                    <line x1="12" x2="12.01" y1="16" y2="16"></line>
                </svg>
            </div>
        </div>
        <p
            class="text-xs text-red-600 mt-2"
            id="hs-validation-name-error-helper"
            class:hidden=move || error.get().is_none()
        >
            {move || { error.get().unwrap_or_default() }}
        </p>
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

        <p
            class="text-xs text-red-600 mt-2"
            id="hs-validation-name-error-helper"
            class:hidden=move || error.get().is_none()
        >
            {move || { error.get().unwrap_or_default() }}
        </p>
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

    view! {
        <div class="relative">
            <input
                type="text"
                class="py-2 px-3 ps-9 pe-20 block w-full border-gray-200 shadow-sm rounded-lg text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
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
                            data.update(
                                element.id,
                                (event_target_value(&ev).parse::<u64>().unwrap_or(0)
                                    * multiplier.get())
                                    .to_string(),
                            );
                        });
                }

                {..attrs}
                disabled=move || disabled.get()
            />

            <div class="absolute inset-y-0 end-0 flex items-center text-gray-500 pe-px">
                <select
                    class="block text-xs w-full border-transparent rounded-lg focus:ring-blue-600 focus:border-blue-600 dark:bg-gray-800"
                    on:change=move |ev| {
                        element
                            .data
                            .update(|data| {
                                data.update(
                                    element.id,
                                    match event_target_value(&ev).parse::<u64>().unwrap_or(0) {
                                        0 => "0".to_string(),
                                        new_multiplier => {
                                            (std::cmp::max(display_value.get(), 1) * new_multiplier)
                                                .to_string()
                                        }
                                    },
                                );
                            });
                    }
                >

                    <option selected=move || multiplier.get() == 0 value="0">
                        Unlimited
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
    }
}

#[component]
pub fn InputSwitch(
    element: FormElement,
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
        <div class="relative">
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
        </div>
    }
}
