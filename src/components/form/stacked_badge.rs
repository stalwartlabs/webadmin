use leptos::*;
use web_sys::wasm_bindgen::JsCast;

use crate::components::{badge::Badge, icon::IconXMark, Color};

use super::ValidateCb;

#[component]
pub fn StackedBadge(
    #[prop(into)] add_button_text: String,
    color: Color,
    values: RwSignal<Vec<String>>,
    #[prop(into, optional)] validate_item: Option<Callback<(String, ValidateCb), ()>>,
) -> impl IntoView {
    let show_tooltip = create_rw_signal(false);
    let add_value = create_rw_signal(String::new());
    let validation_error = create_rw_signal(None::<String>);

    let validate_value = move || {
        let value = add_value.get().trim().to_string();
        if let Some(cb) = validate_item.as_ref() {
            cb.call((
                value,
                (move |result| match result {
                    Ok(value) => {
                        values.update(|v| {
                            v.push(value);
                        });
                        show_tooltip.set(false);
                    }
                    Err(error) => {
                        validation_error.set(Some(error));
                    }
                })
                .into(),
            ));
        } else {
            values.update(|v| {
                v.push(value);
            });
            show_tooltip.set(false);
        }
    };

    view! {
        <div class="relative">

            <For
                each=move || { values.get() }
                key=move |item| item.clone()
                children=move |item| {
                    let label = item.clone();
                    view! {
                        <div class="inline-flex flex-wrap gap-2 p-1">

                            <Badge color=color large=true>
                                {label}
                                <button
                                    type="button"
                                    class="flex-shrink-0 size-4 inline-flex items-center justify-center rounded-full hover:bg-teal-200 focus:outline-none focus:bg-teal-200 focus:text-teal-500 dark:hover:bg-teal-900"
                                    on:click=move |_| {
                                        values
                                            .update(|v| {
                                                v.remove(v.iter().position(|i| i == &item).unwrap());
                                            });
                                    }
                                >

                                    <span class="sr-only">Remove</span>
                                    <IconXMark attr:class="flex-shrink-0 size-3"/>
                                </button>
                            </Badge>

                        </div>
                    }
                }
            />

            <div class="inline-flex flex-wrap gap-2 p-1 hs-tooltip inline-block [--trigger:hover]">
                <button
                    type="button"
                    class="py-1.5 px-2 inline-flex items-center gap-x-1 text-xs font-medium rounded-full border border-dashed border-gray-200 bg-white text-gray-800 hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-gray-800 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    on:click=move |_| {
                        add_value.set(String::new());
                        validation_error.set(None);
                        show_tooltip.set(true);
                    }
                >

                    +
                    {add_button_text}

                </button>

            </div>

        </div>

        <span
            class="hs-tooltip-content hs-tooltip-shown:opacity-100 hs-tooltip-shown:visible transition-opacity inline-block absolute z-10 py-3 px-4 bg-white border text-sm text-gray-600 rounded-lg shadow-md dark:bg-gray-900 dark:border-gray-700 dark:text-gray-400"
            role="tooltip"
            class:hidden=move || !show_tooltip.get()
            class:invisible=move || !show_tooltip.get()
            class:show=move || show_tooltip.get()
        >
            <div>
                <div class="flex rounded-lg shadow-sm">
                    <input
                        type="text"
                        class=move || {
                            if validation_error.get().is_none() {
                                "py-2 px-3 block w-full border-gray-200 shadow-sm rounded-s-md text-sm focus:z-10 focus:border-blue-500 focus:ring-blue-500 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400"
                            } else {
                                "py-2 px-3 block w-full border-red-500 shadow-sm rounded-s-md text-sm focus:z-10 focus:border-red-500 focus:ring-red-500 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400"
                            }
                        }

                        prop:value=add_value
                        on:change=move |ev| {
                            add_value.set(event_target_value(&ev));
                        }

                        on:keyup=move |ev| {
                            if ev.unchecked_ref::<web_sys::KeyboardEvent>().key_code() == 13 {
                                add_value.set(event_target_value(&ev));
                                validate_value();
                            }
                        }
                    />

                    <button
                        type="button"
                        class="-ms-px py-2 px-3 inline-flex justify-center items-center gap-x-2 text-sm font-semibold border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        on:click=move |_| {
                            validate_value();
                        }
                    >

                        Add
                    </button>

                    <button
                        type="button"
                        class="-ms-px py-2 px-3 inline-flex justify-center items-center gap-2 border font-medium bg-white text-gray-700 rounded-e-md shadow-sm align-middle hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-600 transition-all text-sm dark:bg-gray-800 dark:hover:bg-slate-800 dark:border-gray-700 dark:text-gray-400 dark:hover:text-white"
                        on:click=move |_| {
                            show_tooltip.set(false);
                        }
                    >

                        Cancel
                    </button>

                </div>
                <p
                    class="text-xs text-red-600 mt-2"
                    class:hidden=move || validation_error.get().is_none()
                >
                    {move || validation_error.get().unwrap_or_default()}
                </p>
            </div>
        </span>
    }
}
