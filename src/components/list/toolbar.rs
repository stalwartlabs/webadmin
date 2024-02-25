use leptos::*;
use web_sys::wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonIcon {
    Add,
    Delete,
}

#[component]
pub fn ToolbarButton(
    #[prop(into)] text: MaybeSignal<String>,
    icon: ButtonIcon,
    #[prop(into)] on_click: Callback<(), ()>,
) -> impl IntoView {
    let class = match icon {
        ButtonIcon::Add => concat!("py-2 px-3 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg ","border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 ","disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"),
        ButtonIcon::Delete => concat!("py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg ","border border-gray-200 bg-white text-red-500 shadow-sm hover:bg-gray-50 disabled:opacity-50 ","disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:hover:bg-gray-800 ","dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"),
    };

    view! {
        <button class=class on:click=move |_| on_click.call(())>

            {match icon {
                ButtonIcon::Add => {
                    view! {
                        <svg
                            class="flex-shrink-0 size-3"
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 16 16"
                            fill="none"
                        >
                            <path
                                d="M2.63452 7.50001L13.6345 7.5M8.13452 13V2"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                            ></path>
                        </svg>
                    }
                        .into_view()
                }
                ButtonIcon::Delete => {
                    view! {
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
                            <path d="M3 6h18"></path>
                            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                            <line x1="10" x2="10" y1="11" y2="17"></line>
                            <line x1="14" x2="14" y1="11" y2="17"></line>
                        </svg>
                    }
                        .into_view()
                }
            }}

            {move || text.get()}
        </button>
    }
}

#[component]
pub fn SearchBox(
    #[prop(into)] value: MaybeSignal<Option<String>>,
    #[prop(into)] on_search: Callback<String, ()>,
) -> impl IntoView {
    let value_ = value.clone();
    view! {
        <div class="sm:col-span-1">
            <label for="hs-as-table-product-review-search" class="sr-only">
                Search
            </label>
            <div class="relative">
                <input
                    type="text"
                    class=concat!(
                        "py-2 px-3 ps-11 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 ",
                        "disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 ",
                        "dark:text-gray-400 dark:focus:ring-gray-600",
                    )

                    placeholder="Search"
                    prop:value=move || value_.get().unwrap_or_default()
                    on:keyup=move |ev| {
                        let key_code = ev.unchecked_ref::<web_sys::KeyboardEvent>().key_code();
                        if key_code == 13 {
                            let filter = event_target_value(&ev);
                            if filter != value.get().unwrap_or_default() {
                                on_search.call(filter);
                            }
                        }
                    }
                />

                <div class="absolute inset-y-0 start-0 flex items-center pointer-events-none ps-4">
                    <svg
                        class="size-4 text-gray-400"
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        fill="currentColor"
                        viewBox="0 0 16 16"
                    >
                        <path d=concat!(
                            "M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.",
                            "115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 ",
                            "1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 ",
                            "0 5.5 5.5 0 0 1 11 0z",
                        )></path>
                    </svg>
                </div>
            </div>
        </div>
    }
}
