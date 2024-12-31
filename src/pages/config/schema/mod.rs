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
pub mod spamfilter;
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
pub const V_ASN: &str = "asn";
pub const V_COUNTRY: &str = "country";
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
    V_ASN,
    V_COUNTRY,
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
    V_ASN,
    V_COUNTRY,
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
    V_ASN,
    V_COUNTRY,
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
    V_ASN,
    V_COUNTRY,
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

pub const SPAM_FILTER_VARS: &[&str] = &[
    "address",
    "email",
    "rcpt",
    // Spam-related variables
    "remote_ip",
    "remote_ip.ptr",
    "ehlo_domain",
    "auth_as",
    "asn",
    "country",
    "is_tls",
    "env_from",
    "env_from.local",
    "env_from.domain",
    "env_to",
    "from",
    "from.name",
    "from.local",
    "from.domain",
    "reply_to",
    "reply_to.name",
    "reply_to.local",
    "reply_to.domain",
    "to",
    "to.name",
    "to.local",
    "to.domain",
    "cc",
    "cc.name",
    "cc.local",
    "cc.domain",
    "bcc",
    "bcc.name",
    "bcc.local",
    "bcc.domain",
    "body",
    "body.text",
    "body.html",
    "body.raw",
    "subject",
    "subject.thread",
    "location",
    // URL-related variables
    "url",
    "path_query",
    "path",
    "query",
    "scheme",
    "authority",
    "host",
    "sld",
    "port",
    // Email-related variables
    "email",
    "value",
    "name",
    "local",
    "domain",
    // IP-related variables
    "ip",
    "reverse_ip",
    "ip_reverse",
    "octets",
    "is_v4",
    "is_v6",
    // Header-related variables
    "name",
    "name_lower",
    "value",
    "value_lower",
    "email_lower",
    "attributes",
    "raw",
    "raw_lower",
    // Body-related variables
    "input",
    "result",
];
