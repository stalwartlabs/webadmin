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
            .new_field("jmap.folders.inbox.name")
            .label("Name")
            .help("Default name for the inbox folder")
            .default("Inbox")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.inbox.create")
            .label("Create automatically")
            .help("Whether to create the inbox folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.inbox.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the inbox folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.trash.name")
            .label("Name")
            .help("Default name for the trash folder")
            .default("Deleted Items")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.trash.create")
            .label("Create automatically")
            .help("Whether to create the trash folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.trash.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the trash folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.junk.name")
            .label("Name")
            .help("Default name for the junk folder")
            .default("Junk Mail")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.junk.create")
            .label("Create automatically")
            .help("Whether to create the junk folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.junk.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the junk folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.drafts.name")
            .label("Name")
            .help("Default name for the drafts folder")
            .default("Drafts")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.drafts.create")
            .label("Create automatically")
            .help("Whether to create the drafts folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.drafts.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the drafts folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.sent.name")
            .label("Name")
            .help("Default name for the sent folder")
            .default("Sent Items")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.sent.create")
            .label("Create automatically")
            .help("Whether to create the sent folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.sent.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the sent folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.archive.name")
            .label("Name")
            .help("Default name for the archive folder")
            .default("Archive")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("jmap.folders.archive.create")
            .label("Create automatically")
            .help("Whether to create the archive folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.archive.subscribe")
            .label("Subscribe automatically")
            .help("Whether to subscribe to the archive folder automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("jmap.folders.shared.name")
            .label("Name")
            .help("Default name for the shared folder")
            .default("Shared Folders")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("Inbox")
            .fields([
                "jmap.folders.inbox.name",
                "jmap.folders.inbox.create",
                "jmap.folders.inbox.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Trash")
            .fields([
                "jmap.folders.trash.name",
                "jmap.folders.trash.create",
                "jmap.folders.trash.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Junk")
            .fields([
                "jmap.folders.junk.name",
                "jmap.folders.junk.create",
                "jmap.folders.junk.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Drafts")
            .fields([
                "jmap.folders.drafts.name",
                "jmap.folders.drafts.create",
                "jmap.folders.drafts.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Sent")
            .fields([
                "jmap.folders.sent.name",
                "jmap.folders.sent.create",
                "jmap.folders.sent.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Archive")
            .fields([
                "jmap.folders.archive.name",
                "jmap.folders.archive.create",
                "jmap.folders.archive.subscribe",
            ])
            .build()
            .new_form_section()
            .title("Shared Folders")
            .fields(["jmap.folders.shared.name"])
            .build()
            .build()
    }
}
