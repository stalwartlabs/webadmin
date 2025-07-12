/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::{form::Expression, schema::*};

use super::*;

impl Builder<Schemas, ()> {
    pub fn build_smtp_outbound(self) -> Self {
        const REQUIRE_OPTIONAL: &[(&str, &str)] = &[
            ("optional", "Optional"),
            ("require", "Required"),
            ("disable", "Disabled"),
        ];

        let rcpt_vars = ExpressionValidator::new(SMTP_QUEUE_RCPT_VARS, &[]);
        let host_vars = ExpressionValidator::new(SMTP_QUEUE_HOST_VARS, &[]);

        // Strategies
        self.new_schema("smtp-out-strategy")
            .new_field("queue.strategy.route")
            .label("Routing")
            .help(concat!(
                "An expression that returns the route name to use ",
                "when delivering queued messages"
            ))
            .default(Expression::new(
                [("is_local_domain('*', rcpt_domain)", "'local'")],
                "'mx'",
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(host_vars)],
            )
            .new_field("queue.strategy.schedule")
            .label("Scheduling")
            .help(concat!(
                "An expression that returns the scheduling strategy to use ",
                "when queueing messages"
            ))
            .default(Expression::new(
                [
                    ("is_local_domain('*', rcpt_domain)", "'local'"),
                    ("source == 'dsn'", "'dsn'"),
                    ("source == 'report'", "'report'"),
                ],
                "'remote'",
            ))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .new_field("queue.strategy.connection")
            .label("Connection")
            .help(concat!(
                "An expression that returns the connection strategy to use ",
                "when delivering messages to remote SMTP servers"
            ))
            .default("'default'")
            .build()
            .new_field("queue.strategy.tls")
            .label("TLS")
            .typ(Type::Expression)
            .help(concat!(
                "An expression that returns the TLS strategy to use ",
                "when delivering messages to remote SMTP servers"
            ))
            .default(Expression::new(
                [("retry_num > 0 && last_error == 'tls'", "'invalid-tls'")],
                "'default'",
            ))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .build()
            .new_form_section()
            .title("Outbound Strategies")
            .fields([
                "queue.strategy.route",
                "queue.strategy.schedule",
                "queue.strategy.connection",
                "queue.strategy.tls",
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
                typ: SelectType::Single,
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
            .help(concat!(
                "List of custom DNS server URLs to use for resolution"
            ))
            .default("udp://127.0.0.1:53")
            .typ(Type::Array(ArrayType::Text))
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
            .new_field("resolver.edns")
            .label("Enable EDNS")
            .help(concat!(
                "Whether to enable EDNS (Extension Mechanisms for DNS) support"
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
            .new_form_section()
            .title("DNS Resolver settings")
            .fields([
                "resolver.type",
                "resolver.custom",
                "resolver.concurrency",
                "resolver.timeout",
                "resolver.attempts",
                "resolver.preserve-intermediates",
                "resolver.try-tcp-on-error",
                "resolver.edns",
            ])
            .build()
            .build()
            // Routing strategies
            .new_schema("smtp-out-routing")
            .prefix("queue.route")
            .suffix("type")
            .names("route", "routes")
            .new_id_field()
            .label("ID")
            .help("Unique identifier for the route")
            .build()
            .new_field("type")
            .readonly()
            .label("Type")
            .help("Route type")
            .default("mx")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("local", "Local Delivery"),
                    ("mx", "Remote Delivery (MX)"),
                    ("relay", "Relay Host"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("description")
            .label("Description")
            .help(concat!(
                "A short description of the route, which can be used to ",
                "identify it in the list of routes"
            ))
            .typ(Type::Input)
            .placeholder("Route description")
            .build()
            .new_field("ip-lookup")
            .display_if_eq("type", ["mx"])
            .label("IP Resolution")
            .help("IP resolution strategy for MX hosts")
            .default("ipv4_then_ipv6")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("ipv4_then_ipv6", "IPv4 then IPv6"),
                    ("ipv6_then_ipv4", "IPv6 then IPv4"),
                    ("ipv4_only", "IPv4 Only"),
                    ("ipv6_only", "IPv6 Only"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("limits.mx")
            .display_if_eq("type", ["mx"])
            .label("MX Hosts")
            .help(concat!(
                "Maximum number of MX hosts to try on each delivery attempt"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1i64.into())])
            .default("5")
            .build()
            .new_field("limits.multihomed")
            .display_if_eq("type", ["mx"])
            .label("Multi-homed IPs")
            .help(concat!(
                "For multi-homed remote servers, it is the maximum number of ",
                "IP addresses to try on each delivery attempt"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1i64.into())])
            .default("2")
            .build()
            .new_field("address")
            .display_if_eq("type", ["relay"])
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
            .display_if_eq("type", ["relay"])
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
            .display_if_eq("type", ["relay"])
            .label("Protocol")
            .help(concat!(
                "The protocol to use when delivering messages to the remote ",
                "server, which can be either SMTP or LMTP"
            ))
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[("smtp", "SMTP"), ("lmtp", "LMTP")]),
            })
            .default("smtp")
            .build()
            .new_field("tls.implicit")
            .display_if_eq("type", ["relay"])
            .label("Implicit TLS")
            .help(concat!(
                "Whether to use TLS encryption for all connections to the remote ",
                "server"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("tls.allow-invalid-certs")
            .display_if_eq("type", ["relay"])
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether to allow connections to servers with invalid TLS certificates"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("auth.username")
            .display_if_eq("type", ["relay"])
            .label("Username")
            .help(concat!(
                "The username to use when authenticating with the remote server"
            ))
            .typ(Type::Input)
            .build()
            .new_field("auth.secret")
            .display_if_eq("type", ["relay"])
            .label("Secret")
            .help(concat!(
                "The secret to use when authenticating with the remote server"
            ))
            .typ(Type::Secret)
            .build()
            .new_form_section()
            .title("Route Configuration")
            .fields(["_id", "type", "description"])
            .build()
            .new_form_section()
            .title("MX Resolution")
            .display_if_eq("type", ["mx"])
            .fields(["ip-lookup", "limits.mx", "limits.multihomed"])
            .build()
            .new_form_section()
            .title("Server Details")
            .display_if_eq("type", ["relay"])
            .fields(["address", "port", "protocol"])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["relay"])
            .fields(["tls.implicit", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Authentication")
            .display_if_eq("type", ["relay"])
            .fields(["auth.username", "auth.secret"])
            .build()
            .list_title("Routes")
            .list_subtitle("Manage routes for message delivery")
            .list_fields(["_id", "type", "description"])
            .build()
            // Virtual queues
            .new_schema("smtp-out-queues")
            .prefix("queue.virtual")
            .suffix("threads-per-node")
            .names("queue", "queues")
            .new_id_field()
            .label("Name")
            .help("Unique identifier for the queue, max 8 characters")
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::IsId,
                    Validator::MaxLength(8),
                ],
            )
            .build()
            .new_field("threads-per-node")
            .label("Delivery Threads")
            .help(concat!(
                "Maximum number of threads to use for  delivery ",
                "on each node in the cluster"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1i64.into())])
            .default("25")
            .build()
            .new_field("description")
            .label("Description")
            .help(concat!(
                "A short description of the queue, which can be used to ",
                "identify it in the list of queues"
            ))
            .typ(Type::Input)
            .placeholder("Queue description")
            .build()
            .new_form_section()
            .title("Virtual Queue")
            .fields(["_id", "description", "threads-per-node"])
            .build()
            .list_title("Virtual Queues")
            .list_subtitle("Manage virtual queues for message delivery")
            .list_fields(["_id", "threads-per-node", "description"])
            .build()
            // Scheduling
            .new_schema("smtp-out-scheduling")
            .prefix("queue.schedule")
            .suffix("queue-name")
            .names("schedule", "schedules")
            .new_id_field()
            .label("Name")
            .help("Unique identifier for the schedule")
            .build()
            .new_field("queue-name")
            .label("Virtual Queue")
            .help(concat!(
                "The name of the virtual queue to use for this schedule"
            ))
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "smtp-out-queues",
                    field: "description",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("description")
            .label("Description")
            .help(concat!(
                "A short description of the schedule, which can be used to ",
                "identify it in the list of schedules"
            ))
            .typ(Type::Input)
            .placeholder("Schedule description")
            .build()
            .new_field("retry")
            .label("Retry Intervals")
            .help(concat!("List of retry intervals for message delivery"))
            .default(&["2m", "5m", "10m", "15m", "30m", "1h", "2h"][..])
            .typ(Type::Array(ArrayType::Duration))
            .input_check([], [Validator::Required])
            .build()
            .new_field("notify")
            .label("Notify Intervals")
            .help(concat!(
                "List of delayed delivery DSN notification intervals"
            ))
            .default(&["1d", "3d"][..])
            .typ(Type::Array(ArrayType::Duration))
            .input_check([], [Validator::Required])
            .build()
            .new_field("expire-type")
            .label("Expiration Strategy")
            .help(concat!(
                "Whether to expire messages after a number of delivery ",
                "attempts or after certain time (TTL)"
            ))
            .default("ttl")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("ttl", "Time To Live"),
                    ("attempts", "Delivery Attempts"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("expire")
            .display_if_eq("expire-type", ["ttl"])
            .label("Time To Live")
            .help(concat!(
                "Time after which the message will be expired if it is not ",
                "delivered"
            ))
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("3d")
            .build()
            .new_field("max-attempts")
            .display_if_eq("expire-type", ["attempts"])
            .label("Max Attempts")
            .help(concat!(
                "Maximum number of delivery attempts before the message is ",
                "considered failed"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1i64.into())])
            .default("5")
            .build()
            .new_form_section()
            .title("Schedule Details")
            .fields(["_id", "queue-name", "description"])
            .build()
            .new_form_section()
            .title("Delivery Retry Intervals")
            .fields(["retry"])
            .build()
            .new_form_section()
            .title("Delayed Delivery Notifications")
            .fields(["notify"])
            .build()
            .new_form_section()
            .title("Message Expiration")
            .fields(["expire-type", "expire", "max-attempts"])
            .build()
            .list_title("Schedules")
            .list_subtitle("Manage schedules for message delivery")
            .list_fields(["_id", "queue-name", "description"])
            .build()
            // TLS strategies
            .new_schema("smtp-out-tls")
            .prefix("queue.tls")
            .suffix("allow-invalid-certs")
            .names("TLS strategy", "TLS strategies")
            .new_id_field()
            .label("Name")
            .help("Unique identifier for the TLS strategy")
            .build()
            .new_field("dane")
            .label("DANE")
            .help(concat!("Whether DANE is required, optional, or disabled"))
            .default("optional")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(REQUIRE_OPTIONAL),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("mta-sts")
            .label("MTA-STS")
            .help(concat!(
                "Whether MTA-STS is required, optional, or disabled"
            ))
            .default("optional")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(REQUIRE_OPTIONAL),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("starttls")
            .label("STARTTLS")
            .help(concat!(
                "Whether TLS support is required, optional, or disabled"
            ))
            .default("optional")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(REQUIRE_OPTIONAL),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether to allow connections to servers with invalid TLS certificates"
            ))
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("timeout.tls")
            .label("TLS")
            .help(concat!(
                "Maximum time to wait for the TLS handshake to complete"
            ))
            .default("3m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.mta-sts")
            .label("MTA-STS")
            .help(concat!(
                "Maximum time to wait for the MTA-STS policy lookup to complete"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("description")
            .label("Description")
            .help(concat!(
                "A short description of the TLS strategy, which can be used to ",
                "identify it in the list of strategies"
            ))
            .typ(Type::Input)
            .placeholder("TLS Strategy description")
            .build()
            .new_form_section()
            .title("TLS Strategy")
            .fields(["_id", "description"])
            .build()
            .new_form_section()
            .title("Security Requirements")
            .fields(["dane", "mta-sts", "starttls", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields(["timeout.tls", "timeout.mta-sts"])
            .build()
            .list_title("TLS Strategies")
            .list_subtitle("Manage TLS strategies for message delivery")
            .list_fields(["_id", "description"])
            .build()
            // Connection strategies
            .new_schema("smtp-out-connection")
            .prefix("queue.connection")
            .suffix("timeout.connect")
            .names("Connection strategy", "Connection strategies")
            .new_id_field()
            .label("Name")
            .help("Unique identifier for the connection strategy")
            .build()
            .new_field("source-ips")
            .label("Source IPs")
            .help(concat!(
                "List of local IPv4 and IPv6 addresses to use when ",
                "delivering emails to remote SMTP servers"
            ))
            .typ(Type::Array(ArrayType::Text))
            .input_check([], [Validator::IsIpOrMask])
            .build()
            .new_field("ehlo-hostname")
            .label("EHLO Hostname")
            .help(concat!(
                "Overrides the EHLO hostname that will be used when ",
                "connecting to remote SMTP servers"
            ))
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::IsHost])
            .placeholder("mail.example.com")
            .build()
            .new_field("timeout.connect")
            .label("Connect")
            .help(concat!(
                "Maximum time to wait for the connection to be established"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.greeting")
            .label("Greeting")
            .help(concat!(
                "Maximum time to wait for the SMTP greeting message"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.ehlo")
            .label("EHLO")
            .help(concat!(
                "Maximum time to wait for the EHLO command response"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.mail-from")
            .label("MAIL-FROM")
            .help(concat!(
                "Maximum time to wait for the MAIL-FROM command response"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.rcpt-to")
            .label("RCPT-TO")
            .help(concat!(
                "Maximum time to wait for the RCPT-TO command response"
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.data")
            .label("DATA")
            .help(concat!(
                "Maximum time to wait for the DATA command response"
            ))
            .default("10m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("description")
            .label("Description")
            .help("Short description of the connection strategy")
            .typ(Type::Input)
            .build()
            .new_form_section()
            .title("Connection Strategy")
            .fields(["_id", "description", "ehlo-hostname"])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields([
                "timeout.connect",
                "timeout.greeting",
                "timeout.ehlo",
                "timeout.mail-from",
                "timeout.rcpt-to",
                "timeout.data",
            ])
            .build()
            .new_form_section()
            .title("Source IP Addresses")
            .fields(["source-ips"])
            .build()
            .list_title("Connection Strategies")
            .list_subtitle("Manage connection strategies for message delivery")
            .list_fields(["_id", "description"])
            .build()
            // Outbound rate limiter
            .new_schema("smtp-out-throttle")
            .prefix("queue.limiter.outbound")
            .names("throttle", "throttles")
            .suffix("enable")
            .new_id_field()
            .label("Limiter ID")
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
                typ: SelectType::Many,
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
            .new_field("rate")
            .label("Rate limit")
            .help(concat!(
                "Number of incoming requests over a period of time ",
                "that the rate limiter will allow"
            ))
            .input_check([], [Validator::Required])
            .typ(Type::Rate)
            .build()
            .new_form_section()
            .title("Outbound Rate Limiter")
            .fields(["_id", "key", "rate", "match", "enable"])
            .build()
            .list_title("Outbound Rate Limits")
            .list_subtitle("Manage outbound rate limits")
            .list_fields(["_id", "rate", "enable"])
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
                typ: SelectType::Many,
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
            .default("config_get('server.hostname') + ' Stalwart ESMTP at your service'")
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
            .default("config_get('server.hostname')")
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
                [
                    (
                        "local_port != 25 && is_tls",
                        "[plain, login, oauthbearer, xoauth2]",
                    ),
                    ("local_port != 25", "[oauthbearer, xoauth2]"),
                ],
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
            .new_field("session.mail.is-allowed")
            .label("Sender is allowed")
            .help("Expression that returns true when the sender is allowed to send")
            .input_check([], [Validator::IsValidExpression(has_sender_vars)])
            .default(Expression::new(
                [],
                "!is_empty(authenticated_as) || !key_exists('spam-block', sender_domain)",
            ))
            .build()
            .new_form_section()
            .title("MAIL FROM Stage")
            .fields([
                "session.mail.rewrite",
                "session.mail.is-allowed",
                "session.mail.script",
            ])
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
            .default("100")
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
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(has_rcpt_vars)])
            .new_field("session.data.spam-filter")
            .label("Spam filtering")
            .help("Whether to enable the spam filter for incoming messages")
            .default(Expression::new([], "true"))
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
            .new_field("session.data.add-headers.delivered-to")
            .label("Delivered-To")
            .help("Whether to add a Delivered-To header to the message")
            .default(Expression::new([], "true"))
            .build()
            .new_form_section()
            .title("DATA Stage")
            .fields(["session.data.spam-filter", "session.data.script"])
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
                "session.data.add-headers.delivered-to",
            ])
            .build()
            .build()
            // Inbound rate limiter
            .new_schema("smtp-in-throttle")
            .prefix("queue.limiter.inbound")
            .names("throttle", "throttles")
            .suffix("enable")
            .new_id_field()
            .label("Limiter ID")
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
                typ: SelectType::Many,
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
            .new_field("rate")
            .label("Rate limit")
            .help(concat!(
                "Number of incoming requests over a period of time ",
                "that the rate limiter will allow"
            ))
            .typ(Type::Rate)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Inbound Rate Limiter")
            .fields(["_id", "key", "rate", "match", "enable"])
            .build()
            .list_title("Inbound Rate Limits")
            .list_subtitle("Manage inbound rate limits")
            .list_fields(["_id", "rate", "enable"])
            .build()
            // Milter
            .new_schema("milter")
            .prefix("session.milter")
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
                "Whether Stalwart SMTP should connect to a Milter filter ",
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
                "a connection with this Milter server"
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
                "communicating with this Milter server"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.max-response-size")
            .label("Max Response")
            .help(concat!(
                "Maximum size, in bytes, of a response that Stalwart will accept",
                " from this Milter server"
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
                typ: SelectType::Single,
                source: Source::Static(&[("2", "Version 2"), ("6", "Version 6")]),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("stages")
            .label("Run on stages")
            .help("Which SMTP stages to run the milter on")
            .typ(Type::Select {
                typ: SelectType::Many,
                source: Source::Static(SMTP_STAGES),
            })
            .default("data")
            .build()
            .new_form_section()
            .title("Milter settings")
            .fields([
                "_id",
                "hostname",
                "port",
                "enable",
                "tls",
                "allow-invalid-certs",
            ])
            .build()
            .new_form_section()
            .title("Options")
            .fields([
                "stages",
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
            // MTA Hooks
            .new_schema("mta-hooks")
            .prefix("session.hook")
            .suffix("url")
            .names("hook", "hooks")
            .new_id_field()
            .label("Hook Id")
            .help("Unique identifier for this hook")
            .build()
            .new_field("enable")
            .label("Enable")
            .help("Expression that determines whether to enable this hook")
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
            .new_field("url")
            .label("Endpoint URL")
            .help(concat!("URL of the hook endpoint"))
            .placeholder("https://127.0.0.1/filter")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether Stalwart SMTP should connect to a hook ",
                "server that has an invalid TLS certificate"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout")
            .label("Timeout")
            .help(concat!(
                "Maximum amount of time that Stalwart SMTP will wait for a response ",
                "from this hook server"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.tempfail-on-error")
            .label("TempFail on Error")
            .help(concat!(
                "Whether to respond with a temporary failure (typically a 4xx ",
                "SMTP status code) when Stalwart encounters an error while ",
                "communicating with this MTA Hook server"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.max-response-size")
            .label("Max Size")
            .help(concat!(
                "Maximum size, in bytes, of a response that Stalwart will accept",
                " from this MTA Hook server"
            ))
            .default("52428800")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .build()
            .new_field("headers")
            .typ(Type::Array(ArrayType::Text))
            .label("HTTP Headers")
            .help("The headers to be sent with hook requests")
            .build()
            .new_field("auth.username")
            .label("Username")
            .help(concat!(
                "The username to use when authenticating with the hook server"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("auth.secret")
            .label("Secret")
            .help(concat!(
                "The secret to use when authenticating with the hook server"
            ))
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("stages")
            .label("Run on stages")
            .help("Which SMTP stages to run this hook on")
            .typ(Type::Select {
                typ: SelectType::Many,
                source: Source::Static(SMTP_STAGES),
            })
            .default("data")
            .build()
            .new_form_section()
            .title("MTA Hook settings")
            .fields(["_id", "url", "enable", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["auth.username", "auth.secret"])
            .build()
            .new_form_section()
            .title("Options")
            .fields(["stages", "headers"])
            .build()
            .new_form_section()
            .title("Response")
            .fields([
                "options.max-response-size",
                "timeout",
                "options.tempfail-on-error",
            ])
            .build()
            .list_title("MTA Hooks")
            .list_subtitle("Manage MTA Hooks")
            .list_fields(["_id", "url"])
            .build()
            // MTA-STS
            .new_schema("smtp-in-mta-sts")
            .new_field("session.mta-sts.mode")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("enforce", "Enforce"),
                    ("testing", "Testing"),
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
            .typ(Type::Array(ArrayType::Text))
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
            // ASN & GeoIP
            .new_schema("smtp-in-asn")
            .new_field("asn.type")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("resource", "URL Resource"),
                    ("dns", "DNS Lookup"),
                    ("disable", "Disabled"),
                ]),
            })
            .input_check([], [Validator::Required])
            .label("ASN/Geo Source")
            .help("Whether to obtain ASN and geolocation data from a URL or DNS lookup")
            .default("disable")
            .build()
            .new_field("asn.urls.asn")
            .label("ASN URLs")
            .help(concat!(
                "URLs to fetch CSV file containing the IP to ASN mappings.",
            ))
            .typ(Type::Array(ArrayType::Text))
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.urls.geo")
            .label("Geolocation URLs")
            .help(concat!(
                "URLs to fetch CSV file containing the IP to country code mappings.",
            ))
            .typ(Type::Array(ArrayType::Text))
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.timeout")
            .label("Timeout")
            .help(concat!(
                "Time after which the ASN/Geo resource fetch is considered failed.",
            ))
            .default("5m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.expires")
            .label("Expiry")
            .help(concat!("How often to refresh the ASN/Geo data.",))
            .default("1d")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.max-size")
            .label("Max Size")
            .help(concat!("Maximum size of the ASN/Geo data file.",))
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .default("104857600")
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.headers")
            .typ(Type::Array(ArrayType::Text))
            .label("HTTP Headers")
            .help(concat!(
                "Headers to send with the ASN/Geo resource fetch request.",
            ))
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_field("asn.zone.ipv4")
            .label("IPv4 Zone")
            .help(concat!(
                "The DNS zone to query for IPv4 ASN and geolocation data.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["dns"])
            .build()
            .new_field("asn.zone.ipv6")
            .label("IPv6 Zone")
            .help(concat!(
                "The DNS zone to query for IPv6 ASN and geolocation data.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["dns"])
            .build()
            .new_field("asn.separator")
            .label("Separator")
            .help(concat!(
                "The separator character used in the DNS TXT record.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["dns"])
            .default("|")
            .build()
            .new_field("asn.index.asn")
            .label("ASN Index")
            .help(concat!("The position of the ASN in the DNS TXT record.",))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("asn.type", ["dns"])
            .default("0")
            .build()
            .new_field("asn.index.asn-name")
            .label("ASN Name Index")
            .help(concat!(
                "The position of the ASN Name in the DNS TXT record.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("asn.type", ["dns"])
            .build()
            .new_field("asn.index.country")
            .label("Country Index")
            .help(concat!(
                "The position of the country code in the DNS TXT record.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("asn.type", ["dns"])
            .build()
            .new_form_section()
            .title("ASN & GeoIP Settings")
            .fields(["asn.type"])
            .build()
            .new_form_section()
            .title("URL Resources")
            .fields(["asn.urls.asn", "asn.urls.geo"])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_form_section()
            .title("Retrieval")
            .fields(["asn.expires", "asn.timeout", "asn.max-size"])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["asn.headers"])
            .display_if_eq("asn.type", ["resource"])
            .build()
            .new_form_section()
            .title("DNS Zones")
            .fields(["asn.zone.ipv4", "asn.zone.ipv6"])
            .display_if_eq("asn.type", ["dns"])
            .build()
            .new_form_section()
            .title("TXT Record Format")
            .fields([
                "asn.separator",
                "asn.index.asn",
                "asn.index.asn-name",
                "asn.index.country",
            ])
            .display_if_eq("asn.type", ["dns"])
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

pub static SMTP_STAGES: &[(&str, &str)] = &[
    ("connect", "Connect"),
    ("ehlo", "EHLO"),
    ("mail", "MAIL FROM"),
    ("rcpt", "RCPT TO"),
    ("data", "DATA"),
];
