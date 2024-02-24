use leptos::*;

use crate::components::form::FormValue;

use super::FormListValidator;

#[component]
pub fn StackedInput(
    add_button_text: String,
    values: FormListValidator<String>,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
) -> impl IntoView {
    let values = values.signal();

    view! {
        <div id="hs-wrapper-for-copy" class="space-y-3">

            <For
                each=move || { values.get().into_iter().enumerate().collect::<Vec<_>>() }
                key=move |(idx, item)| format!("{}_{idx}", item.hash())
                children=move |(idx, item)| {
                    let is_err = item.is_err();
                    let err = if is_err { item.clone().unwrap_err() } else { "".to_string() };
                    view! {
                        <div id="hs-wrapper-for-copy" class="space-y-3">
                            <div class="relative">
                                <input
                                    id="af-account-email"
                                    type="text"
                                    class=move || {
                                        if !is_err {
                                            "py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm text-sm rounded-lg focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        } else {
                                            "py-2 px-3 pe-11 block w-full border-red-500 shadow-sm text-sm rounded-lg focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        }
                                    }

                                    prop:value=item.unwrap()
                                    placeholder=placeholder.clone().map(|p| move || p.get())
                                    on:change=move |ev| {
                                        values
                                            .update(|v| {
                                                v[idx] = FormValue::Ok(event_target_value(&ev));
                                            });
                                    }
                                />

                                <button
                                    type="button"
                                    class="absolute top-0 end-0 p-2.5 rounded-e-md dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                    on:click=move |_| {
                                        values
                                            .update(|v| {
                                                v.remove(idx);
                                            });
                                    }
                                >

                                    <svg
                                        class="flex-shrink-0 size-4"
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
                                        <path d="M18 6 6 18"></path>
                                        <path d="m6 6 12 12"></path>
                                    </svg>
                                </button>
                            </div>
                            <p
                                class="text-xs text-red-600 mt-2"
                                id="hs-validation-name-error-helper"
                                class:hidden=!is_err
                            >
                                {err}
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
                    if values.get().last().map_or(true, |v| !v.get().is_empty()) {
                        values
                            .update(|v| {
                                v.push(FormValue::Ok("".to_string()));
                            });
                    }
                }
            >

                <svg
                    class="flex-shrink-0 size-3.5"
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
                    <path d="M5 12h14"></path>
                    <path d="M12 5v14"></path>
                </svg>
                {add_button_text}
            </button>
        </p>
    }
}
