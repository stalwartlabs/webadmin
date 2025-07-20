use crate::{components::icon::IconLanguages, i18n::{self, use_i18n}};
use leptos::*;
use leptos_i18n::Locale;

#[component]
pub fn LanguageSelector() -> impl IntoView {
    let i18n = use_i18n();
    let show_dropdown = create_rw_signal(false);

    view! {
        <div class="relative inline-block text-left">
            <button
                type="button"
                class="size-[38px] flex items-center justify-center rounded-full bg-white dark:bg-neutral-900 border border-transparent text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-white dark:hover:bg-neutral-700 dark:focus:bg-neutral-700 transition-colors duration-200"
                aria-haspopup="menu"
                aria-expanded=move || show_dropdown.get()
                on:click=move |_| show_dropdown.update(|v| *v = !*v)
            >
                <IconLanguages />
                <span class="sr-only">Select a language</span>
            </button>
            <div
                class=move || {
                    if show_dropdown.get() {
                        "origin-top-right absolute right-0 mt-2 w-36 rounded-lg shadow-lg bg-white/95 ring-1 ring-black/5 dark:bg-gray-800/95 dark:ring-white/10 z-50 transition-all duration-200"
                    } else {
                        "hidden"
                    }
                }
                role="menu"
            >
                <fieldset class="flex flex-col py-2">
                    {move || {
                        i18n::Locale::get_all()
                            .iter()
                            .cloned()
                            .map(|lang| {
                                view! {
                                    <label class="flex items-center gap-2 px-4 py-1.5 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors duration-150">
                                        <input
                                            type="radio"
                                            id=lang.to_string()
                                            name="language"
                                            value=lang.to_string()
                                            checked={i18n.get_locale() == lang}
                                            class="form-radio h-3.5 w-3.5 text-blue-500 border-gray-300 focus:ring-blue-400 dark:border-gray-600 dark:focus:ring-blue-300 transition-colors duration-200"
                                            on:click=move |_| {
                                                i18n.set_locale(lang);
                                            }
                                        />
                                        <span class="text-sm text-gray-700 dark:text-gray-200">
                                            {lang.to_string()}
                                        </span>
                                    </label>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </fieldset>
            </div>
        </div>
    }
}
