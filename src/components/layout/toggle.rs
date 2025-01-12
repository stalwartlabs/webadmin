/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;
use leptos_router::use_location;

use super::MenuItem;

#[derive(Clone, Debug, PartialEq, Eq)]
struct BreadCrumb {
    name: String,
    is_last: bool,
}

#[component]
pub fn ToggleNavigation(menu_items: Vec<MenuItem>, show_sidebar: RwSignal<bool>) -> impl IntoView {
    let build_path = move || {
        let current_route = use_location().pathname.get();
        let mut path = Vec::new();

        // Menus can have a maximum of 3 levels
        'outer: for item in &menu_items {
            if item
                .route
                .as_ref()
                .is_some_and( |route| route == &current_route)
            {
                path.push(BreadCrumb::child(&item.name));
                break;
            } else {
                for child in &item.children {
                    if child
                        .route
                        .as_ref()
                        .is_some_and( |route| route == &current_route)
                    {
                        path.push(BreadCrumb::parent(&item.name));
                        path.push(BreadCrumb::child(&child.name));
                        break 'outer;
                    } else {
                        for subchild in &child.children {
                            if subchild
                                .route
                                .as_ref()
                                .is_some_and( |route| route == &current_route)
                            {
                                path.push(BreadCrumb::parent(&item.name));
                                path.push(BreadCrumb::parent(&child.name));
                                path.push(BreadCrumb::child(&subchild.name));
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        path
    };

    view! {
        <div class="sticky top-0 inset-x-0 z-20 bg-white border-y px-4 sm:px-6 md:px-8 lg:hidden dark:bg-gray-800 dark:border-gray-700">
            <div class="flex items-center py-4">
                <button
                    type="button"
                    class="text-gray-500 hover:text-gray-600"
                    data-hs-overlay="#application-sidebar"
                    aria-controls="application-sidebar"
                    aria-label="Toggle navigation"
                    on:click=move |_| show_sidebar.update(|v| *v = !*v)
                >
                    <span class="sr-only">Toggle Navigation</span>
                    <svg
                        class="size-5"
                        width="16"
                        height="16"
                        fill="currentColor"
                        viewBox="0 0 16 16"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M2.5 12a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5zm0-4a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5zm0-4a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5z"
                        ></path>
                    </svg>
                </button>

                <ol class="ms-3 flex items-center whitespace-nowrap" aria-label="Breadcrumb">

                    <For each=build_path key=|item| item.name.clone() let:item>

                        {if !item.is_last {
                            view! {
                                <li class="flex items-center text-sm text-gray-800 dark:text-gray-400">
                                    {item.name}
                                    <svg
                                        class="flex-shrink-0 mx-3 overflow-visible size-2.5 text-gray-400 dark:text-gray-600"
                                        width="16"
                                        height="16"
                                        viewBox="0 0 16 16"
                                        fill="none"
                                        xmlns="http://www.w3.org/2000/svg"
                                    >
                                        <path
                                            d="M5 1L10.6869 7.16086C10.8637 7.35239 10.8637 7.64761 10.6869 7.83914L5 14"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                        ></path>
                                    </svg>
                                </li>
                            }
                                .into_view()
                        } else {
                            view! {
                                <li
                                    class="text-sm font-semibold text-gray-800 truncate dark:text-gray-400"
                                    aria-current="page"
                                >
                                    {item.name}
                                </li>
                            }
                                .into_view()
                        }}

                    </For>

                </ol>
            </div>
        </div>
    }
}

impl BreadCrumb {
    pub fn parent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_last: false,
        }
    }

    pub fn child(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_last: true,
        }
    }
}
