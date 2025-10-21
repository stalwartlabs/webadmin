/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

impl Builder<Schemas, ()> {
    #![allow(clippy::useless_concat)]
    pub fn build_jmap(self) -> Self {
        self.new_schema("jmap-limits")
            .new_field("jmap.protocol.get.max-objects")
            .label("Get")
            .help(concat!(
                "Determines the maximum number of objects that can be fetched in a ",
                "single method call"
            ))
            .default("500")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .new_field("jmap.protocol.set.max-objects")
            .label("Set")
            .help(concat!(
                "Establishes the maximum number of objects that can be modified in ",
                "a single method call"
            ))
            .default("500")
            .new_field("jmap.protocol.request.max-concurrent")
            .label("Concurrent")
            .help(concat!(
                "Restricts the number of concurrent requests a user can make to the ",
                "JMAP server"
            ))
            .default("4")
            .new_field("jmap.protocol.request.max-size")
            .label("Size")
            .help(concat!(
                "Defines the maximum size of a single request, in bytes, that the ",
                "server will accept"
            ))
            .default("10000000")
            .new_field("jmap.protocol.request.max-calls")
            .label("Method Calls")
            .help(concat!(
                "Limits the maximum number of method calls that can be included in",
                " a single request"
            ))
            .default("16")
            .new_field("jmap.protocol.query.max-results")
            .label("Query")
            .help(concat!(
                "Sets the maximum number of results that a Query method can return"
            ))
            .default("5000")
            .new_field("jmap.protocol.upload.max-size")
            .label("Max Size")
            .help(concat!(
                "Defines the maximum file size for file uploads to the server"
            ))
            .default("50000000")
            .new_field("jmap.protocol.upload.max-concurrent")
            .label("Max Concurrent")
            .help(concat!(
                "Restricts the number of concurrent file uploads a user can perform"
            ))
            .default("4")
            .new_field("jmap.protocol.upload.quota.files")
            .label("Total Files")
            .help(concat!(
                "Specifies the maximum number of files that a user can upload within ",
                "a certain period"
            ))
            .default("1000")
            .new_field("jmap.protocol.upload.quota.size")
            .label("Total Size")
            .default("50000000")
            .help(concat!(
                "Defines the total size of files that a user can upload within a ",
                "certain period"
            ))
            .new_field("jmap.protocol.changes.max-results")
            .label("Changes")
            .help(concat!(
                "Determines the maximum number of change objects that a Changes",
                " method can return"
            ))
            .default("5000")
            .new_field("jmap.mailbox.max-depth")
            .label("Max Depth")
            .help(concat!(
                "Restricts the maximum depth of nested mailboxes a user can ",
                "create"
            ))
            .default("10")
            .new_field("jmap.mailbox.max-name-length")
            .label("Name Length")
            .help(concat!("Establishes the maximum length of a mailbox name"))
            .default("255")
            .new_field("jmap.email.max-attachment-size")
            .label("Attachment Size")
            .help(concat!(
                "Specifies the maximum size for an email attachment"
            ))
            .default("50000000")
            .new_field("jmap.email.max-size")
            .label("E-mail Size")
            .help(concat!("Determines the maximum size for an email message"))
            .default("75000000")
            .new_field("jmap.email.parse.max-items")
            .label("Emails")
            .help(concat!(
                "Limits the maximum number of e-mail message that can be parsed in ",
                "a single request"
            ))
            .default("10")
            .new_field("jmap.calendar.parse.max-items")
            .label("Calendars")
            .help(concat!(
                "Limits the maximum number of iCalendar items that can be parsed in ",
                "a single request"
            ))
            .default("10")
            .new_field("jmap.contact.parse.max-items")
            .label("Contacts")
            .help(concat!(
                "Limits the maximum number of vCard items that can be parsed in ",
                "a single request"
            ))
            .default("10")
            .build()
            .new_field("jmap.protocol.upload.ttl")
            .label("Expire after")
            .help(concat!(
                "Specifies the Time-To-Live (TTL) for each uploaded file, after ",
                "which the file is deleted from temporary storage"
            ))
            .default("1h")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Request Limits")
            .fields([
                "jmap.protocol.request.max-concurrent",
                "jmap.protocol.request.max-size",
                "jmap.protocol.request.max-calls",
            ])
            .build()
            .new_form_section()
            .title("Max Objects")
            .fields([
                "jmap.protocol.get.max-objects",
                "jmap.protocol.set.max-objects",
            ])
            .build()
            .new_form_section()
            .title("Max Results")
            .fields([
                "jmap.protocol.query.max-results",
                "jmap.protocol.changes.max-results",
            ])
            .build()
            .new_form_section()
            .title("Upload Limits")
            .fields([
                "jmap.protocol.upload.max-size",
                "jmap.protocol.upload.max-concurrent",
                "jmap.protocol.upload.quota.files",
                "jmap.protocol.upload.quota.size",
                "jmap.protocol.upload.ttl",
            ])
            .build()
            .new_form_section()
            .title("Mailbox Limits")
            .fields(["jmap.mailbox.max-depth", "jmap.mailbox.max-name-length"])
            .build()
            .new_form_section()
            .title("Email Limits")
            .fields(["jmap.email.max-attachment-size", "jmap.email.max-size"])
            .build()
            .new_form_section()
            .title("Parsing Limits")
            .fields([
                "jmap.email.parse.max-items",
                "jmap.calendar.parse.max-items",
                "jmap.contact.parse.max-items",
            ])
            .build()
            .build()
            // Push & EventSource
            .new_schema("jmap-push")
            .new_field("jmap.push.throttle")
            .label("Throttle")
            .help("Time to wait before sending a new request to the push service")
            .default("1ms")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.push.attempts.interval")
            .label("Attempt Interval")
            .help("Time to wait between push attempts")
            .default("1m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.push.attempts.max")
            .label("Max Attempts")
            .help("Maximum number of push attempts before a notification is discarded")
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .build()
            .new_field("jmap.push.retry.interval")
            .label("Retry Interval")
            .help("Time to wait between retry attempts")
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.push.timeout.request")
            .label("Request")
            .help("Time before a connection with a push service URL times out")
            .default("10s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.push.timeout.verify")
            .label("Verify")
            .help("Time to wait for the push service to verify a subscription")
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.event-source.throttle")
            .label("Throttle")
            .help("Specifies the minimum time between two event source notifications")
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Push Subscriptions")
            .fields([
                "jmap.push.throttle",
                "jmap.push.attempts.interval",
                "jmap.push.attempts.max",
                "jmap.push.retry.interval",
            ])
            .build()
            .new_form_section()
            .title("Push Timeouts")
            .fields(["jmap.push.timeout.request", "jmap.push.timeout.verify"])
            .build()
            .new_form_section()
            .title("Event Source")
            .fields(["jmap.event-source.throttle"])
            .build()
            .build()
            // Web Sockets
            .new_schema("jmap-web-sockets")
            .new_field("jmap.web-sockets.throttle")
            .label("Throttle")
            .help("Amount of time to wait before sending a batch of notifications to a WS client")
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.web-sockets.timeout")
            .label("Timeout")
            .help("Time before an inactive WebSocket connection times out")
            .default("10m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.web-sockets.heartbeat")
            .label("Heartbeat")
            .help("Time to wait before sending a new heartbeat to the WebSocket client")
            .default("1m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("JMAP over WebSocket")
            .fields([
                "jmap.web-sockets.throttle",
                "jmap.web-sockets.timeout",
                "jmap.web-sockets.heartbeat",
            ])
            .build()
            .build()
    }
}
