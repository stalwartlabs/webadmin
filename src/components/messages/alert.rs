use std::time::Duration;

use leptos::*;

use crate::core::http;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
    Success,
    Error,
    Warning,
    None,
}

#[derive(Clone)]
pub struct Alert {
    pub typ: AlertType,
    pub message: String,
    pub details: Option<String>,
    pub timeout: Option<Duration>,
}

pub fn init_alerts() {
    provide_context(create_rw_signal(Alert::disabled()));
}

pub fn use_alerts() -> RwSignal<Alert> {
    let signal = expect_context::<RwSignal<Alert>>();
    signal.set(Alert::disabled());
    signal
}

#[component]
pub fn Alerts() -> impl IntoView {
    let alert = use_context::<RwSignal<Alert>>().unwrap();

    create_effect(move |_| {
        let alert_data = alert.get();
        if alert_data.is_open() {
            if let Some(timeout) = alert_data.timeout() {
                set_timeout(
                    move || {
                        alert.update(|alert| {
                            alert.close();
                        });
                    },
                    timeout,
                );
            }
        }
    });

    view! {
        <div class=move || {
            match alert.get().typ {
                AlertType::None => "hidden",
                _ => "pb-5",
            }
        }>
            <div
                class=move || {
                    match alert.get().typ {
                        AlertType::Success => {
                            "bg-teal-50 border border-teal-200 text-sm text-teal-800 rounded-lg p-4 dark:bg-teal-800/10 dark:border-teal-900 dark:text-teal-500"
                        }
                        AlertType::Error => {
                            "bg-red-50 border border-red-200 text-sm text-red-800 rounded-lg p-4 dark:bg-red-800/10 dark:border-red-900 dark:text-red-500"
                        }
                        AlertType::Warning => {
                            "bg-yellow-50 border border-yellow-200 text-sm text-yellow-800 rounded-lg p-4 dark:bg-yellow-800/10 dark:border-yellow-900 dark:text-yellow-500"
                        }
                        AlertType::None => "hidden",
                    }
                }

                role="alert"
            >
                <div class="flex">
                    <div class="flex-shrink-0">
                        {move || {
                            match alert.get().typ {
                                AlertType::Success | AlertType::None => {
                                    view! {
                                        <svg
                                            class="flex-shrink-0 size-4 text-blue-600 mt-1"
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="teal"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"></path>
                                            <path d="m9 12 2 2 4-4"></path>
                                        </svg>
                                    }
                                        .into_any()
                                }
                                AlertType::Error => {
                                    view! {
                                        <svg
                                            class="flex-shrink-0 size-4 mt-0.5"
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="red"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <path d="m15 9-6 6"></path>
                                            <path d="m9 9 6 6"></path>
                                        </svg>
                                    }
                                        .into_any()
                                }
                                AlertType::Warning => {
                                    view! {
                                        <svg
                                            class="flex-shrink-0 size-4 mt-0.5"
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="#854d0e"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path>
                                            <path d="M12 9v4"></path>
                                            <path d="M12 17h.01"></path>
                                        </svg>
                                    }
                                        .into_any()
                                }
                            }
                        }}

                    </div>
                    <div class="ms-4">
                        <h3 class="text-sm font-semibold">
                            {move || { alert.get().message.clone() }}
                        </h3>
                        <div class="mt-1 text-sm">
                            {move || {
                                alert.get().details.as_deref().unwrap_or_default().to_string()
                            }}

                        </div>
                    </div>

                    <div class="ps-3 ms-auto">
                        <div class="-mx-1.5 -my-1.5">
                            <button
                                type="button"
                                class=move || {
                                    match alert.get().typ {
                                        AlertType::Success => {
                                            "inline-flex bg-teal-50 rounded-lg p-1.5 text-teal-500 hover:bg-teal-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-teal-50 focus:ring-teal-600 dark:bg-transparent dark:hover:bg-teal-800/50 dark:text-teal-600"
                                        }
                                        AlertType::Error => {
                                            "inline-flex bg-red-50 rounded-lg p-1.5 text-red-500 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-red-50 focus:ring-red-600 dark:bg-transparent dark:hover:bg-red-800/50 dark:text-red-600"
                                        }
                                        AlertType::Warning => {
                                            "inline-flex bg-yellow-50 rounded-lg p-1.5 text-yellow-500 hover:bg-yellow-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-yellow-50 focus:ring-yellow-600 dark:bg-transparent dark:hover:bg-yellow-800/50 dark:text-yellow-600"
                                        }
                                        AlertType::None => "hidden",
                                    }
                                }

                                on:click=move |_| {
                                    alert
                                        .update(|alert| {
                                            alert.close();
                                        });
                                }
                            >

                                <span class="sr-only">Dismiss</span>
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
                    </div>
                </div>
            </div>
        </div>
    }
}

impl Alert {
    pub fn disabled() -> Self {
        Self::new(AlertType::None, "")
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(AlertType::Success, message).with_timeout(Duration::from_secs(5))
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(AlertType::Error, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(AlertType::Warning, message)
    }

    pub fn new(typ: AlertType, message: impl Into<String>) -> Self {
        Self {
            typ,
            message: message.into(),
            details: None,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn close(&mut self) {
        self.typ = AlertType::None;
    }

    pub fn is_open(&self) -> bool {
        self.typ != AlertType::None
    }

    pub fn is_closed(&self) -> bool {
        self.typ == AlertType::None
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

impl From<http::Error> for Alert {
    fn from(value: http::Error) -> Self {
        match value {
            http::Error::Network(details) => Alert::error("Network error").with_details(details),
            http::Error::Serializer { error, response } => {
                log::debug!("Failed to deserialize request: {}", response);
                Alert::error("Failed to deserialize response").with_details(error)
            }
            http::Error::Server { error, details } => Alert::error(error).with_details(details),
            http::Error::NotFound => Alert::error("Not found"),
            http::Error::Unauthorized => Alert::error("Unauthorized"),
        }
    }
}
