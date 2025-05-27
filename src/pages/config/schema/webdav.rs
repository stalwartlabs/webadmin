/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_webdav(self) -> Self {
        // WebDAV
        self.new_schema("webdav")
            .new_field("dav.request.max-size")
            .label("Max Request Size")
            .help(concat!(
                "Determines the maximum XML size of a WebDAV request that the server will accept"
            ))
            .default("26214400")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .new_field("file-storage.max-size")
            .label("Max File Size")
            .help(concat!(
                "Specifies the maximum size of a file that can be uploaded to the server"
            ))
            .default("26214400")
            .new_field("dav.property.max-size.live")
            .label("Live Property")
            .help(concat!(
                "Specifies the maximum size of a WebDAV live property value that the server will accept"
            ))
            .default("250")
            .new_field("dav.property.max-size.dead")
            .label("Dead Property")
            .default("1024")
            .help(concat!(
                "Specifies the maximum size of a WebDAV dead property value that the server will accept"
            ))
            .input_check([], [])
            .build()
            .new_field("dav.lock.max-timeout")
            .label("Max Lock Timeout")
            .default("1h")
            .help(concat!(
                "Specifies the maximum duration for which a lock can be held on a resource"
            ))
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("dav.locks.max-per-user")
            .label("Max Locks Per User")
            .help(concat!(
                "Specifies the maximum number of locks that a user can create on a resource"
            ))
            .default("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required],
            )
            .new_field("dav.response.max-results")
            .label("Max Results")
            .help(concat!(
                "Specifies the maximum number of results that a WebDAV query can return"
            ))
            .default("2000")
            .build()
            .new_form_section()
            .title("Requests")
            .fields([
                "dav.request.max-size",
                "dav.response.max-results"
            ])
            .build()
            .new_form_section()
            .title("Property Limits")
            .fields([
                "dav.property.max-size.live",
                "dav.property.max-size.dead",
            ])
            .build()
            .new_form_section()
            .title("Locking")
            .fields([
                "dav.lock.max-timeout",
                "dav.locks.max-per-user",
            ])
            .build()
            .new_form_section()
            .title("File Storage")
            .fields([
                "file-storage.max-size",
            ])
            .build()
            .build()
            // CalDAV
            .new_schema("caldav")
            .new_field("calendar.max-size")
            .label("Max iCal Size")
            .help(concat!(
                "Specifies the maximum size of an iCalendar file that can be uploaded to the server"
            ))
            .default("524288")
            .typ(Type::Size)
            .input_check([], [Validator::Required])

            .new_field("calendar.max-recurrence-expansions")
            .label("Max iCal Instances")
            .help(concat!(
                "Specifies the maximum number of instances that can be generated from a recurring iCalendar event"
            ))
            .default("3000")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required],
            )
            .new_field("calendar.max-attendees-per-instance")
            .label("Max iCal Attendees")
            .help(concat!(
                "Specifies the maximum number of attendees that can be included in a single iCalendar instance"
            ))
            .default("20")
            .typ(Type::Input)
            .new_field("calendar.default.href-name")
            .label("Default Href Name")
            .help(concat!(
                "Specifies the default href name for a calendar when it is created"
            ))
            .default("default")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [],
            )
            .new_field("calendar.default.display-name")
            .label("Default Display Name")
            .help(concat!(
                "Specifies the default display name for a calendar when it is created"
            ))
            .default("Stalwart Calendar")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required],
            )
            .build()
            .new_form_section()
            .title("iCalendar Limits")
            .fields([
                "calendar.max-size",
                "calendar.max-recurrence-expansions",
                "calendar.max-attendees-per-instance",
            ])
            .build()
            .new_form_section()
            .title("Default Names")
            .fields([
                "calendar.default.href-name",
                "calendar.default.display-name",
            ])
            .build()
            .build()
            // CardDAV
             .new_schema("carddav")
            .new_field("contacts.max-size")
            .label("Max vCard Size")
            .help(concat!(
                "Specifies the maximum size of a vCard file that can be uploaded to the server"
            ))
            .default("524288")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .new_field("contacts.default.href-name")
            .label("Default Href Name")
            .help(concat!(
                "Specifies the default href name for a contact when it is created"
            ))
            .default("default")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [],
            )
            .new_field("contacts.default.display-name")
            .label("Default Display Name")
            .help(concat!(
                "Specifies the default display name for a contact when it is created"
            ))
            .default("Stalwart Address Book")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required],
            )
            .build()
            .new_form_section()
            .title("vCard Limits")
            .fields([
                "contacts.max-size",
                "contacts.default.href-name",
                "contacts.default.display-name",
            ])
            .build()
            .build()
    }
}
