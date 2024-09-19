/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod edit;
pub mod list;
pub mod schema;
pub mod search;

use std::{collections::BTreeMap, str::FromStr};

use crate::{
    components::{
        form::input::{Duration, Rate},
        icon::{
            IconCircleStack, IconCodeBracket, IconHandRaised, IconInbox, IconInboxArrowDown,
            IconInboxStack, IconKey, IconServer, IconServerStack, IconShieldCheck, IconSignal,
        },
        layout::{LayoutBuilder, MenuItem},
    },
    core::{
        form::{FormData, FormValue},
        schema::*,
    },
};
use ahash::AHashMap;
use humansize::{format_size, DECIMAL};
use leptos::view;
use serde::{Deserialize, Serialize};

pub type Settings = AHashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UpdateSettings {
    Delete {
        keys: Vec<String>,
    },
    Clear {
        prefix: String,
    },
    Insert {
        prefix: Option<String>,
        values: Vec<(String, String)>,
        assert_empty: bool,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct ReloadSettings {
    pub warnings: BTreeMap<String, ConfigWarning>,
    pub errors: BTreeMap<String, ConfigError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigWarning {
    Missing,
    AppliedDefault { default: String },
    Unread { value: String },
    Build { error: String },
    Parse { error: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigError {
    Parse { error: String },
    Build { error: String },
    Macro { error: String },
}

impl FormData {
    pub fn build_update(&self) -> Vec<UpdateSettings> {
        let mut updates = Vec::new();
        let mut insert_prefix = None;
        let mut assert_empty = false;

        match &self.schema.typ {
            SchemaType::Record { prefix, .. } => {
                if self.is_update {
                    updates.push(UpdateSettings::Clear {
                        prefix: format!("{prefix}.{}.", self.value_as_str("_id").unwrap()),
                    });
                } else {
                    assert_empty = true;
                }

                insert_prefix = format!("{prefix}.{}", self.value_as_str("_id").unwrap()).into();
            }
            SchemaType::Entry { prefix } => {
                updates.push(UpdateSettings::Insert {
                    prefix: None,
                    assert_empty: !self.is_update,
                    values: vec![(
                        format!("{prefix}.{}", self.value_as_str("_id").unwrap()),
                        self.value_as_str("_value").unwrap_or_default().to_string(),
                    )],
                });
                return updates;
            }
            SchemaType::List => {
                if self.is_update {
                    let mut delete_keys = Vec::new();
                    for field in self.schema.fields.values() {
                        if field.is_multivalue() {
                            updates.push(UpdateSettings::Clear {
                                prefix: format!("{}.", field.id),
                            });
                            delete_keys.push(field.id.to_string());
                        } else if self.value_is_empty(field.id) {
                            delete_keys.push(field.id.to_string());
                        }
                    }

                    if !delete_keys.is_empty() {
                        updates.push(UpdateSettings::Delete { keys: delete_keys });
                    }
                }
            }
        }

        let mut key_values = Vec::new();
        for (key, value) in &self.values {
            if key.starts_with('_') {
                continue;
            }

            match value {
                FormValue::Value(value) if !value.is_empty() => {
                    key_values.push((key.to_string(), value.to_string()));
                }
                FormValue::Array(values) if !values.is_empty() => {
                    let total_values = values.len();
                    if total_values > 1 {
                        let pad_len = (total_values - 1).to_string().len();

                        for (idx, value) in values.iter().enumerate() {
                            key_values.push((format!("{key}.{idx:0>pad_len$}"), value.to_string()));
                        }
                    } else {
                        key_values.push((key.to_string(), values.first().unwrap().to_string()));
                    }
                }
                FormValue::Expression(expr) if !expr.is_empty() => {
                    if !expr.if_thens.is_empty() {
                        let total_values = expr.if_thens.len();
                        let pad_len = total_values.to_string().len();

                        for (idx, if_then) in expr.if_thens.iter().enumerate() {
                            key_values.push((
                                format!("{key}.{idx:0>pad_len$}.if"),
                                if_then.if_.to_string(),
                            ));
                            key_values.push((
                                format!("{key}.{idx:0>pad_len$}.then"),
                                if_then.then_.to_string(),
                            ));
                        }

                        key_values.push((
                            format!("{key}.{total_values:0>pad_len$}.else"),
                            expr.else_.to_string(),
                        ));
                    } else {
                        key_values.push((key.to_string(), expr.else_.to_string()));
                    }
                }
                _ => (),
            }
        }

        if !key_values.is_empty() {
            updates.push(UpdateSettings::Insert {
                prefix: insert_prefix,
                values: key_values,
                assert_empty,
            });
        }

        updates
    }
}

pub trait SettingsValues {
    fn array_values(&self, prefix: &str) -> Vec<(&str, &str)>;
    fn format(&self, field: &Field) -> String;
}

impl SettingsValues for Settings {
    fn array_values(&self, key: &str) -> Vec<(&str, &str)> {
        let full_prefix = key;
        let prefix = format!("{key}.");

        let mut results = self
            .iter()
            .filter_map(move |(key, value)| {
                if key.starts_with(&prefix) || key == full_prefix {
                    (key.as_str(), value.as_str()).into()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Sort by key
        results.sort_by(|(l_key, _), (r_key, _)| l_key.cmp(r_key));
        results
    }

    fn format(&self, field: &Field) -> String {
        match &field.typ_ {
            Type::Select {
                source: Source::Static(items),
                typ: SelectType::Single,
            } => {
                let value = self
                    .get(field.id)
                    .map(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string();
                items
                    .iter()
                    .find_map(|(k, v)| if k == &value { Some(*v) } else { None })
                    .map(|s| s.to_string())
                    .unwrap_or(value)
            }
            Type::Array => self
                .array_values(field.id)
                .first()
                .map(|(_, v)| v.to_string())
                .unwrap_or_default(),
            Type::Boolean => {
                if self.get(field.id).map_or(false, |s| s == "true") {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                }
            }
            Type::Duration => self
                .get(field.id)
                .and_then(|s| Duration::from_str(s).ok())
                .and_then(|d| d.format())
                .unwrap_or_default(),
            Type::Rate => self
                .get(field.id)
                .and_then(|s| Rate::from_str(s).ok())
                .and_then(|d| d.format())
                .unwrap_or_default(),
            Type::Size => self
                .get(field.id)
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| format_size(s, DECIMAL))
                .unwrap_or_default(),
            _ => self
                .get(field.id)
                .map(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl LayoutBuilder {
    pub fn settings(manage_url: &'static str) -> Vec<MenuItem> {
        LayoutBuilder::new("/settings")
            // Server
            .create("Server")
            .icon(view! { <IconServerStack/> })
            // Network
            .create("Network")
            .route("/network/edit")
            .insert(true)
            // HTTP
            .create("HTTP")
            .route("/http/edit")
            .insert(true)
            // System
            .create("System")
            .route("/system/edit")
            .insert(true)
            // Listener
            .create("Listeners")
            .route("/listener")
            .insert(true)
            // TLS
            .create("TLS")
            .create("ACME Providers")
            .route("/acme")
            .insert(true)
            .create("Certificates")
            .route("/certificate")
            .insert(true)
            .create("Defaults")
            .route("/tls/edit")
            .insert(true)
            .insert(true)
            // System
            .create("Cluster")
            .route("/cluster/edit")
            .insert(true)
            // Cache
            .create("Cache")
            .route("/cache/edit")
            .insert(true)
            // Cache
            .create("Enterprise")
            .route("/enterprise/edit")
            .insert(true)
            .insert(true)
            // Storage
            .create("Storage")
            .icon(view! { <IconCircleStack/> })
            .create("Settings")
            .route("/storage/edit")
            .insert(true)
            .create("Stores")
            .route("/store")
            .insert(true)
            .insert(true)
            // Authentication
            .create("Authentication")
            .icon(view! { <IconKey/> })
            .create("Settings")
            .route("/authentication/edit")
            .insert(true)
            .create("Directories")
            .route("/directory")
            .insert(true)
            .create("OAuth")
            .route("/oauth/edit")
            .insert(true)
            .insert(true)
            // SMTP
            .create("SMTP")
            .icon(view! { <IconInboxArrowDown/> })
            .create("Inbound")
            .create("Connect stage")
            .route("/smtp-in-connect/edit")
            .insert(true)
            .create("EHLO stage")
            .route("/smtp-in-ehlo/edit")
            .insert(true)
            .create("AUTH stage")
            .route("/smtp-in-auth/edit")
            .insert(true)
            .create("MAIL stage")
            .route("/smtp-in-mail/edit")
            .insert(true)
            .create("RCPT stage")
            .route("/smtp-in-rcpt/edit")
            .insert(true)
            .create("DATA stage")
            .route("/smtp-in-data/edit")
            .insert(true)
            .create("Extensions")
            .route("/smtp-in-extensions/edit")
            .insert(true)
            .create("Session Limits")
            .route("/smtp-in-limits/edit")
            .insert(true)
            .create("MTA-STS")
            .route("/smtp-in-mta-sts/edit")
            .insert(true)
            .create("Throttles")
            .route("/smtp-in-throttle")
            .insert(true)
            .create("Milters")
            .route("/milter")
            .insert(true)
            .create("MTA Hooks")
            .route("/mta-hooks")
            .insert(true)
            .create("Pipes")
            .route("/pipe")
            .insert(true)
            .insert(true)
            .create("Outbound")
            .create("Queue")
            .route("/smtp-out-queue/edit")
            .insert(true)
            .create("Routing")
            .route("/smtp-out-routing/edit")
            .insert(true)
            .create("TLS")
            .route("/smtp-out-tls/edit")
            .insert(true)
            .create("Limits")
            .route("/smtp-out-limits/edit")
            .insert(true)
            .create("DNS Resolver")
            .route("/smtp-out-resolver/edit")
            .insert(true)
            .create("Remote Hosts")
            .route("/smtp-out-remote")
            .insert(true)
            .create("Throttles")
            .route("/smtp-out-throttle")
            .insert(true)
            .create("Quotas")
            .route("/smtp-out-quota")
            .insert(true)
            .insert(true)
            .create("DKIM")
            .create("Settings")
            .route("/dkim/edit")
            .insert(true)
            .create("Signatures")
            .route("/signature")
            .insert(true)
            .insert(true)
            .create("ARC")
            .route("/arc/edit")
            .insert(true)
            .create("SPF")
            .route("/spf/edit")
            .insert(true)
            .create("DMARC")
            .route("/dmarc/edit")
            .insert(true)
            .create("Reporting")
            .route("/report/edit")
            .insert(true)
            .insert(true)
            // JMAP
            .create("JMAP")
            .icon(view! { <IconInboxStack/> })
            .create("Session")
            .route("/jmap-session/edit")
            .insert(true)
            .create("Push Notifications")
            .route("/jmap-push/edit")
            .insert(true)
            .create("Web Sockets")
            .route("/jmap-web-sockets/edit")
            .insert(true)
            .create("Protocol Limits")
            .route("/jmap-limits/edit")
            .insert(true)
            .create("Rate Limits")
            .route("/jmap-rate-limit/edit")
            .insert(true)
            .insert(true)
            // IMAP
            .create("IMAP & POP3")
            .icon(view! { <IconInbox/> })
            .create("Authentication")
            .route("/imap-auth/edit")
            .insert(true)
            .create("Folders")
            .route("/imap-folders/edit")
            .insert(true)
            .create("Protocol Limits")
            .route("/imap-limits/edit")
            .insert(true)
            .create("Rate Limits")
            .route("/imap-rate-limit/edit")
            .insert(true)
            .insert(true)
            // Security
            .create("Security")
            .icon(view! { <IconHandRaised/> })
            // Fail2ban
            .create("Fail2ban")
            .route("/fail2ban/edit")
            .insert(true)
            // Blocked IPs
            .create("Blocked IPs")
            .route("/blocked-ip")
            .insert(true)
            // Blocked IPs
            .create("Allowed IPs")
            .route("/allowed-ip")
            .insert(true)
            .insert(true)
            // Telemetry
            .create("Telemetry")
            .icon(view! { <IconSignal/> })
            .create("Logging & Tracing")
            .route("/tracing")
            .insert(true)
            .create("Metrics")
            .route("/metrics/edit")
            .insert(true)
            .create("Alerts")
            .route("/alerts")
            .insert(true)
            .create("Webhooks")
            .route("/web-hooks")
            .insert(true)
            .create("Custom levels")
            .route("/custom-levels")
            .insert(true)
            .create("History")
            .route("/telemetry-history/edit")
            .insert(true)
            .insert(true)
            // SPAM Filter
            .create("Antispam")
            .icon(view! { <IconShieldCheck/> })
            .create("Settings")
            .route("/spam-settings/edit")
            .insert(true)
            .create("Scores")
            .route("/spam-scores")
            .insert(true)
            .create("Free domains")
            .route("/spam-free")
            .insert(true)
            .create("Disposable domains")
            .route("/spam-disposable")
            .insert(true)
            .create("URL Redirectors")
            .route("/spam-redirect")
            .insert(true)
            .create("Trusted domains")
            .route("/spam-allow")
            .insert(true)
            .create("Blocked domains")
            .route("/spam-block")
            .insert(true)
            .create("DMARC domains")
            .route("/spam-dmarc")
            .insert(true)
            .create("SPF/DKIM domains")
            .route("/spam-spdk")
            .insert(true)
            .create("Spam traps")
            .route("/spam-trap")
            .insert(true)
            .create("MIME Types")
            .route("/spam-mime")
            .insert(true)
            .insert(true)
            // Sieve Scripting
            .create("Scripting")
            .icon(view! { <IconCodeBracket/> })
            .create("Settings")
            .route("/sieve-settings/edit")
            .insert(true)
            .create("Limits")
            .route("/sieve-limits/edit")
            .insert(true)
            .create("System Scripts")
            .route("/trusted-script")
            .insert(true)
            .create("User Scripts")
            .route("/untrusted-script")
            .insert(true)
            .insert(true)
            .create("Management")
            .icon(view! { <IconServer/> })
            .raw_route(manage_url)
            .insert(true)
            .menu_items
    }
}
