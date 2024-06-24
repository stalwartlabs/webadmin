/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::time::Duration;

use leptos::*;

use crate::{
    components::icon::{
        IconCheckCircle, IconExclamationCircle, IconExclamationTriangle, IconXMark,
    },
    core::http::{self, ManagementApiError},
    pages::config::{ConfigError, ConfigWarning, ReloadSettings},
};

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
    pub details: Option<View>,
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
    let alert = expect_context::<RwSignal<Alert>>();

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
                                        <IconCheckCircle
                                            attr:class="flex-shrink-0 size-4 text-blue-600 mt-1"
                                            attr:stroke="teal"
                                        />
                                    }
                                        .into_view()
                                }
                                AlertType::Error => {
                                    view! {
                                        <IconExclamationCircle
                                            attr:class="flex-shrink-0 size-4 mt-0.5"
                                            attr:stroke="red"
                                        />
                                    }
                                        .into_view()
                                }
                                AlertType::Warning => {
                                    view! {
                                        <IconExclamationTriangle
                                            attr:class="flex-shrink-0 size-4 mt-0.5"
                                            attr:stroke="#854d0e"
                                        />
                                    }
                                        .into_view()
                                }
                            }
                        }}

                    </div>
                    <div class="ms-4">
                        <h3 class="text-sm font-semibold">
                            {move || { alert.get().message.clone() }}
                        </h3>
                        <div class="mt-1 text-sm">{move || { alert.get().details }}</div>
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
                                <IconXMark/>

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

    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        let details = details.into();
        self.details = Some(details.into_view());
        self
    }

    pub fn with_details_list<T, I>(mut self, details: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<String>,
    {
        let items = details
            .into_iter()
            .map(|item| {
                view! { <li>{item.into()}</li> }
            })
            .collect_view();
        self.details =
            Some(view! { <ul class="list-disc space-y-1 ps-5">{items}</ul> }.into_view());
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
            http::Error::Server(error) => {
                let (title, details) = match error {
                    ManagementApiError::FieldAlreadyExists { field, value } => (
                        "Field already exists".to_string(),
                        format!(
                            "Another record exists with value {value:?} in field {field:?}."
                        ),
                    ),
                    ManagementApiError::FieldMissing { field } => (
                        "Missing required field".to_string(),
                        format!("Field {} is missing", field),
                    ),
                    ManagementApiError::NotFound { item } => {
                        ("Not found".to_string(), format!("{item} was not found"))
                    }
                    ManagementApiError::Unsupported { details } => {
                        ("Operation not allowed".to_string(), details)
                    }
                    ManagementApiError::AssertFailed => (
                        "Record already exists".to_string(),
                        "Another record with the same ID already exists".to_string(),
                    ),
                    ManagementApiError::Other { details } => {
                        ("Operation failed".to_string(), details)
                    }
                    ManagementApiError::UnsupportedDirectoryOperation { class } => (
                        format!("{class} directory cannot be managed"),
                        "Only internal directories support inserts and update operations."
                            .to_string(),
                    ),
                };

                Alert::error(title).with_details(details)
            }
            http::Error::NotFound => Alert::error("Not found"),
            http::Error::Unauthorized => Alert::error("Unauthorized"),
        }
    }
}

impl From<ReloadSettings> for Alert {
    fn from(value: ReloadSettings) -> Self {
        if value.errors.is_empty() && value.warnings.is_empty() {
            Alert::success("Settings successfully reloaded")
        } else {
            let messages = value
                .errors
                .iter()
                .map(|(key, error)| {
                    view! {
                        <li>
                            {match error {
                                ConfigError::Parse { error } => {
                                    format!("Failed to parse {key:?}: {error}")
                                }
                                ConfigError::Build { error } => {
                                    format!("Build error for {key:?}: {error}")
                                }
                                ConfigError::Macro { error } => {
                                    format!("Macro error on {key:?}: {error}")
                                }
                            }}

                        </li>
                    }
                })
                .chain(value.warnings.iter().map(|(key, warning)| {
                    view! {
                        <li>
                            {match warning {
                                ConfigWarning::Missing => format!("Waring: Missing setting {key:?}"),
                                ConfigWarning::AppliedDefault { default } => {
                                    format!("Warning: Applied default value {default:?} to {key:?}")
                                }
                            }}

                        </li>
                    }
                }))
                .collect_view();

            Alert {
                typ: if value.errors.is_empty() {
                    AlertType::Warning
                } else {
                    AlertType::Error
                },
                message: "Failed to reload settings".to_string(),
                details: Some(
                    view! { <ul class="list-disc space-y-1 ps-5">{messages}</ul> }.into_view(),
                ),
                timeout: None,
            }
        }
    }
}
