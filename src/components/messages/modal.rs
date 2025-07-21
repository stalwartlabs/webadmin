/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::{html::Div, *};
use leptos_use::on_click_outside;
use std::sync::Arc;

use crate::components::icon::IconXMark;

pub trait ModalCb: Fn() + 'static {}

#[derive(Clone)]
pub struct Modal {
    pub is_open: bool,
    pub title: String,
    pub message: String,
    pub button_text: String,
    pub danger: bool,
    pub on_confirm: Arc<dyn Fn()>,
}

pub fn init_modals() {
    provide_context(create_rw_signal(Modal::default()));
}

pub fn use_modals() -> RwSignal<Modal> {
    let signal = expect_context::<RwSignal<Modal>>();
    signal.set(Modal::default());
    signal
}

#[component]
pub fn Modal() -> impl IntoView {
    let modal = expect_context::<RwSignal<Modal>>();
    let (processing, set_processing) = create_signal(false);

    // Dismiss modal when "Escape" (or 'q') key is pressed
    let dismiss_modal_with_keyboard = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" || ev.key() == "q" || ev.key() == "Q" {
            modal.update(|modal| {
                modal.is_open = false;
            });
        }
    });
    on_cleanup(move || dismiss_modal_with_keyboard.remove());

    // Click outside modal to dismiss
    let modal_target: NodeRef<Div> = create_node_ref::<Div>();
    on_cleanup(on_click_outside(modal_target, move |_| {
        modal.update(|modal| {
            modal.is_open = false;
        });
    }));

    view! {
        <Show when=move || modal.get().is_open fallback=|| ()>
            <Portal mount=document().get_element_by_id("portal_root").unwrap()>
                <div class="transition duration fixed inset-0 z-50 bg-gray-900 bg-opacity-50 dark:bg-opacity-80 hs-overlay-backdrop">

                    <div
                        id="hs-vertically-centered-modal"
                        class="open hs-overlay size-full fixed top-0 start-0 z-[80] overflow-x-hidden overflow-y-auto"
                    >

                        <div
                            class="hs-overlay-open:mt-7 hs-overlay-open:opacity-100 hs-overlay-open:duration-500 mt-0 opacity-0 ease-out transition-all sm:max-w-lg sm:w-full m-3 sm:mx-auto min-h-[calc(100%-3.5rem)] flex items-center"
                            node_ref=modal_target
                        >
                            <div class="w-full flex flex-col bg-white border shadow-sm rounded-xl dark:bg-gray-800 dark:border-gray-700 dark:shadow-slate-700/[.7]">
                                <div class="flex justify-between items-center py-3 px-4 border-b dark:border-gray-700">
                                    <h3 class="font-bold text-gray-800 dark:text-white">
                                        {move || { modal.get().title }}
                                    </h3>
                                    <button
                                        type="button"
                                        class="flex justify-center items-center size-7 text-sm font-semibold rounded-full border border-transparent text-gray-800 hover:bg-gray-100 disabled:opacity-50 disabled:pointer-events-none dark:text-white dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                        on:click=move |_| {
                                            modal
                                                .update(|modal| {
                                                    modal.is_open = false;
                                                });
                                        }
                                    >

                                        <span class="sr-only">Close</span>
                                        <IconXMark/>

                                    </button>
                                </div>
                                <div class="p-4 overflow-y-auto">
                                    <p class="text-gray-800 dark:text-gray-400">
                                        {move || { modal.get().message }}
                                    </p>
                                </div>
                                <div class="flex justify-end items-center gap-x-2 py-3 px-4 border-t dark:border-gray-700">
                                    <button
                                        type="button"
                                        class="py-2 px-3 inline-flex items-center gap-x-2 text-sm font-medium rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-white dark:hover:bg-gray-800 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                        on:click=move |_| {
                                            modal
                                                .update(|modal| {
                                                    modal.is_open = false;
                                                });
                                        }
                                    >

                                        Close
                                    </button>
                                    <button
                                        type="button"
                                        class=move || {
                                            if modal.get().danger {
                                                "py-3 px-4 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-red-500 text-white hover:bg-red-600 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                            } else {
                                                "py-3 px-4 inline-flex items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                            }
                                        }

                                        on:click=move |_| {
                                            modal
                                                .update(|modal| {
                                                    set_processing.set(true);
                                                    (modal.on_confirm)();
                                                    set_processing.set(false);
                                                    modal.is_open = false;
                                                });
                                        }

                                        disabled=move || processing.get()
                                    >

                                        {move || { modal.get().button_text }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Portal>
        </Show>
    }
}

impl Modal {
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            is_open: true,
            title: title.into(),
            message: String::new(),
            button_text: "Confirm".to_string(),
            danger: false,
            on_confirm: Arc::new(|| {}),
        }
    }
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_button(mut self, button_text: impl Into<String>) -> Self {
        self.button_text = button_text.into();
        self
    }

    pub fn with_dangerous_callback(mut self, on_confirm: impl Fn() + 'static) -> Self {
        self.danger = true;
        self.on_confirm = Arc::new(on_confirm);
        self
    }

    pub fn with_callback(mut self, on_confirm: impl Fn() + 'static) -> Self {
        self.danger = false;
        self.on_confirm = Arc::new(on_confirm);
        self
    }
}

impl Default for Modal {
    fn default() -> Self {
        Modal {
            is_open: false,
            title: String::new(),
            message: String::new(),
            button_text: "Confirm".to_string(),
            danger: false,
            on_confirm: Arc::new(|| {}),
        }
    }
}
