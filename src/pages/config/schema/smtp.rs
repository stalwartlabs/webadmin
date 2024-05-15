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

use crate::core::{form::Expression, schema::*};

use super::*;

impl Builder<Schemas, ()> {
    pub fn build_smtp_outbound(self) -> Self {
        let rcpt_vars = ExpressionValidator::new(SMTP_QUEUE_RCPT_VARS, &[]);
        let sender_vars = ExpressionValidator::new(SMTP_QUEUE_SENDER_VARS, &[]);
        let mx_vars = ExpressionValidator::new(SMTP_QUEUE_MX_VARS, &[]);
        let host_vars = ExpressionValidator::new(SMTP_QUEUE_HOST_VARS, &[]);

        // Queue
        self.new_schema("smtp-out-queue")
            .new_field("queue.schedule.retry")
            .label("Retry")
            .help(concat!(
                "List of durations defining the schedule for retrying the ",
                "delivery of a message"
            ))
            .default("[2m, 5m, 10m, 15m, 30m, 1h, 2h]")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(host_vars)],
            )
            .new_field("queue.schedule.notify")
            .label("Notify")
            .help(concat!(
                "List of durations specifying when to notify the sender of ",
                "any delivery problems"
            ))
            .default("[1d, 3d]")
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .new_field("queue.schedule.expire")
            .label("Expire")
            .help(concat!(
                "Maximum duration that a message can remain in the queue before",
                " it expires and is returned to the sender"
            ))
            .default("5d")
            .build()
            .new_field("report.dsn.from-name")
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of Delivery Status ",
                "Notifications (DSN) reports"
            ))
            .default("'Mail Delivery Subsystem'")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(sender_vars),
                ],
            )
            .new_field("report.dsn.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "Delivery Status Notifications (DSN) reports"
            ))
            .default("'MAILER-DAEMON@' + key_get('default', 'domain')")
            .new_field("report.dsn.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing Delivery Status ",
                "Notifications"
            ))
            .default("['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]")
            .build()
            .new_form_section()
            .title("Queue Schedule")
            .fields([
                "queue.schedule.retry",
                "queue.schedule.notify",
                "queue.schedule.expire",
            ])
            .build()
            .new_form_section()
            .title("Delivery Status Notifications (DSN) Reports")
            .fields([
                "report.dsn.from-name",
                "report.dsn.from-address",
                "report.dsn.sign",
            ])
            .build()
            .build()
            // Routing
            .new_schema("smtp-out-routing")
            .new_field("queue.outbound.hostname")
            .label("EHLO Hostname")
            .help(concat!(
                "Overrides the default EHLO hostname used when sending messages",
                " to remote SMTP servers"
            ))
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(sender_vars)])
            .new_field("queue.outbound.next-hop")
            .label("Next hop")
            .help(concat!(
                "Can either point to a remote host or 'false' which indicates",
                " that the message delivery should be done through DNS resolution"
            ))
            .default(Expression::new([("is_local_domain('*', rcpt_domain)", "'local'")], "false"))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .new_field("queue.outbound.ip-strategy")
            .label("IP Strategy")
            .help(concat!(
                "Determines which type of IP address to use when delivering ",
                "emails to a remote SMTP server"
            ))
            .default("ipv4_then_ipv6")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(sender_vars.constants(IP_STRATEGY_CONSTANTS)),
                ],
            )
            .new_field("queue.outbound.source-ip.v4")
            .label("IPv4 addresses")
            .help(concat!(
                "Determines a list of local IPv4 addresses to use when ",
                "delivery emails to remote SMTP servers"
            ))
            .input_check([], [Validator::IsValidExpression(mx_vars)])
            .new_field("queue.outbound.source-ip.v6")
            .label("IPv6 addresses")
            .help(concat!(
                "Determines a list of local IPv6 addresses to use when ",
                "delivery emails to remote SMTP servers"
            ))
            .build()
            .new_form_section()
            .title("Routing")
            .fields([
                "queue.outbound.next-hop",
                "queue.outbound.ip-strategy",
                "queue.outbound.hostname",
            ])
            .build()
            .new_form_section()
            .title("Source IP Addresses")
            .fields(["queue.outbound.source-ip.v4", "queue.outbound.source-ip.v6"])
            .build()
            .build()
            // TLS
            .new_schema("smtp-out-tls")
            .new_field("queue.outbound.tls.dane")
            .label("DANE")
            .help(concat!("Whether DANE is required, optional, or disabled"))
            .default("optional")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(mx_vars.constants(REQUIRE_OPTIONAL_CONSTANTS)),
                ],
            )
            .new_field("queue.outbound.tls.starttls")
            .label("STARTTLS")
            .help(concat!(
                "Whether TLS support is required, optional, or disabled"
            ))
            .default("require")
            .new_field("queue.outbound.tls.mta-sts")
            .label("MTA-STS")
            .help(concat!(
                "Whether MTA-STS is required, optional, or disabled"
            ))
            .default("optional")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(REQUIRE_OPTIONAL_CONSTANTS)),
                ],
            )
            .new_field("queue.outbound.tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether to allow connections to servers with invalid TLS certificates"
            ))
            .default(Expression::new([("retry_num > 0 && last_error == 'tls'", "true")], "false"))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(mx_vars)],
            )
            .build()
            .new_field("report.tls.aggregate.from-name")
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of the TLS ",
                "aggregate report email"
            ))
            .default("'Report Subsystem'")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(sender_vars),
                ],
            )
            .new_field("report.tls.aggregate.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "the TLS aggregate report email"
            ))
            .default("'noreply-tls@' + key_get('default', 'domain')")
            .new_field("report.tls.aggregate.subject")
            .label("Subject")
            .help(concat!(
                "Subject name that will be used in the TLS aggregate report email"
            ))
            .default("'TLS Aggregate Report'")
            .new_field("report.tls.aggregate.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing the TLS ",
                "aggregate report"
            ))
            .default("['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]")
            .new_field("report.tls.aggregate.org-name")
            .label("Organization")
            .help(concat!(
                "Name of the organization to be included in the report"
            ))
            .default("key_get('default', 'domain')")
            .new_field("report.tls.aggregate.contact-info")
            .label("Contact")
            .help(concat!("Contact information to be included in the report"))
            .default("")
            .new_field("report.tls.aggregate.max-size")
            .label("Max Report Size")
            .help(concat!("Maximum size of the TLS aggregate report in bytes"))
            .default("26214400")
            .new_field("report.tls.aggregate.send")
            .label("Frequency")
            .help(concat!(
                "Frequency at which the TLS aggregate reports will be sent. The options ",
                "are hourly, daily, weekly, or never to disable reporting"
            ))
            .default("daily")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(sender_vars.constants(AGGREGATE_FREQ_CONSTANTS)),
                ],
            )
            .build()
            .new_form_section()
            .title("TLS Security")
            .fields([
                "queue.outbound.tls.starttls",
                "queue.outbound.tls.dane",
                "queue.outbound.tls.mta-sts",
                "queue.outbound.tls.allow-invalid-certs",
            ])
            .build()
            .new_form_section()
            .title("TLS Aggregate Reporting")
            .fields([
                "report.tls.aggregate.from-name",
                "report.tls.aggregate.from-address",
                "report.tls.aggregate.subject",
                "report.tls.aggregate.sign",
                "report.tls.aggregate.org-name",
                "report.tls.aggregate.contact-info",
                "report.tls.aggregate.max-size",
                "report.tls.aggregate.send",
            ])
            .build()
            .build()
            // Limits & Timeouts
            .new_schema("smtp-out-limits")
            .new_field("queue.outbound.limits.mx")
            .label("MX Hosts")
            .help(concat!(
                "Maximum number of MX hosts to try on each delivery attempt"
            ))
            .default("7")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .new_field("queue.outbound.limits.multihomed")
            .label("Multi-homed IPs")
            .help(concat!(
                "For multi-homed remote servers, it is the maximum number of ",
                "IP addresses to try on each delivery attempt"
            ))
            .default("2")
            .new_field("queue.outbound.timeouts.connect")
            .label("Connect")
            .help(concat!(
                "Maximum time to wait for the connection to be established"
            ))
            .default("3m")
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(host_vars)],
            )
            .new_field("queue.outbound.timeouts.greeting")
            .label("Greeting")
            .help(concat!(
                "Maximum time to wait for the SMTP greeting message"
            ))
            .default("3m")
            .new_field("queue.outbound.timeouts.tls")
            .label("TLS Handshake")
            .help(concat!(
                "Maximum time to wait for the TLS handshake to complete"
            ))
            .default("2m")
            .new_field("queue.outbound.timeouts.ehlo")
            .label("EHLO command")
            .help(concat!(
                "Maximum time to wait for the EHLO command response"
            ))
            .default("3m")
            .new_field("queue.outbound.timeouts.mail-from")
            .label("MAIL-FROM command")
            .help(concat!(
                "Maximum time to wait for the MAIL-FROM command response"
            ))
            .default("3m")
            .new_field("queue.outbound.timeouts.rcpt-to")
            .label("RCPT-TO command")
            .help(concat!(
                "Maximum time to wait for the RCPT-TO command response"
            ))
            .default("3m")
            .new_field("queue.outbound.timeouts.data")
            .label("DATA command")
            .help(concat!(
                "Maximum time to wait for the DATA command response"
            ))
            .default("10m")
            .new_field("queue.outbound.timeouts.mta-sts")
            .label("MTA-STS lookup")
            .help(concat!(
                "Maximum time to wait for the MTA-STS policy lookup to complete"
            ))
            .default("2m")
            .build()
            .new_form_section()
            .title("Limits")
            .fields([
                "queue.outbound.limits.mx",
                "queue.outbound.limits.multihomed",
            ])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields([
                "queue.outbound.timeouts.connect",
                "queue.outbound.timeouts.greeting",
                "queue.outbound.timeouts.tls",
                "queue.outbound.timeouts.ehlo",
                "queue.outbound.timeouts.mail-from",
                "queue.outbound.timeouts.rcpt-to",
                "queue.outbound.timeouts.data",
                "queue.outbound.timeouts.mta-sts",
            ])
            .build()
            .build()
            // Resolver
            .new_schema("smtp-out-resolver")
            .new_field("resolver.type")
            .label("Resolver")
            .help(concat!("Resolver to use for DNS resolution"))
            .default("system")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("system", "System Resolver"),
                    ("custom", "Custom DNS"),
                    ("cloudflare", "Cloudflare DNS "),
                    ("cloudflare-tls", "Cloudflare DNS (TLS)"),
                    ("quad9", "Quad9 DNS"),
                    ("quad9-tls", "Quad9 DNS (TLS)"),
                    ("google", "Google DNS"),
                ]),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.custom")
            .label("DNS Servers")
            .help(concat!("List of custom DNS server URLs to use for resolution"))
            .default("udp://127.0.0.1:53")
            .typ(Type::Array)
            .input_check([], [Validator::Required])
            .display_if_eq("resolver.type", ["custom"])
            .build()
            .new_field("resolver.preserve-intermediates")
            .label("Preserve Intermediates")
            .help(concat!(
                "Whether to preserve the intermediate name servers in the ",
                "DNS resolution results"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.concurrency")
            .label("Concurrent Requests")
            .help(concat!(
                "Number of concurrent resolution requests that can be made ",
                "at the same time"
            ))
            .default("2")
            .typ(Type::Input)
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.timeout")
            .label("Timeout")
            .help(concat!(
                "Time after which a resolution request will be timed out if ",
                "no response is received"
            ))
            .default("5s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.attempts")
            .label("Max Attempts")
            .help(concat!(
                "Number of times a resolution request will be retried before ",
                "it is considered failed"
            ))
            .default("2")
            .typ(Type::Input)
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.try-tcp-on-error")
            .label("Try TCP on Error")
            .help(concat!(
                "Whether to try using TCP for resolution requests if an error ",
                "occurs during a UDP resolution request"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("resolver.public-suffix")
            .label("Public Suffix list")
            .help(concat!(
                "URL of the list of top-level domain names (or suffixes) under ",
                "which Internet users can register domain names"
            ))
            .default(&[
                "https://publicsuffix.org/list/public_suffix_list.dat",
                "https://raw.githubusercontent.com/publicsuffix/list/master/public_suffix_list.dat"
            ][..])
            .typ(Type::Array)
            .input_check([], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("cache.resolver.txt.size")
            .label("TXT Records")
            .help(concat!("Number of TXT records to cache"))
            .default("2048")
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1.into())])
            .new_field("cache.resolver.mx.size")
            .label("MX Records")
            .help(concat!("Number of MX records to cache"))
            .default("1024")
            .typ(Type::Input)
            .new_field("cache.resolver.ipv4.size")
            .label("IPv4 Records")
            .help(concat!("Number of IPv4 records to cache"))
            .default("1024")
            .typ(Type::Input)
            .new_field("cache.resolver.ipv6.size")
            .label("IPv6 Records")
            .help(concat!("Number of IPv6 records to cache"))
            .default("1024")
            .typ(Type::Input)
            .new_field("cache.resolver.ptr.size")
            .label("PTR Records")
            .help(concat!("Number of PTR records to cache"))
            .default("1024")
            .typ(Type::Input)
            .new_field("cache.resolver.tlsa.size")
            .label("TLSA Records")
            .help(concat!("Number of TLSA records to cache"))
            .default("1024")
            .typ(Type::Input)
            .new_field("cache.resolver.mta-sts.size")
            .label("MTA-STS Records")
            .help(concat!("Number of MTA-STS records to cache"))
            .default("1024")
            .typ(Type::Input)
            .build()
            .new_form_section()
            .title("DNS Resolver settings")
            .fields([
                "resolver.type",
                "resolver.custom",
                "resolver.concurrency",
                "resolver.timeout",
                "resolver.attempts",
                "resolver.public-suffix",
                "resolver.preserve-intermediates",
                "resolver.try-tcp-on-error",
            ])
            .build()
            .new_form_section()
            .title("DNS Record Cache")
            .fields([
                "cache.resolver.txt.size",
                "cache.resolver.mx.size",
                "cache.resolver.ipv4.size",
                "cache.resolver.ipv6.size",
                "cache.resolver.ptr.size",
                "cache.resolver.tlsa.size",
                "cache.resolver.mta-sts.size",
            ])
            .build()
            .build()
            // Remote hosts
            .new_schema("smtp-out-remote")
            .prefix("remote")
            .suffix("address")
            .names("host", "hosts")
            .new_id_field()
            .label("Host ID")
            .help("Unique identifier for the remote host")
            .build()
            .new_field("address")
            .label("Address")
            .help(concat!(
                "The address of the remote SMTP server, which can be an IP ",
                "address or a domain name"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::IsHost])
            .placeholder("127.0.0.1")
            .build()
            .new_field("port")
            .label("Port")
            .help(concat!(
                "The port number of the remote server, which is typically ",
                "25 for SMTP and 11200 for LMTP"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::IsPort])
            .placeholder("25")
            .build()
            .new_field("protocol")
            .label("Protocol")
            .help(concat!(
                "The protocol to use when delivering messages to the remote ",
                "server, which can be either SMTP or LMTP"
            ))
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[("smtp", "SMTP"), ("lmtp", "LMTP")]),
            })
            .default("smtp")
            .build()
            .new_field("tls.implicit")
            .label("Implicit TLS")
            .help(concat!(
                "Whether to use TLS encryption for all connections to the remote ",
                "server"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether to allow connections to servers with invalid TLS certificates"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("auth.username")
            .label("Username")
            .help(concat!(
                "The username to use when authenticating with the remote server"
            ))
            .typ(Type::Input)
            .build()
            .new_field("auth.secret")
            .label("Secret")
            .help(concat!(
                "The secret to use when authenticating with the remote server"
            ))
            .typ(Type::Secret)
            .build()
            .new_form_section()
            .title("Server Details")
            .fields(["_id", "address", "port", "protocol"])
            .build()
            .new_form_section()
            .title("TLS")
            .fields(["tls.implicit", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["auth.username", "auth.secret"])
            .build()
            .list_title("Remote SMTP Servers")
            .list_subtitle("Manage remote SMTP and LMTP servers for message delivery")
            .list_fields(["_id", "protocol", "address", "port"])
            .build()
            // Outbound throttle
            .new_schema("smtp-out-throttle")
            .prefix("queue.throttle")
            .names("throttle", "throttles")
            .suffix("enable")
            .new_id_field()
            .label("Throttle ID")
            .help("Unique identifier for the throttle")
            .build()
            .new_field("enable")
            .label("Enabled")
            .help("Whether to enable this throttle")
            .typ(Type::Boolean)
            .default("true")
            .build()
            .new_field("key")
            .label("Keys")
            .help(concat!(
                "Optional list of context variables that determine ",
                "where this throttle should be applied"
            ))
            .typ(Type::Select {
                multi: true,
                source: Source::Static(&[
                    (V_MX, "MX Host"),
                    (V_REMOTE_IP, "Remote IP"),
                    (V_LOCAL_IP, "Local IP"),
                    (V_SENDER, "Sender"),
                    (V_SENDER_DOMAIN, "Sender Domain"),
                    (V_RECIPIENT_DOMAIN, "Recipient Domain"),
                ]),
            })
            .build()
            .new_field("match")
            .label("Match condition")
            .help(concat!(
                "Enable the imposition of concurrency and rate limits only ",
                "when a specific condition is met"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::IsValidExpression(ExpressionValidator::new(
                        SMTP_QUEUE_HOST_VARS,
                        &[],
                    )),
                    Validator::MaxItems(1),
                ],
            )
            .build()
            .new_field("concurrency")
            .label("Concurrency")
            .help(concat!(
                "Maximum number of concurrent connections that ",
                "the throttle will allow"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("rate")
            .label("Rate limit")
            .help(concat!(
                "Number of incoming requests over a period of time ",
                "that the rate limiter will allow"
            ))
            .typ(Type::Rate)
            .build()
            .new_form_section()
            .title("Throttle")
            .fields(["_id", "key", "concurrency", "rate", "match", "enable"])
            .build()
            .list_title("Outbound Throttles")
            .list_subtitle("Manage outbound concurrency and rate limits")
            .list_fields(["_id", "concurrency", "rate", "enable"])
            .build()
            // Queue quotas
            .new_schema("smtp-out-quota")
            .prefix("queue.quota")
            .names("quota", "quotas")
            .suffix("enable")
            .new_id_field()
            .label("Quota ID")
            .help("Unique identifier for the quota")
            .build()
            .new_field("enable")
            .label("Enabled")
            .help("Whether to enable this quota")
            .typ(Type::Boolean)
            .default("true")
            .build()
            .new_field("key")
            .label("Keys")
            .help(concat!(
                "Optional list of context variables that determine ",
                "where this quota should be applied"
            ))
            .typ(Type::Select {
                multi: true,
                source: Source::Static(&[
                    (V_SENDER, "Sender"),
                    (V_SENDER_DOMAIN, "Sender Domain"),
                    (V_RECIPIENT, "Recipient"),
                    (V_RECIPIENT_DOMAIN, "Recipient Domain"),
                ]),
            })
            .build()
            .new_field("match")
            .label("Match condition")
            .help(concat!(
                "Enable the imposition of concurrency and rate limits only ",
                "when a specific condition is met"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::IsValidExpression(ExpressionValidator::new(
                        SMTP_QUEUE_HOST_VARS,
                        &[],
                    )),
                    Validator::MaxItems(1),
                ],
            )
            .build()
            .new_field("messages")
            .label("Max Messages")
            .help(concat!(
                "Maximum number of messages in the queue that ",
                "this quota will allow"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("size")
            .label("Max Size")
            .help(concat!(
                "Maximum total size of messages in the queue that ",
                "this quota will allow"
            ))
            .typ(Type::Size)
            .build()
            .new_form_section()
            .title("Quota")
            .fields(["_id", "key", "messages", "size", "match", "enable"])
            .build()
            .list_title("Quota Queues")
            .list_subtitle("Manage quotas on message queues")
            .list_fields(["_id", "messages", "size", "enable"])
            .build()
    }

    pub fn build_smtp_inbound(self) -> Self {
        let has_conn_vars = ExpressionValidator::new(CONNECTION_VARS, &[]);
        let has_ehlo_hars = ExpressionValidator::new(SMTP_EHLO_VARS, &[]);
        let has_sender_vars = ExpressionValidator::new(SMTP_MAIL_FROM_VARS, &[]);
        let has_rcpt_vars = ExpressionValidator::new(SMTP_RCPT_TO_VARS, &[]);

        // Connect
        self.new_schema("smtp-in-connect")
            .new_field("session.connect.script")
            .typ(Type::Expression)
            .label("Run Script")
            .help("Which Sieve script to run when a client connects")
            .input_check([], [Validator::IsValidExpression(has_conn_vars)])
            .new_field("session.connect.greeting")
            .label("SMTP greeting")
            .help("The greeting message sent by the SMTP/LMTP server")
            .default("'Stalwart ESMTP at your service'")
            .new_field("session.connect.hostname")
            .label("Server hostname")
            .help("The SMTP server hostname")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_conn_vars),
                ],
            )
            .default("key_get('default', 'hostname')")
            .build()
            .new_field("auth.iprev.verify")
            .typ(Type::Expression)
            .label("IPRev Verify")
            .help("How strict to be when verifying the reverse DNS of the client IP")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_conn_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .default(Expression::new(
                [("local_port == 25", "relaxed")],
                "disable",
            ))
            .build()
            .new_form_section()
            .title("Connect Stage")
            .fields([
                "session.connect.hostname",
                "session.connect.greeting",
                "session.connect.script",
                "auth.iprev.verify",
            ])
            .build()
            .build()
            // EHLO stage
            .new_schema("smtp-in-ehlo")
            .new_field("session.ehlo.require")
            .label("Require EHLO")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_conn_vars),
                ],
            )
            .default("true")
            .help(concat!(
                "Whether the remote client must send an EHLO command ",
                "before starting an SMTP transaction"
            ))
            .build()
            .new_field("session.ehlo.reject-non-fqdn")
            .label("Reject Non-FQDN")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_conn_vars),
                ],
            )
            .help(concat!(
                "Whether to reject EHLO commands that do not include a ",
                "fully-qualified domain name as a parameter"
            ))
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .build()
            .new_field("session.ehlo.script")
            .label("Run Script")
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(has_conn_vars)])
            .help("Which Sieve script to run after the client sends an EHLO command")
            .build()
            .new_form_section()
            .title("EHLO Stage")
            .fields([
                "session.ehlo.require",
                "session.ehlo.reject-non-fqdn",
                "session.ehlo.script",
            ])
            .build()
            .build()
            // Limits
            .new_schema("smtp-in-limits")
            .new_field("session.timeout")
            .label("Timeout")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_conn_vars),
                ],
            )
            .default("5m")
            .help("How long to wait for a client to send a command before timing out")
            .new_field("session.transfer-limit")
            .label("Bytes Limit")
            .default("262144000")
            .help("The maximum number of bytes that can be transferred per session")
            .new_field("session.duration")
            .label("Duration")
            .default("10m")
            .help("The maximum duration of a session")
            .build()
            .new_form_section()
            .title("SMTP Session Limits")
            .fields([
                "session.timeout",
                "session.transfer-limit",
                "session.duration",
            ])
            .build()
            .build()
            // Extensions
            .new_schema("smtp-in-extensions")
            .new_field("session.extensions.pipelining")
            .label("Pipelining")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_sender_vars),
                ],
            )
            .default("true")
            .help(concat!(
                "Enables SMTP pipelining (RFC 2920), which enables multiple ",
                "commands to be sent in a single request to speed up communication ",
                "between the client and server"
            ))
            .new_field("session.extensions.chunking")
            .label("Chunking")
            .help(concat!(
                "Enables chunking (RFC 1830), an extension that allows large ",
                "messages to be transferred in chunks which may reduce the load ",
                "on the network and server."
            ))
            .default("true")
            .new_field("session.extensions.requiretls")
            .label("Require TLS")
            .help(concat!(
                "Enables require TLS (RFC 8689), an extension that allows",
                " clients to require TLS encryption for the SMTP session"
            ))
            .default("true")
            .new_field("session.extensions.no-soliciting")
            .label("No-Soliciting")
            .help(concat!(
                "Specifies the text to include in the NOSOLICITING (RFC 3865) ",
                "message, which indicates that the server does not accept unsolicited ",
                "commercial email (UCE or spam)"
            ))
            .default("\"\"")
            .new_field("session.extensions.dsn")
            .label("DSN")
            .help(concat!(
                "Enables delivery status notifications (RFC 3461), which allows ",
                "the sender to request a delivery status notification (DSN) from ",
                "the recipient's mail server"
            ))
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "true")],
                "false",
            ))
            .new_field("session.extensions.expn")
            .label("EXPN")
            .help(concat!(
                "Specifies whether to enable the EXPN command, which allows ",
                "the sender to request the membership of a mailing list. It is ",
                "recommended to disable this command to prevent spammers ",
                "from harvesting email addresses"
            ))
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "true")],
                "false",
            ))
            .new_field("session.extensions.vrfy")
            .label("VRFY")
            .help(concat!(
                "Specifies whether to enable the VRFY command, which allows ",
                "the sender to verify the existence of a mailbox. It is recommended ",
                "to disable this command to prevent spammers from ",
                "harvesting email addresses"
            ))
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "true")],
                "false",
            ))
            .new_field("session.extensions.future-release")
            .label("Future Release")
            .help(concat!(
                "Specifies the maximum time that a message can be held for ",
                "delivery using the FUTURERELEASE (RFC 4865) extension"
            ))
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "7d")],
                "false",
            ))
            .new_field("session.extensions.deliver-by")
            .label("Deliver By")
            .help(concat!(
                "Specifies the maximum delivery time for a message using the ",
                "DELIVERBY (RFC 2852) extension, which allows the sender to request ",
                "a specific delivery time for a message"
            ))
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "15d")],
                "false",
            ))
            .new_field("session.extensions.mt-priority")
            .label("MT Priority")
            .help(concat!(
                "Specifies the priority assignment policy to advertise on the ",
                "MT-PRIORITY (RFC 6710) extension, which allows the sender to specify ",
                "a priority for a message. Available policies are mixer, stanag4406 a",
                "nd nsep, or false to disable this extension"
            ))
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_sender_vars.constants(&[
                        "mixer",
                        "stanag4406",
                        "nsep",
                    ])),
                ],
            )
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "mixer")],
                "false",
            ))
            .build()
            .new_form_section()
            .title("SMTP Extensions")
            .fields([
                "session.extensions.pipelining",
                "session.extensions.chunking",
                "session.extensions.requiretls",
                "session.extensions.no-soliciting",
                "session.extensions.dsn",
                "session.extensions.expn",
                "session.extensions.vrfy",
                "session.extensions.future-release",
                "session.extensions.deliver-by",
                "session.extensions.mt-priority",
            ])
            .build()
            .build()
            // AUTH stage
            .new_schema("smtp-in-auth")
            .new_field("session.auth.require")
            .label("Require Authentication")
            .help(concat!(
                "Specifies whether authentication is necessary to send email messages"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_ehlo_hars),
                ],
            )
            .default(Expression::new([("local_port != 25", "true")], "false"))
            .new_field("session.auth.must-match-sender")
            .label("Must match sender")
            .help(concat!(
                "Specifies whether the authenticated user or any of their associated ",
                "e-mail addresses must match the sender of the email message"
            ))
            .default("true")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_sender_vars),
                ],
            )
            .new_field("session.auth.directory")
            .label("Directory")
            .help("Specifies the directory to use for authentication")
            .default(Expression::new([("local_port != 25", "'*'")], "false"))
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_ehlo_hars),
                ],
            )
            .new_field("session.auth.errors.total")
            .label("Max Errors")
            .help(concat!(
                "Maximum number of authentication errors allowed before the session ",
                "is disconnected"
            ))
            .default("3")
            .new_field("session.auth.errors.wait")
            .label("Error wait")
            .help("Time interval to wait after an authentication failure")
            .default("5s")
            .new_field("session.auth.mechanisms")
            .label("Allowed Mechanisms")
            .help(concat!(
                "A list of SASL authentication mechanisms offered to clients, or an ",
                "empty list to disable authentication. Stalwart SMTP currently supports PLAIN, ",
                "LOGIN, and OAUTHBEARER mechanisms"
            ))
            .default(Expression::new(
                [("local_port != 25 && is_tls", "[plain, login]")],
                "false",
            ))
            .input_check(
                [],
                [Validator::IsValidExpression(
                    has_conn_vars.constants(AUTH_CONSTANTS),
                )],
            )
            .build()
            .new_form_section()
            .title("AUTH Stage")
            .fields([
                "session.auth.directory",
                "session.auth.require",
                "session.auth.must-match-sender",
                "session.auth.mechanisms",
            ])
            .build()
            .new_form_section()
            .title("Authentication Errors")
            .fields(["session.auth.errors.total", "session.auth.errors.wait"])
            .build()
            .build()
            // MAIL stage
            .new_schema("smtp-in-mail")
            .new_field("session.mail.rewrite")
            .label("Sender Rewriting")
            .help("Expression to rewrite the sender address")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_sender_vars),
                ],
            )
            .default("false")
            .new_field("session.mail.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a MAIL command")
            .input_check([], [Validator::IsValidExpression(has_sender_vars)])
            .build()
            .new_form_section()
            .title("MAIL FROM Stage")
            .fields(["session.mail.rewrite", "session.mail.script"])
            .build()
            .build()
            // RCPT stage
            .new_schema("smtp-in-rcpt")
            .new_field("session.rcpt.directory")
            .label("Directory")
            .help("Directory to use to validate local recipients")
            .default("\"*\"")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_rcpt_vars),
                ],
            )
            .new_field("session.rcpt.relay")
            .label("Allow Relaying")
            .help("Whether to allow relaying for non-local recipients")
            .default(Expression::new(
                [("!is_empty(authenticated_as)", "true")],
                "false",
            ))
            .new_field("session.rcpt.max-recipients")
            .label("Max Recipients")
            .help("Maximum number of recipients per message")
            .default("25")
            .new_field("session.rcpt.rewrite")
            .label("Recipient Rewriting")
            .help("Expression to rewrite the recipient address")
            .default("false")
            .new_field("session.rcpt.errors.total")
            .label("Max Errors")
            .help(concat!(
                "Maximum number of recipient errors before ",
                "the session is disconnected"
            ))
            .default("5")
            .new_field("session.rcpt.errors.wait")
            .label("Error wait")
            .help("Amount of time to wait after a recipient error")
            .default("5s")
            .new_field("session.rcpt.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a RCPT command")
            .input_check([], [Validator::IsValidExpression(has_rcpt_vars)])
            .build()
            .new_field("session.rcpt.catch-all")
            .label("Catch-all")
            .help("Expression to enable catch-all address")
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(has_rcpt_vars)])
            .default("true")
            .new_field("session.rcpt.sub-addressing")
            .label("Sub-addressing")
            .help("Expression to enable sub-addressing")
            .default("true")
            .build()
            .new_form_section()
            .title("RCPT TO Stage")
            .fields([
                "session.rcpt.directory",
                "session.rcpt.relay",
                "session.rcpt.max-recipients",
                "session.rcpt.script",
            ])
            .build()
            .new_form_section()
            .title("Address Handling")
            .fields([
                "session.rcpt.rewrite",
                "session.rcpt.catch-all",
                "session.rcpt.sub-addressing",
            ])
            .build()
            .new_form_section()
            .title("Recipient Errors")
            .fields(["session.rcpt.errors.total", "session.rcpt.errors.wait"])
            .build()
            .build()
            // DATA stage
            .new_schema("smtp-in-data")
            .new_field("session.data.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a DATA command")
            .default(Expression::new(
                [("is_empty(authenticated_as)", "'spam-filter'")],
                "'track-replies'",
            ))
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(has_rcpt_vars)])
            .new_field("session.data.limits.messages")
            .label("Messages")
            .help("Maximum number of messages that can be submitted per SMTP session")
            .default("10")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_rcpt_vars),
                ],
            )
            .new_field("session.data.limits.size")
            .label("Size")
            .help("Maximum size of a message in bytes")
            .default("104857600")
            .new_field("session.data.limits.received-headers")
            .label("Received Headers")
            .help(concat!(
                "Maximum limit on the number of Received headers, ",
                "which helps to prevent message loops"
            ))
            .default("50")
            .new_field("session.data.add-headers.received")
            .label("Received")
            .help("Whether to add a Received header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .new_field("session.data.add-headers.received-spf")
            .label("Received-SPF")
            .help("Whether to add a Received-SPF header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .new_field("session.data.add-headers.auth-results")
            .label("Authentication-Results")
            .help("Whether to add an Authentication-Results header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .new_field("session.data.add-headers.message-id")
            .label("Message-Id")
            .help("Whether to add a Message-Id header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .new_field("session.data.add-headers.date")
            .label("Date")
            .help("Whether to add a Date header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .new_field("session.data.add-headers.return-path")
            .label("Return-Path")
            .help("Whether to add a Return-Path header to the message")
            .default(Expression::new([("local_port == 25", "true")], "false"))
            .build()
            .new_form_section()
            .title("DATA Stage")
            .fields(["session.data.script"])
            .build()
            .new_form_section()
            .title("Limits")
            .fields([
                "session.data.limits.messages",
                "session.data.limits.size",
                "session.data.limits.received-headers",
            ])
            .build()
            .new_form_section()
            .title("Add Headers")
            .fields([
                "session.data.add-headers.received",
                "session.data.add-headers.received-spf",
                "session.data.add-headers.auth-results",
                "session.data.add-headers.message-id",
                "session.data.add-headers.date",
                "session.data.add-headers.return-path",
            ])
            .build()
            .build()
            // Throttle
            .new_schema("smtp-in-throttle")
            .prefix("session.throttle")
            .names("throttle", "throttles")
            .suffix("enable")
            .new_id_field()
            .label("Throttle ID")
            .help("Unique identifier for the throttle")
            .build()
            .new_field("enable")
            .label("Enabled")
            .help("Whether to enable this throttle")
            .typ(Type::Boolean)
            .default("true")
            .build()
            .new_field("key")
            .label("Keys")
            .help(concat!(
                "Optional list of context variables that determine ",
                "where this throttle should be applied"
            ))
            .typ(Type::Select {
                multi: true,
                source: Source::Static(&[
                    (V_LISTENER, "Listener"),
                    (V_REMOTE_IP, "Remote IP"),
                    (V_LOCAL_IP, "Local IP"),
                    (V_AUTHENTICATED_AS, "Authenticated As"),
                    (V_HELO_DOMAIN, "EHLO Domain"),
                    (V_SENDER, "Sender"),
                    (V_SENDER_DOMAIN, "Sender Domain"),
                    (V_RECIPIENT, "Recipient"),
                    (V_RECIPIENT_DOMAIN, "Recipient Domain"),
                ]),
            })
            .build()
            .new_field("match")
            .label("Match condition")
            .help(concat!(
                "Enable the imposition of concurrency and rate limits only ",
                "when a specific condition is met"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::IsValidExpression(has_rcpt_vars),
                    Validator::MaxItems(1),
                ],
            )
            .build()
            .new_field("concurrency")
            .label("Concurrency")
            .help(concat!(
                "Maximum number of concurrent connections that ",
                "the throttle will allow"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("rate")
            .label("Rate limit")
            .help(concat!(
                "Number of incoming requests over a period of time ",
                "that the rate limiter will allow"
            ))
            .typ(Type::Rate)
            .build()
            .new_form_section()
            .title("Throttle")
            .fields(["_id", "key", "concurrency", "rate", "match", "enable"])
            .build()
            .list_title("Inbound Throttles")
            .list_subtitle("Manage inbound concurrency and rate limits")
            .list_fields(["_id", "concurrency", "rate", "enable"])
            .build()
            // Milter
            .new_schema("milter")
            .prefix("session.data.milter")
            .suffix("hostname")
            .names("milter", "milters")
            .new_id_field()
            .label("Milter Id")
            .help("Unique identifier for this milter")
            .build()
            .new_field("enable")
            .label("Enable")
            .help("Expression that determines whether to enable this milter")
            .default("true")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_rcpt_vars),
                ],
            )
            .build()
            .new_field("hostname")
            .label("Hostname")
            .help(concat!(
                "Hostname or IP address of the server where the Milter ",
                "filter is running"
            ))
            .placeholder("127.0.0.1")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            .build()
            .new_field("port")
            .label("Port")
            .help("Network port on the Milter filter host server")
            .placeholder("11332")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .build()
            .new_field("tls")
            .label("Enable TLS")
            .help(concat!(
                "Whether to use Transport Layer Security (TLS) for the connection ",
                "between Stalwart SMTP and the Milter filter"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether Stalwart SMTP should accept connections to a Milter filter ",
                "server that has an invalid TLS certificate"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.connect")
            .label("Connection")
            .help(concat!(
                "Maximum amount of time that Stalwart SMTP will wait to establish ",
                "a connection with a Milter server"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.command")
            .label("Command")
            .help(concat!(
                "How long Stalwart SMTP will wait to send a command to the ",
                "Milter server"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.data")
            .label("Data")
            .help(concat!(
                "Maximum amount of time Stalwart SMTP will wait for a response",
                " from the Milter server"
            ))
            .default("60s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.tempfail-on-error")
            .label("TempFail on Error")
            .help(concat!(
                "Whether to respond with a temporary failure (typically a 4xx ",
                "SMTP status code) when Stalwart encounters an error while ",
                "communicating with a Milter server"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.max-response-size")
            .label("Max Response")
            .help(concat!(
                "Maximum size, in bytes, of a response that Stalwart will accept",
                " from a Milter server"
            ))
            .default("52428800")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.version")
            .label("Protocol Version")
            .help(concat!(
                "Version of the Milter protocol that Stalwart SMTP should use when",
                " communicating with the Milter server"
            ))
            .default("6")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[("2", "Version 2"), ("6", "Version 6")]),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Milter settings")
            .fields(["_id", "hostname", "port", "enable"])
            .build()
            .new_form_section()
            .title("TLS")
            .fields(["tls", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Options")
            .fields([
                "options.max-response-size",
                "options.version",
                "options.tempfail-on-error",
            ])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields(["timeout.connect", "timeout.command", "timeout.data"])
            .build()
            .list_title("Milter filters")
            .list_subtitle("Manage Milter filters")
            .list_fields(["_id", "hostname", "port"])
            .build()
            // Pipes
            .new_schema("pipe")
            .names("pipe", "pipes")
            .prefix("session.data.pipe")
            .suffix("command")
            .new_id_field()
            .label("Pipe Id")
            .help("Unique identifier for this pipe")
            .build()
            .new_field("command")
            .label("Command")
            .help("Command name to execute")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(has_rcpt_vars),
                ],
            )
            .new_field("arguments")
            .label("Arguments")
            .help("Arguments to pass to the command")
            .default("[]")
            .new_field("timeout")
            .label("Timeout")
            .help("Maximum time to wait for the command to complete")
            .default("30s")
            .build()
            .new_form_section()
            .title("Pipe settings")
            .fields(["_id", "command", "arguments", "timeout"])
            .build()
            .list_title("Pipes")
            .list_subtitle("Manage external filters (pipes)")
            .list_fields(["_id", "command", "arguments"])
            .build()
            // MTA-STS
            .new_schema("smtp-in-mta-sts")
            .new_field("session.mta-sts.mode")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("enforce", "Enforce"),
                    ("testing", "Testing"),
                    ("none", "None"),
                    ("disable", "Disabled"),
                ]),
            })
            .input_check([], [Validator::Required])
            .label("Policy Application")
            .help("Whether to enforce, test, or disable the MTA-STS policy")
            .default("testing")
            .build()
            .new_field("session.mta-sts.max-age")
            .label("Max lifetime")
            .typ(Type::Duration)
            .help("Maximum time to cache the MTA-STS policy")
            .default("7d")
            .input_check([], [Validator::Required])
            .build()
            .new_field("session.mta-sts.mx")
            .label("MX Patterns (override)")
            .help(concat!(
                "Override the allowed MX hosts for the MTA-STS policy domain. ",
                "If empty, the MX hosts are determined from the available TLS certificates"
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("MTA-STS Policy")
            .fields([
                "session.mta-sts.mode",
                "session.mta-sts.max-age",
                "session.mta-sts.mx",
            ])
            .build()
            .build()
    }
}

pub const VERIFY_CONSTANTS: &[&str] =
    &["relaxed", "strict", "disable", "disabled", "never", "none"];
pub const AUTH_CONSTANTS: &[&str] = &["plain", "login", "xoauth2", "oauthbearer"];
pub const IP_STRATEGY_CONSTANTS: &[&str] =
    &["ipv4_only", "ipv6_only", "ipv6_then_ipv4", "ipv4_then_ipv6"];
pub const REQUIRE_OPTIONAL_CONSTANTS: &[&str] = &[
    "optional", "require", "required", "disable", "disabled", "none", "false",
];
pub const AGGREGATE_FREQ_CONSTANTS: &[&str] = &[
    "daily", "day", "hourly", "hour", "weekly", "week", "never", "disable", "false",
];
