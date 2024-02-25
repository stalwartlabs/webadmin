use leptos::*;

#[component]
pub fn Pagination(
    #[prop(into)] current_page: MaybeSignal<u32>,
    #[prop(into)] total_results: MaybeSignal<Option<u32>>,
    #[prop(into)] page_size: MaybeSignal<u32>,
    #[prop(into)] on_page_change: Callback<u32, ()>,
) -> impl IntoView {
    let total_pages = create_memo(move |_| {
        (total_results.get().unwrap_or(0) as f64 / page_size.get() as f64).ceil() as u32
    });

    view! {
        <div class="px-6 py-4 grid gap-3 md:flex md:justify-between md:items-center border-t border-gray-200 dark:border-gray-700">
            <Show when=move || { total_results.get().map_or(false, |r| r > 0) }>
                <div class="inline-flex items-center gap-x-2">

                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        <span class="font-semibold text-gray-800 dark:text-gray-200">
                            {move || { total_results.get().map_or(0, |r| r) }}
                        </span>
                        results. Page

                    </p>
                    <div class="max-w-sm space-y-3">
                        <select
                            class="py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400"
                            on:change=move |ev| {
                                on_page_change.call(event_target_value(&ev).parse().unwrap_or(1));
                            }
                        >

                            <For each=move || (1..=total_pages()) key=|page| *page let:page>
                                <option selected=move || current_page() == page>{page}</option>

                            </For>

                        </select>
                    </div>

                    <p class="text-sm text-gray-600 dark:text-gray-400">of {total_pages}</p>
                </div>

            </Show>

            <div>
                <div class="inline-flex gap-x-2">
                    <button
                        type="button"
                        class="py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        disabled=move || {
                            total_results.get().map_or(true, |r| r == 0) || current_page() <= 1
                        }

                        on:click=move |_| {
                            on_page_change.call(current_page() - 1);
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
                            <path d="m15 18-6-6 6-6"></path>
                        </svg>
                        Prev
                    </button>

                    <Suspense>
                        <button
                            type="button"
                            class="py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            on:click=move |_| {
                                on_page_change.call(current_page() + 1);
                            }

                            disabled=move || { current_page() >= total_pages.get() }
                        >

                            Next
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
                                <path d="m9 18 6-6-6-6"></path>
                            </svg>
                        </button>

                    </Suspense>

                </div>
            </div>
        </div>
    }
}
