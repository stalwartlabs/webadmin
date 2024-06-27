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
            .label("Logger Id")
            .help("Unique identifier for the tracer")
            .build()
            // Type
            .new_field("type")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("log", "Log file"),
                    ("stdout", "Standard output"),
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
                multi: false,
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
            // Enable
            .new_field("ansi")
            .typ(Type::Boolean)
            .label("Use ANSI colors in logs")
            .help("Whether to use ANSI colors in logs")
            .display_if_eq("type", ["log", "stdout"])
            .default("false")
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
                multi: false,
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
                multi: false,
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
            // Forms
            .new_form_section()
            .title("Logger settings")
            .fields([
                "_id",
                "type",
                "level",
                "path",
                "prefix",
                "rotate",
                "transport",
                "endpoint",
                "headers",
                "ansi",
                "enable",
            ])
            .build()
            .list_title("Logging methods")
            .list_subtitle("Manage logging and tracing methods")
            .list_fields(["_id", "type", "level", "enable"])
            .build()
    }
}
