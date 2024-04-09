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

pub const CONNECTION_VARS: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_REMOTE_PORT,
    V_LOCAL_IP,
    V_LOCAL_PORT,
    V_PROTOCOL,
    V_TLS,
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
];
pub const SMTP_QUEUE_RCPT_VARS: &[&str] = &[
    V_RECIPIENT_DOMAIN,
    V_RECIPIENTS,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
];
pub const SMTP_QUEUE_SENDER_VARS: &[&str] = &[V_SENDER, V_SENDER_DOMAIN, V_PRIORITY];
pub const SMTP_QUEUE_MX_VARS: &[&str] = &[
    V_RECIPIENT_DOMAIN,
    V_RECIPIENTS,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
    V_MX,
];
