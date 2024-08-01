/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_tracing(self) -> Self {
        self.new_schema("tracing")
            .names("tracer", "tracers")
            .prefix("tracer")
            .suffix("type")
            // Id
            .new_id_field()
            .label("Tracer Id")
            .help("Unique identifier for the tracer")
            .build()
            // Type
            .new_field("type")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("log", "Log file"),
                    ("stdout", "Console"),
                    ("journal", "Systemd Journal"),
                    ("open-telemetry", "Open Telemetry"),
                ]),
            })
            .label("Method")
            .help("The type of tracer")
            .input_check([], [Validator::Required])
            .default("log")
            .build()
            // Level
            .new_field("level")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("error", "Error - Only errors are logged"),
                    ("warn", "Warning - Errors and warnings are logged"),
                    ("info", "Info - Errors, warnings and info are logged"),
                    (
                        "debug",
                        "Debug - Errors, warnings, info and debug are logged",
                    ),
                    (
                        "trace",
                        "Trace - Errors, warnings, info, debug and trace are logged",
                    ),
                ]),
            })
            .label("Logging level")
            .help("The logging level for this tracer")
            .input_check([], [Validator::Required])
            .default("info")
            .build()
            // Enable
            .new_field("enable")
            .typ(Type::Boolean)
            .label("Enable this tracer")
            .help("Enable or disable the tracer")
            .default("true")
            .build()
            // ANSI
            .new_field("ansi")
            .typ(Type::Boolean)
            .label("Use ANSI colors")
            .help("Whether to use ANSI colors in logs")
            .display_if_eq("type", ["log", "stdout"])
            .default("false")
            .build()
            // Multiline
            .new_field("multiline")
            .typ(Type::Boolean)
            .label("Multiline entries")
            .help("Whether to write log entries as a single line or multiline")
            .display_if_eq("type", ["log", "stdout"])
            .default("false")
            .build()
            // Buffered
            .new_field("buffered")
            .typ(Type::Boolean)
            .label("Buffered writes")
            .help("Whether to buffer log entries before writing to console")
            .display_if_eq("type", ["stdout"])
            .default("true")
            .build()
            // Lossy
            .new_field("lossy")
            .typ(Type::Boolean)
            .label("Lossy mode")
            .help("Whether to drop log entries if there is backlog")
            .default("false")
            .build()
            // Disabled events
            .new_field("disabled-events")
            .label("Disabled Events")
            .help("Which events to disable for this tracer")
            .typ(Type::Select {
                typ: SelectType::ManyWithSearch,
                source: Source::StaticId(EVENT_NAMES),
            })
            .build()
            // Log Path
            .new_field("path")
            .typ(Type::Input)
            .label("Path")
            .help("The path to the log file")
            .placeholder("/var/log")
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("type", ["log"])
            .build()
            // Log Prefix
            .new_field("prefix")
            .typ(Type::Input)
            .label("Prefix")
            .help("The prefix for the log file")
            .placeholder("stalwart.log")
            .input_check([Transformer::Trim], [Validator::Required])
            .display_if_eq("type", ["log"])
            .build()
            // Log Rotate
            .new_field("rotate")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("daily", "Daily"),
                    ("hourly", "Hourly"),
                    ("minutely", "Minutely"),
                    ("never", "Never"),
                ]),
            })
            .label("Rotate frequency")
            .help("The frequency to rotate the log file")
            .input_check([], [Validator::Required])
            .default("daily")
            .display_if_eq("type", ["log"])
            .build()
            // OT Transport
            .new_field("transport")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[("http", "HTTP"), ("grpc", "gRPC")]),
            })
            .label("Transport")
            .help("The transport protocol for Open Telemetry")
            .input_check([], [Validator::Required])
            .display_if_eq("type", ["open-telemetry"])
            .default("http")
            .build()
            // OT Endpoint
            .new_field("endpoint")
            .typ(Type::Input)
            .label("Endpoint")
            .help("The endpoint for Open Telemetry")
            .placeholder("https://tracing.example.com/v1/otel")
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .display_if_eq("type", ["open-telemetry"])
            .build()
            // OT Headers
            .new_field("headers")
            .typ(Type::Array)
            .label("HTTP Headers")
            .help("The headers to be sent with OpenTelemetry requests")
            .display_if_eq("transport", ["http"])
            .build()
            // OT Timeout
            .new_field("timeout")
            .label("Timeout")
            .help(concat!(
                "Maximum amount of time that Stalwart will wait for a response ",
                "from the OpenTelemetry endpoint"
            ))
            .default("10s")
            .typ(Type::Duration)
            .display_if_eq("type", ["open-telemetry"])
            .input_check([], [Validator::Required])
            .build()
            // OT Throttle
            .new_field("throttle")
            .label("Throttle")
            .help(concat!(
                "The minimum amount of time that must pass between ",
                "each request to the OpenTelemetry endpoint"
            ))
            .default("1s")
            .display_if_eq("type", ["open-telemetry"])
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            // OT Export Logs
            .new_field("enable.log-exporter")
            .typ(Type::Boolean)
            .label("Export logs")
            .help("Whether to export logs to OpenTelemetry")
            .display_if_eq("type", ["open-telemetry"])
            .default("true")
            .build()
            // OT Export Spans
            .new_field("enable.span-exporter")
            .typ(Type::Boolean)
            .label("Export spans")
            .help("Whether to export spans to OpenTelemetry")
            .display_if_eq("type", ["open-telemetry"])
            .default("true")
            .build()
            // Forms
            .new_form_section()
            .title("Tracer configuration")
            .fields(["_id", "type", "level", "enable"])
            .build()
            .new_form_section()
            .title("Options")
            .fields([
                "path",
                "prefix",
                "rotate",
                "transport",
                "endpoint",
                "timeout",
                "throttle",
                "headers",
                "enable.log-exporter",
                "enable.span-exporter",
                "ansi",
                "multiline",
                "buffered",
                "lossy",
            ])
            .build()
            .new_form_section()
            .title("Override events")
            .fields(["disabled-events"])
            .build()
            .list_title("Logging & tracing methods")
            .list_subtitle("Manage logging and tracing methods")
            .list_fields(["_id", "type", "level", "enable"])
            .build()
            // Custom levels
            .new_schema("custom-levels")
            .names("event", "events")
            .prefix("tracing.level")
            // Id
            .new_field("_id")
            .readonly()
            .label("Event Id")
            .help("Unique identifier of the event")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::StaticId(EVENT_NAMES),
            })
            .build()
            // Level
            .new_field("_value")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[
                    ("disable", "Disabled"),
                    ("error", "Error"),
                    ("warn", "Warning"),
                    ("info", "Info"),
                    ("debug", "Debug"),
                    ("trace", "Trace"),
                ]),
            })
            .label("Level")
            .help("The logging level for this event")
            .input_check([], [Validator::Required])
            .default("info")
            .build()
            .new_form_section()
            .fields(["_id", "_value"])
            .build()
            .list_title("Custom event levels")
            .list_subtitle("Manage custom event logging levels")
            .list_fields(["_id", "_value"])
            .build()
    }
}

pub static EVENT_NAMES: &[&str] = &[
    "acme.auth-completed",
    "acme.auth-error",
    "acme.auth-pending",
    "acme.auth-start",
    "acme.auth-too-many-attempts",
    "acme.auth-valid",
    "acme.client-missing-sni",
    "acme.client-supplied-sni",
    "acme.dns-record-created",
    "acme.dns-record-creation-failed",
    "acme.dns-record-deletion-failed",
    "acme.dns-record-lookup-failed",
    "acme.dns-record-not-propagated",
    "acme.dns-record-propagated",
    "acme.dns-record-propagation-timeout",
    "acme.error",
    "acme.order-completed",
    "acme.order-invalid",
    "acme.order-processing",
    "acme.order-ready",
    "acme.order-start",
    "acme.order-valid",
    "acme.process-cert",
    "acme.renew-backoff",
    "acme.tls-alpn-error",
    "acme.tls-alpn-received",
    "acme.token-not-found",
    "arc.broken-chain",
    "arc.chain-too-long",
    "arc.has-header-tag",
    "arc.invalid-cv",
    "arc.invalid-instance",
    "arc.sealer-not-found",
    "auth.banned",
    "auth.error",
    "auth.failed",
    "auth.missing-totp",
    "auth.success",
    "auth.too-many-attempts",
    "cluster.decryption-error",
    "cluster.empty-packet",
    "cluster.error",
    "cluster.invalid-packet",
    "cluster.one-or-more-peers-offline",
    "cluster.peer-alive",
    "cluster.peer-back-online",
    "cluster.peer-discovered",
    "cluster.peer-has-config-changes",
    "cluster.peer-has-list-changes",
    "cluster.peer-leaving",
    "cluster.peer-offline",
    "cluster.peer-suspected",
    "cluster.peer-suspected-is-alive",
    "config.already-up-to-date",
    "config.build-error",
    "config.build-warning",
    "config.default-applied",
    "config.external-key-ignored",
    "config.fetch-error",
    "config.import-external",
    "config.macro-error",
    "config.missing-setting",
    "config.parse-error",
    "config.parse-warning",
    "config.unused-setting",
    "config.write-error",
    "dane.authentication-failure",
    "dane.authentication-success",
    "dane.certificate-parse-error",
    "dane.no-certificates-found",
    "dane.tlsa-record-fetch",
    "dane.tlsa-record-fetch-error",
    "dane.tlsa-record-invalid",
    "dane.tlsa-record-match",
    "dane.tlsa-record-not-dnssec-signed",
    "dane.tlsa-record-not-found",
    "delivery.attempt-end",
    "delivery.attempt-start",
    "delivery.auth",
    "delivery.auth-failed",
    "delivery.completed",
    "delivery.concurrency-limit-exceeded",
    "delivery.connect",
    "delivery.connect-error",
    "delivery.delivered",
    "delivery.domain-delivery-start",
    "delivery.double-bounce",
    "delivery.dsn-perm-fail",
    "delivery.dsn-success",
    "delivery.dsn-temp-fail",
    "delivery.ehlo",
    "delivery.ehlo-rejected",
    "delivery.failed",
    "delivery.greeting-failed",
    "delivery.implicit-tls-error",
    "delivery.ip-lookup",
    "delivery.ip-lookup-failed",
    "delivery.mail-from",
    "delivery.mail-from-rejected",
    "delivery.message-rejected",
    "delivery.missing-outbound-hostname",
    "delivery.mx-lookup",
    "delivery.mx-lookup-failed",
    "delivery.null-mx",
    "delivery.rate-limit-exceeded",
    "delivery.raw-input",
    "delivery.raw-output",
    "delivery.rcpt-to",
    "delivery.rcpt-to-failed",
    "delivery.rcpt-to-rejected",
    "delivery.start-tls",
    "delivery.start-tls-disabled",
    "delivery.start-tls-error",
    "delivery.start-tls-unavailable",
    "dkim.fail",
    "dkim.failed-auid-match",
    "dkim.failed-body-hash-match",
    "dkim.failed-verification",
    "dkim.incompatible-algorithms",
    "dkim.neutral",
    "dkim.none",
    "dkim.pass",
    "dkim.perm-error",
    "dkim.revoked-public-key",
    "dkim.signature-expired",
    "dkim.signature-length",
    "dkim.signer-not-found",
    "dkim.temp-error",
    "dkim.unsupported-algorithm",
    "dkim.unsupported-canonicalization",
    "dkim.unsupported-key-type",
    "dkim.unsupported-version",
    "dmarc.fail",
    "dmarc.none",
    "dmarc.pass",
    "dmarc.perm-error",
    "dmarc.temp-error",
    "eval.directory-not-found",
    "eval.error",
    "eval.result",
    "eval.store-not-found",
    "fts-index.blob-not-found",
    "fts-index.index",
    "fts-index.lock-busy",
    "fts-index.locked",
    "fts-index.metadata-not-found",
    "housekeeper.purge-accounts",
    "housekeeper.purge-sessions",
    "housekeeper.purge-store",
    "housekeeper.schedule",
    "housekeeper.start",
    "housekeeper.stop",
    "http.error",
    "http.request-body",
    "http.request-url",
    "http.response-body",
    "http.x-forwarded-missing",
    "imap.append",
    "imap.capabilities",
    "imap.close",
    "imap.copy",
    "imap.create-mailbox",
    "imap.delete-mailbox",
    "imap.enable",
    "imap.error",
    "imap.expunge",
    "imap.fetch",
    "imap.get-acl",
    "imap.id",
    "imap.idle-start",
    "imap.idle-stop",
    "imap.list",
    "imap.list-rights",
    "imap.logout",
    "imap.lsub",
    "imap.move",
    "imap.my-rights",
    "imap.namespace",
    "imap.noop",
    "imap.raw-input",
    "imap.raw-output",
    "imap.rename-mailbox",
    "imap.search",
    "imap.select",
    "imap.set-acl",
    "imap.sort",
    "imap.status",
    "imap.store",
    "imap.subscribe",
    "imap.thread",
    "imap.unsubscribe",
    "incoming-report.abuse-report",
    "incoming-report.arf-parse-failed",
    "incoming-report.auth-failure-report",
    "incoming-report.decompress-error",
    "incoming-report.dmarc-parse-failed",
    "incoming-report.dmarc-report",
    "incoming-report.dmarc-report-with-warnings",
    "incoming-report.fraud-report",
    "incoming-report.message-parse-failed",
    "incoming-report.not-spam-report",
    "incoming-report.other-report",
    "incoming-report.tls-report",
    "incoming-report.tls-report-with-warnings",
    "incoming-report.tls-rpc-parse-failed",
    "incoming-report.virus-report",
    "iprev.fail",
    "iprev.none",
    "iprev.pass",
    "iprev.perm-error",
    "iprev.temp-error",
    "jmap.account-not-found",
    "jmap.account-not-supported-by-method",
    "jmap.account-read-only",
    "jmap.anchor-not-found",
    "jmap.cannot-calculate-changes",
    "jmap.forbidden",
    "jmap.invalid-arguments",
    "jmap.invalid-result-reference",
    "jmap.method-call",
    "jmap.not-found",
    "jmap.not-json",
    "jmap.not-request",
    "jmap.request-too-large",
    "jmap.state-mismatch",
    "jmap.unknown-capability",
    "jmap.unknown-data-type",
    "jmap.unknown-method",
    "jmap.unsupported-filter",
    "jmap.unsupported-sort",
    "jmap.websocket-error",
    "jmap.websocket-start",
    "jmap.websocket-stop",
    "limit.blob-quota",
    "limit.calls-in",
    "limit.concurrent-connection",
    "limit.concurrent-request",
    "limit.concurrent-upload",
    "limit.quota",
    "limit.size-request",
    "limit.size-upload",
    "limit.too-many-requests",
    "mail-auth.base64",
    "mail-auth.crypto",
    "mail-auth.dns-error",
    "mail-auth.dns-invalid-record-type",
    "mail-auth.dns-record-not-found",
    "mail-auth.io",
    "mail-auth.missing-parameters",
    "mail-auth.no-headers-found",
    "mail-auth.parse-error",
    "mail-auth.policy-not-aligned",
    "manage-sieve.capabilities",
    "manage-sieve.check-script",
    "manage-sieve.create-script",
    "manage-sieve.delete-script",
    "manage-sieve.error",
    "manage-sieve.get-script",
    "manage-sieve.have-space",
    "manage-sieve.list-scripts",
    "manage-sieve.logout",
    "manage-sieve.noop",
    "manage-sieve.raw-input",
    "manage-sieve.raw-output",
    "manage-sieve.rename-script",
    "manage-sieve.set-active",
    "manage-sieve.start-tls",
    "manage-sieve.unauthenticate",
    "manage-sieve.update-script",
    "manage.already-exists",
    "manage.assert-failed",
    "manage.error",
    "manage.missing-parameter",
    "manage.not-found",
    "manage.not-supported",
    "milter.action-accept",
    "milter.action-connection-failure",
    "milter.action-discard",
    "milter.action-reject",
    "milter.action-reply-code",
    "milter.action-shutdown",
    "milter.action-temp-fail",
    "milter.disconnected",
    "milter.frame-invalid",
    "milter.frame-too-large",
    "milter.io-error",
    "milter.parse-error",
    "milter.read",
    "milter.timeout",
    "milter.tls-invalid-name",
    "milter.unexpected-response",
    "milter.write",
    "mta-hook.action-accept",
    "mta-hook.action-discard",
    "mta-hook.action-quarantine",
    "mta-hook.action-reject",
    "mta-hook.error",
    "mta-sts.authorized",
    "mta-sts.invalid-policy",
    "mta-sts.not-authorized",
    "mta-sts.policy-fetch",
    "mta-sts.policy-fetch-error",
    "mta-sts.policy-not-found",
    "network.accept-error",
    "network.bind-error",
    "network.closed",
    "network.connection-end",
    "network.connection-start",
    "network.drop-blocked",
    "network.flush-error",
    "network.listen-error",
    "network.listen-start",
    "network.listen-stop",
    "network.proxy-error",
    "network.read-error",
    "network.set-opt-error",
    "network.split-error",
    "network.timeout",
    "network.write-error",
    "outgoing-report.dkim-rate-limited",
    "outgoing-report.dkim-report",
    "outgoing-report.dmarc-aggregate-report",
    "outgoing-report.dmarc-rate-limited",
    "outgoing-report.dmarc-report",
    "outgoing-report.http-submission",
    "outgoing-report.lock-busy",
    "outgoing-report.lock-deleted",
    "outgoing-report.locked",
    "outgoing-report.no-recipients-found",
    "outgoing-report.not-found",
    "outgoing-report.reporting-address-validation-error",
    "outgoing-report.spf-rate-limited",
    "outgoing-report.spf-report",
    "outgoing-report.submission-error",
    "outgoing-report.tls-aggregate",
    "outgoing-report.unauthorized-reporting-address",
    "pop3.capabilities",
    "pop3.delete",
    "pop3.error",
    "pop3.fetch",
    "pop3.list",
    "pop3.list-message",
    "pop3.noop",
    "pop3.quit",
    "pop3.raw-input",
    "pop3.raw-output",
    "pop3.reset",
    "pop3.start-tls",
    "pop3.stat",
    "pop3.uidl",
    "pop3.uidl-message",
    "pop3.utf8",
    "purge.auto-expunge",
    "purge.error",
    "purge.finished",
    "purge.purge-active",
    "purge.running",
    "purge.started",
    "purge.tombstone-cleanup",
    "push-subscription.error",
    "push-subscription.not-found",
    "push-subscription.success",
    "queue.blob-not-found",
    "queue.concurrency-limit-exceeded",
    "queue.lock-busy",
    "queue.locked",
    "queue.quota-exceeded",
    "queue.rate-limit-exceeded",
    "queue.rescheduled",
    "queue.scheduled",
    "resource.bad-parameters",
    "resource.download-external",
    "resource.error",
    "resource.not-found",
    "resource.webadmin-unpacked",
    "server.licensing",
    "server.shutdown",
    "server.startup",
    "server.startup-error",
    "server.thread-error",
    "sieve.action-accept",
    "sieve.action-accept-replace",
    "sieve.action-discard",
    "sieve.action-reject",
    "sieve.list-not-found",
    "sieve.message-too-large",
    "sieve.not-supported",
    "sieve.quota-exceeded",
    "sieve.runtime-error",
    "sieve.script-not-found",
    "sieve.send-message",
    "sieve.unexpected-error",
    "smtp.already-authenticated",
    "smtp.arc-fail",
    "smtp.arc-pass",
    "smtp.auth-exchange-too-long",
    "smtp.auth-mechanism-not-supported",
    "smtp.auth-not-allowed",
    "smtp.command-not-implemented",
    "smtp.concurrency-limit-exceeded",
    "smtp.deliver-by-disabled",
    "smtp.deliver-by-invalid",
    "smtp.did-not-say-ehlo",
    "smtp.dkim-fail",
    "smtp.dkim-pass",
    "smtp.dmarc-fail",
    "smtp.dmarc-pass",
    "smtp.dsn-disabled",
    "smtp.ehlo",
    "smtp.ehlo-expected",
    "smtp.error",
    "smtp.expn",
    "smtp.expn-disabled",
    "smtp.expn-not-found",
    "smtp.future-release-disabled",
    "smtp.future-release-invalid",
    "smtp.help",
    "smtp.invalid-command",
    "smtp.invalid-ehlo",
    "smtp.invalid-parameter",
    "smtp.invalid-recipient-address",
    "smtp.invalid-sender-address",
    "smtp.iprev-fail",
    "smtp.iprev-pass",
    "smtp.lhlo-expected",
    "smtp.loop-detected",
    "smtp.mail-from",
    "smtp.mail-from-missing",
    "smtp.mail-from-rewritten",
    "smtp.mail-from-unauthenticated",
    "smtp.mail-from-unauthorized",
    "smtp.mailbox-does-not-exist",
    "smtp.message-parse-failed",
    "smtp.message-too-large",
    "smtp.missing-auth-directory",
    "smtp.missing-local-hostname",
    "smtp.mt-priority-disabled",
    "smtp.mt-priority-invalid",
    "smtp.multiple-mail-from",
    "smtp.noop",
    "smtp.pipe-error",
    "smtp.pipe-success",
    "smtp.quit",
    "smtp.rate-limit-exceeded",
    "smtp.raw-input",
    "smtp.raw-output",
    "smtp.rcpt-to",
    "smtp.rcpt-to-duplicate",
    "smtp.rcpt-to-missing",
    "smtp.rcpt-to-rewritten",
    "smtp.relay-not-allowed",
    "smtp.remote-id-not-found",
    "smtp.request-too-large",
    "smtp.require-tls-disabled",
    "smtp.rset",
    "smtp.spf-ehlo-fail",
    "smtp.spf-ehlo-pass",
    "smtp.spf-from-fail",
    "smtp.spf-from-pass",
    "smtp.start-tls",
    "smtp.start-tls-already",
    "smtp.start-tls-unavailable",
    "smtp.syntax-error",
    "smtp.time-limit-exceeded",
    "smtp.too-many-invalid-rcpt",
    "smtp.too-many-messages",
    "smtp.too-many-recipients",
    "smtp.transfer-limit-exceeded",
    "smtp.unsupported-parameter",
    "smtp.vrfy",
    "smtp.vrfy-disabled",
    "smtp.vrfy-not-found",
    "spam.classify",
    "spam.classify-error",
    "spam.list-updated",
    "spam.not-enough-training-data",
    "spam.pyzor-error",
    "spam.train",
    "spam.train-balance",
    "spam.train-error",
    "spf.fail",
    "spf.neutral",
    "spf.none",
    "spf.pass",
    "spf.perm-error",
    "spf.soft-fail",
    "spf.temp-error",
    "store.assert-value-failed",
    "store.blob-missing-marker",
    "store.crypto-error",
    "store.data-corruption",
    "store.decompress-error",
    "store.deserialize-error",
    "store.elasticsearch-error",
    "store.filesystem-error",
    "store.foundationdb-error",
    "store.ingest",
    "store.ingest-duplicate",
    "store.ingest-error",
    "store.ldap-bind",
    "store.ldap-error",
    "store.ldap-query",
    "store.mysql-error",
    "store.not-configured",
    "store.not-found",
    "store.not-supported",
    "store.pool-error",
    "store.postgresql-error",
    "store.redis-error",
    "store.rocksdb-error",
    "store.s3-error",
    "store.sql-query",
    "store.sqlite-error",
    "store.unexpected-error",
    "tls-rpt.record-fetch",
    "tls-rpt.record-fetch-error",
    "tls.certificate-not-found",
    "tls.handshake",
    "tls.handshake-error",
    "tls.multiple-certificates-available",
    "tls.no-certificates-available",
    "tls.not-configured",
    "tracing.journal-error",
    "tracing.log-error",
    "tracing.otel-error",
    "tracing.update",
    "tracing.webhook-error",
];
