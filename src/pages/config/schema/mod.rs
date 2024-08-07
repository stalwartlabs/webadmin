/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod auth;
pub mod authentication;
pub mod directory;
pub mod imap;
pub mod jmap;
pub mod listener;
pub mod server;
pub mod sieve;
pub mod smtp;
pub mod spamlists;
pub mod storage;
pub mod store;
pub mod tls;
pub mod tracing;

use crate::core::schema::*;

pub const V_RECIPIENT: &str = "rcpt";
pub const V_RECIPIENT_DOMAIN: &str = "rcpt_domain";
pub const V_SENDER: &str = "sender";
pub const V_SENDER_DOMAIN: &str = "sender_domain";
pub const V_MX: &str = "mx";
pub const V_HELO_DOMAIN: &str = "helo_domain";
pub const V_AUTHENTICATED_AS: &str = "authenticated_as";
pub const V_LISTENER: &str = "listener";
pub const V_REMOTE_IP: &str = "remote_ip";
pub const V_REMOTE_PORT: &str = "remote_port";
pub const V_LOCAL_IP: &str = "local_ip";
pub const V_LOCAL_PORT: &str = "local_port";
pub const V_PRIORITY: &str = "priority";
pub const V_PROTOCOL: &str = "protocol";
pub const V_TLS: &str = "is_tls";
pub const V_RECIPIENTS: &str = "recipients";
pub const V_QUEUE_RETRY_NUM: &str = "retry_num";
pub const V_QUEUE_NOTIFY_NUM: &str = "notify_num";
pub const V_QUEUE_EXPIRES_IN: &str = "expires_in";
pub const V_QUEUE_LAST_STATUS: &str = "last_status";
pub const V_QUEUE_LAST_ERROR: &str = "last_error";
pub const V_URL: &str = "url";
pub const V_URL_PATH: &str = "url_path";
pub const V_HEADERS: &str = "headers";
pub const V_METHOD: &str = "method";

pub const CONNECTION_VARS: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
];
pub const HTTP_VARS: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
    V_URL,
    V_URL_PATH,
    V_HEADERS,
    V_METHOD,
];
pub const RCPT_DOMAIN_VARS: &[&str] = &[V_RECIPIENT_DOMAIN];
pub const SMTP_EHLO_VARS: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
    V_HELO_DOMAIN,
];
pub const SMTP_MAIL_FROM_VARS: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_AUTHENTICATED_AS,
];
pub const SMTP_RCPT_TO_VARS: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_RECIPIENTS,
    V_RECIPIENT,
    V_RECIPIENT_DOMAIN,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
    V_PRIORITY,
    V_HELO_DOMAIN,
];
pub const SMTP_QUEUE_HOST_VARS: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_RECIPIENT_DOMAIN,
    V_RECIPIENT,
    V_RECIPIENTS,
    V_MX,
    V_PRIORITY,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_QUEUE_RETRY_NUM,
    V_QUEUE_NOTIFY_NUM,
    V_QUEUE_EXPIRES_IN,
    V_QUEUE_LAST_STATUS,
    V_QUEUE_LAST_ERROR,
];
pub const SMTP_QUEUE_RCPT_VARS: &[&str] = &[
    V_RECIPIENT_DOMAIN,
    V_RECIPIENTS,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
    V_QUEUE_RETRY_NUM,
    V_QUEUE_NOTIFY_NUM,
    V_QUEUE_EXPIRES_IN,
    V_QUEUE_LAST_STATUS,
    V_QUEUE_LAST_ERROR,
];
pub const SMTP_QUEUE_SENDER_VARS: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
    V_QUEUE_RETRY_NUM,
    V_QUEUE_NOTIFY_NUM,
    V_QUEUE_EXPIRES_IN,
    V_QUEUE_LAST_STATUS,
    V_QUEUE_LAST_ERROR,
];
pub const SMTP_QUEUE_MX_VARS: &[&str] = &[
    V_RECIPIENT_DOMAIN,
    V_RECIPIENTS,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
    V_MX,
    V_QUEUE_RETRY_NUM,
    V_QUEUE_NOTIFY_NUM,
    V_QUEUE_EXPIRES_IN,
    V_QUEUE_LAST_STATUS,
    V_QUEUE_LAST_ERROR,
];
