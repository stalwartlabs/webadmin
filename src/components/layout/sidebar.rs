use leptos::*;
use leptos_router::use_location;

use super::MenuItem;

#[component]
pub fn SideBar(menu_items: Vec<MenuItem>, show_sidebar: RwSignal<bool>) -> impl IntoView {
    let current_route = create_memo(move |_| use_location().pathname.get());

    view! {
        <div
            id="application-sidebar"
            class="hs-overlay hs-overlay-open:translate-x-0 -translate-x-full transition-all duration-300 transform fixed top-0 start-0 bottom-0 z-[60] w-64 bg-white border-e border-gray-200 pt-7 pb-10 overflow-y-auto lg:block lg:translate-x-0 lg:end-auto lg:bottom-0 [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-track]:bg-gray-100 [&::-webkit-scrollbar-thumb]:bg-gray-300 dark:[&::-webkit-scrollbar-track]:bg-slate-700 dark:[&::-webkit-scrollbar-thumb]:bg-slate-500 dark:bg-gray-800 dark:border-gray-700"
            class:hidden=move || !show_sidebar.get()
            class:open=move || show_sidebar.get()
        >
            <div class="px-6">
                <a
                    class="flex-none text-xl font-semibold dark:text-white"
                    href="#"
                    aria-label="Brand"
                >
                    Stalwart
                </a>
            </div>

            <nav
                class="hs-accordion-group p-6 w-full flex flex-col flex-wrap"
                data-hs-accordion-always-open
            >

                <ul class="space-y-1.5">
                    <For each=move || menu_items.clone() key=|item| item.id() let:item>

                        {if !item.children.is_empty() {
                            let has_sub_children = item
                                .children
                                .first()
                                .unwrap()
                                .children
                                .is_empty();
                            let children = item.children.clone();
                            let is_displayed = create_rw_signal(false);
                            let is_active = create_memo(move |_| {
                                is_displayed.get()
                                    || children
                                        .iter()
                                        .any(|i| {
                                            i
                                                .match_route
                                                .as_ref()
                                                .map_or(false, |f| current_route.get().starts_with(f))
                                                || i
                                                    .children
                                                    .iter()
                                                    .any(|i| {
                                                        i.match_route
                                                            .as_ref()
                                                            .map_or(false, |f| current_route.get().starts_with(f))
                                                    })
                                        })
                            });
                            view! {
                                <li class="hs-accordion">
                                    <button
                                        type="button"
                                        class="hs-accordion-toggle w-full text-start flex items-center gap-x-3.5 py-2 px-2.5 hs-accordion-active:text-blue-600 hs-accordion-active:hover:bg-transparent text-sm text-slate-700 rounded-lg hover:bg-gray-100 dark:bg-gray-800 dark:hover:bg-gray-900 dark:text-slate-400 dark:hover:text-slate-300 dark:hs-accordion-active:text-white dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                        class:active=move || is_active.get()
                                        on:click=move |_| is_displayed.update(|v| *v = !*v)
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
                                            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"></path>
                                            <circle cx="9" cy="7" r="4"></circle>
                                            <path d="M22 21v-2a4 4 0 0 0-3-3.87"></path>
                                            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
                                        </svg>

                                        {item.name}

                                        <svg
                                            class="hs-accordion-active:block ms-auto size-4"
                                            class:hidden=move || !is_active.get()
                                            class:block=move || is_active.get()
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
                                            <path d="m18 15-6-6-6 6"></path>
                                        </svg>

                                        <svg
                                            class="hs-accordion-active:hidden ms-auto size-4"
                                            class:hidden=move || is_active.get()
                                            class:block=move || !is_active.get()
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
                                            <path d="m6 9 6 6 6-6"></path>
                                        </svg>
                                    </button>

                                    <div
                                        class="hs-accordion-content w-full overflow-hidden transition-[height] duration-300"
                                        hidden=move || !is_active.get()
                                    >
                                        <ul class=move || {
                                            if has_sub_children {
                                                "pt-2 ps-2"
                                            } else {
                                                "hs-accordion-group pt-3 ps-2"
                                            }
                                        }>

                                            <For
                                                each=move || item.children.clone()
                                                key=|item| item.id()
                                                let:item
                                            >

                                                {if !item.children.is_empty() {
                                                    let is_displayed = create_rw_signal(false);
                                                    let children = item.children.clone();
                                                    let is_active = create_memo(move |_| {
                                                        children
                                                            .iter()
                                                            .any(|i| {
                                                                is_displayed.get()
                                                                    || i
                                                                        .match_route
                                                                        .as_ref()
                                                                        .map_or(false, |f| current_route.get().starts_with(f))
                                                            })
                                                    });
                                                    view! {
                                                        <li class="hs-accordion">
                                                            <button
                                                                type="button"
                                                                class="hs-accordion-toggle w-full text-start flex items-center gap-x-3.5 py-2 px-2.5 hs-accordion-active:text-blue-600 hs-accordion-active:hover:bg-transparent text-sm text-slate-700 rounded-lg hover:bg-gray-100 dark:bg-gray-800 dark:hover:bg-gray-900 dark:text-slate-400 dark:hover:text-slate-300 dark:hs-accordion-active:text-white dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                                                on:click=move |_| is_displayed.update(|v| *v = !*v)
                                                            >

                                                                {item.name}

                                                                <svg
                                                                    class="hs-accordion-active:block ms-auto size-4"
                                                                    class:hidden=move || !is_active.get()
                                                                    class:block=move || is_active.get()
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
                                                                    <path d="m18 15-6-6-6 6"></path>
                                                                </svg>

                                                                <svg
                                                                    class="hs-accordion-active:hidden ms-auto size-4"
                                                                    class:hidden=move || is_active.get()
                                                                    class:block=move || !is_active.get()
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
                                                                    <path d="m6 9 6 6 6-6"></path>
                                                                </svg>
                                                            </button>

                                                            <div
                                                                class="hs-accordion-content w-full overflow-hidden transition-[height] duration-300"
                                                                hidden=move || !is_active.get()
                                                            >
                                                                <ul class="pt-2 ps-2">
                                                                    <For
                                                                        each=move || item.children.clone()
                                                                        key=|item| item.id()
                                                                        let:item
                                                                    >

                                                                        {
                                                                            let route = item.match_route.clone().unwrap();
                                                                            view! {
                                                                                <li>
                                                                                    <a
                                                                                        class=move || {
                                                                                            format!(
                                                                                                "flex items-center gap-x-3.5 py-2 px-2.5 text-sm text-slate-700 rounded-lg hover:bg-gray-100 dark:bg-gray-800 dark:text-slate-400 dark:hover:text-slate-300 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600{}",
                                                                                                if current_route.get().starts_with(&route) {
                                                                                                    " bg-gray-100"
                                                                                                } else {
                                                                                                    ""
                                                                                                },
                                                                                            )
                                                                                        }

                                                                                        href=move || item.route.clone().unwrap()
                                                                                    >
                                                                                        {item.name}
                                                                                    </a>
                                                                                </li>
                                                                            }
                                                                                .into_view()
                                                                        }

                                                                    </For>

                                                                </ul>
                                                            </div>
                                                        </li>
                                                    }
                                                        .into_view()
                                                } else {
                                                    let route = item.match_route.unwrap();
                                                    view! {
                                                        <li>
                                                            <a
                                                                class=move || {
                                                                    format!(
                                                                        "flex items-center gap-x-3.5 py-2 px-2.5 text-sm text-slate-700 rounded-lg hover:bg-gray-100 dark:bg-gray-800 dark:text-slate-400 dark:hover:text-slate-300 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600{}",
                                                                        if current_route.get().starts_with(&route) {
                                                                            " bg-gray-100"
                                                                        } else {
                                                                            ""
                                                                        },
                                                                    )
                                                                }

                                                                href=move || item.route.clone().unwrap()
                                                            >
                                                                {item.name}
                                                            </a>
                                                        </li>
                                                    }
                                                        .into_view()
                                                }}

                                            </For>

                                        </ul>
                                    </div>

                                </li>
                            }
                                .into_view()
                        } else {
                            let route = item.route.clone().unwrap();
                            let class = format!(
                                "w-full flex items-center gap-x-3.5 py-2 px-2.5 text-sm text-slate-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-900 dark:text-slate-400 dark:hover:text-slate-300 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600{}",
                                if route == use_location().pathname.get() {
                                    " bg-gray-100"
                                } else {
                                    ""
                                },
                            );
                            view! {
                                <li>
                                    <a class=class href=route>
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
                                            <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                                            <polyline points="9 22 9 12 15 12 15 22"></polyline>
                                        </svg>
                                        {item.name}
                                    </a>
                                </li>
                            }
                                .into_view()
                        }}

                    </For>

                </ul>
            </nav>
        </div>
    }
}
