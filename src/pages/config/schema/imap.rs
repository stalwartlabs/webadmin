/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_imap(self) -> Self {
        // Authentication
        self.new_schema("imap-auth")
            .new_field("imap.auth.max-failures")
            .label("Max Failures")
            .help(concat!(
                "Number of authentication attempts a user can make before being ",
                "disconnected by the server"
            ))
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .build()
            .new_field("imap.auth.allow-plain-text")
            .label("Allow plain text authentication")
            .help("Whether to allow plain text authentication on unencrypted connections")
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_form_section()
            .title("Authentication settings")
            .fields(["imap.auth.max-failures", "imap.auth.allow-plain-text"])
            .build()
            .build()
            // Rate limiting
            .new_schema("imap-rate-limit")
            .new_field("imap.rate-limit.requests")
            .label("Requests")
            .help("The maximum number of requests per minute")
            .default("2000/1m")
            .typ(Type::Rate)
            .build()
            .new_field("imap.rate-limit.concurrent")
            .label("Concurrent")
            .help("The maximum number of concurrent connections")
            .default("6")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_form_section()
            .title("Rate Limiting")
            .fields(["imap.rate-limit.requests", "imap.rate-limit.concurrent"])
            .build()
            .build()
            // Protocol limits
            .new_schema("imap-limits")
            .new_field("imap.request.max-size")
            .label("Request Size")
            .help("Maximum size of an IMAP request that the server will accept")
            .default("52428800")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .build()
            .new_field("imap.timeout.authenticated")
            .label("Authenticated")
            .help(concat!(
                "Time an authenticated session can remain idle before the server ",
                "terminates it"
            ))
            .default("30m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("imap.timeout.anonymous")
            .label("Anonymous")
            .help(concat!(
                "Time an unauthenticated session can stay inactive before being ",
                "ended by the server"
            ))
            .default("1m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("imap.timeout.idle")
            .label("Idle")
            .help(concat!(
                "Time a connection can stay idle in the IMAP IDLE state before ",
                "the server breaks the connection"
            ))
            .default("30m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Limits")
            .fields(["imap.request.max-size"])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields([
                "imap.timeout.authenticated",
                "imap.timeout.anonymous",
                "imap.timeout.idle",
            ])
            .build()
            .build()
            // Folders
            .new_schema("imap-folders")
            .new_field("email.folders.inbox.name")
            .label("Name")
            .help("Default name for the inbox folder")
            .default("Inbox")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.inbox.create")
            .label("Create automatically")
            .help("Whether to create the inbox folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.inbox.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the inbox folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.trash.name")
            .label("Name")
            .help("Default name for the trash folder")
            .default("Deleted Items")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.trash.create")
            .label("Create automatically")
            .help("Whether to create the trash folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.trash.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the trash folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.junk.name")
            .label("Name")
            .help("Default name for the junk folder")
            .default("Junk Mail")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.junk.create")
            .label("Create automatically")
            .help("Whether to create the junk folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.junk.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the junk folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.drafts.name")
            .label("Name")
            .help("Default name for the drafts folder")
            .default("Drafts")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.drafts.create")
            .label("Create automatically")
            .help("Whether to create the drafts folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.drafts.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the drafts folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.sent.name")
            .label("Name")
            .help("Default name for the sent folder")
            .default("Sent Items")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.sent.create")
            .label("Create automatically")
            .help("Whether to create the sent folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.sent.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the sent folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.archive.name")
            .label("Name")
            .help("Default name for the archive folder")
            .default("Archive")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("email.folders.archive.create")
            .label("Create automatically")
            .help("Whether to create the archive folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.archive.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the archive folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.folders.shared.name")
            .label("Name")
            .help("Default name for the shared folder")
            .default("Shared Folders")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("Inbox")
            .fields([
                "email.folders.inbox.name",
                "email.folders.inbox.create",
                "email.folders.inbox.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Trash")
            .fields([
                "email.folders.trash.name",
                "email.folders.trash.create",
                "email.folders.trash.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Junk")
            .fields([
                "email.folders.junk.name",
                "email.folders.junk.create",
                "email.folders.junk.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Drafts")
            .fields([
                "email.folders.drafts.name",
                "email.folders.drafts.create",
                "email.folders.drafts.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Sent")
            .fields([
                "email.folders.sent.name",
                "email.folders.sent.create",
                "email.folders.sent.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Archive")
            .fields([
                "email.folders.archive.name",
                "email.folders.archive.create",
                "email.folders.archive.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Shared Folders")
            .fields(["email.folders.shared.name"])
            .build()
            .build()
    }
}
