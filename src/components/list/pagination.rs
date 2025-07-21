/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;

use crate::components::icon::{IconArrowLeft, IconArrowRight};

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
            <Show when=move || { total_results.get().is_some_and(|r| r > 0) }>
                <div class="inline-flex items-center gap-x-2">

                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        <span class="font-semibold text-gray-800 dark:text-gray-200">
                            {move || { total_results.get().map_or(0, |r| r) }}
                        </span>
                        " results. Page"

                    </p>
                    <div class="max-w-sm space-y-3">
                        <select
                            class="py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400"
                            on:change=move |ev| {
                                on_page_change.call(event_target_value(&ev).parse().unwrap_or(1));
                            }
                        >

                            {move || {
                                let total_pages = total_pages.get();
                                let current_page = current_page.get();
                                let mut views = Vec::with_capacity(10);
                                if total_pages > 10 {
                                    let start = if current_page > 6 { current_page - 5 } else { 1 };
                                    let end = if current_page + 5 > total_pages {
                                        total_pages
                                    } else {
                                        current_page + 5
                                    };
                                    if start > 1 {
                                        views
                                            .push(
                                                view! { <option selected=current_page == 1>{1}</option> }
                                                    .into_view(),
                                            );
                                        if start > 2 {
                                            views
                                                .push(
                                                    view! { <option disabled=true>...</option> }.into_view(),
                                                );
                                        }
                                    }
                                    for page in start..=end {
                                        views
                                            .push(
                                                view! {
                                                    <option selected=current_page == page>{page}</option>
                                                }
                                                    .into_view(),
                                            );
                                    }
                                    if end < total_pages {
                                        if end < total_pages - 1 {
                                            views
                                                .push(
                                                    view! { <option disabled=true>...</option> }.into_view(),
                                                );
                                        }
                                        views
                                            .push(
                                                view! {
                                                    <option selected=current_page
                                                        == total_pages>{total_pages}</option>
                                                }
                                                    .into_view(),
                                            );
                                    }
                                } else {
                                    for page in 1..=total_pages {
                                        views
                                            .push(
                                                view! {
                                                    <option selected=current_page == page>{page}</option>
                                                }
                                                    .into_view(),
                                            );
                                    }
                                }
                                views.collect_view()
                            }}

                        </select>
                    </div>

                    <p class="text-sm text-gray-600 dark:text-gray-400">"of " {total_pages}</p>
                </div>

            </Show>

            <div>
                <div class="inline-flex gap-x-2">
                    <button
                        type="button"
                        class="py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        disabled=move || {
                            total_results.get().is_none_or(|r| r == 0) || current_page.get() <= 1
                        }

                        on:click=move |_| {
                            on_page_change.call(current_page.get() - 1);
                        }
                    >

                        <IconArrowLeft attr:class="flex-shrink-0 size-4"/>

                        Prev
                    </button>

                    <Suspense>
                        <button
                            type="button"
                            class="py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                            on:click=move |_| {
                                on_page_change.call(current_page.get() + 1);
                            }

                            disabled=move || { current_page.get() >= total_pages.get() }
                        >

                            Next
                            <IconArrowRight attr:class="flex-shrink-0 size-4"/>
                        </button>

                    </Suspense>

                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ItemPagination(
    #[prop(into)] current_item: MaybeSignal<u32>,
    #[prop(into)] total_items: MaybeSignal<u32>,
    #[prop(into)] on_item_change: Callback<u32, ()>,
) -> impl IntoView {
    view! {
        <Show when=move || { total_items.get() > 1 }>
            <nav class="flex items-center gap-x-1">
                <button
                    type="button"
                    class="min-h-[32px] min-w-8 py-2 px-2 inline-flex justify-center items-center gap-x-2 text-sm rounded-full text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-white/10 dark:focus:bg-white/10"
                    disabled=move || { current_item.get() <= 1 }
                    on:click=move |_| {
                        on_item_change.call(current_item.get() - 1);
                    }
                >

                    <IconArrowLeft attr:class="flex-shrink-0 size-3.5"/>
                    <span aria-hidden="true" class="sr-only">
                        Previous
                    </span>
                </button>
                <div class="flex items-center gap-x-1">
                    <span class="min-h-[32px] min-w-8 flex justify-center items-center border border-gray-200 text-gray-800 py-1 px-3 text-sm rounded-full focus:outline-none focus:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:border-gray-700 dark:text-white dark:bg-white/10">
                        {current_item}
                    </span>
                    <span class="min-h-[32px] flex justify-center items-center text-gray-500 py-1.5 px-1.5 text-sm dark:text-gray-500">
                        of
                    </span>
                    <span class="min-h-[32px] flex justify-center items-center text-gray-500 py-1.5 px-1.5 text-sm dark:text-gray-500">
                        {total_items}
                    </span>
                </div>
                <button
                    type="button"
                    class="min-h-[32px] min-w-8 py-2 px-2 inline-flex justify-center items-center gap-x-2 text-sm rounded-full text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-white/10 dark:focus:bg-white/10"
                    disabled=move || { current_item.get() >= total_items.get() }
                    on:click=move |_| {
                        on_item_change.call(current_item.get() + 1);
                    }
                >

                    <span aria-hidden="true" class="sr-only">
                        Next
                    </span>
                    <IconArrowRight attr:class="flex-shrink-0 size-3.5"/>
                </button>
            </nav>
        </Show>
    }
}
