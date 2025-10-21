/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

impl Builder<Schemas, ()> {
    #![allow(clippy::useless_concat)]
    pub fn build_storage(self) -> Self {
        self.new_schema("storage")
            .new_field("storage.data")
            .label("Store")
            .help(concat!(
                "Core storage unit where email metadata, folders, and various settings ",
                "are stored. Essentially, it contains all the data except for ",
                "large binary objects (blobs)"
            ))
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "sql-read-replica",
            ])
            .input_check([], [Validator::Required])
            .build()
            .new_field("storage.blob")
            .label("Store")
            .help(concat!(
                "Used for storing large binary objects such as emails, sieve scripts, ",
                "and other files"
            ))
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "s3",
                "azure",
                "fs",
                "sql-read-replica",
                "sharded-blob",
            ])
            .input_check([], [Validator::Required])
            .build()
            .new_field("storage.fts")
            .label("Store")
            .help(concat!(
                "Dedicated to indexing for full-text search, enhancing the speed and ",
                "efficiency of text-based queries"
            ))
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "elasticsearch",
                "sql-read-replica",
            ])
            .input_check([], [Validator::Required])
            .build()
            .new_field("storage.lookup")
            .label("Store")
            .help(concat!(
                "Key-value storage used primarily by the SMTP server and anti-spam ",
                "components"
            ))
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "redis",
                "sql-read-replica",
                "sharded-in-memory",
            ])
            .input_check([], [Validator::Required])
            .build()
            .new_field("email.encryption.enable")
            .label("Enable encryption at rest")
            .help(concat!(
                "Allow users to configure encryption at rest for their data"
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("email.encryption.append")
            .label("Encrypt on append")
            .help(concat!(
                "Encrypt messages that are manually appended by the user using ",
                "JMAP or IMAP"
            ))
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("storage.full-text.default-language")
            .label("Default Language")
            .help(concat!(
                "Default language to use when language detection is not possible"
            ))
            .typ(Type::Input)
            .default("en")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("account.purge.frequency")
            .label("Frequency")
            .help(concat!(
                "Specifies how often tombstoned messages are deleted ",
                "from the database"
            ))
            .default("0 0 *")
            .typ(Type::Cron)
            .input_check([], [Validator::Required])
            .build()
            .new_field("changes.max-history")
            .label("Changes history")
            .help(concat!(
                "How many changes to keep in the history for each account. ",
                "This is used to determine the changes that have occurred ",
                "since the last time the client requested changes."
            ))
            .default("10000")
            .typ(Type::Input)
            .build()
            .new_field("email.auto-expunge")
            .label("Trash auto-expunge")
            .help(concat!(
                "How long to keep messages in the Trash and Junk Mail folders ",
                "before auto-expunging"
            ))
            .default("30d")
            .typ(Type::Duration)
            .build()
            .new_field("storage.undelete.retention")
            .label("Un-delete period")
            .help(concat!(
                "How long to keep deleted emails before they are permanently ",
                "removed from the system. (Enterprise feature)"
            ))
            .default("false")
            .typ(Type::Duration)
            .enterprise_feature()
            .build()
            .new_form_section()
            .title("Data Store")
            .fields([
                "storage.data",
                "email.encryption.enable",
                "email.encryption.append",
            ])
            .build()
            .new_form_section()
            .title("Blob Store")
            .fields(["storage.blob", "storage.undelete.retention"])
            .build()
            .new_form_section()
            .title("Full Text Index Store")
            .fields(["storage.fts", "storage.full-text.default-language"])
            .build()
            .new_form_section()
            .title("In-Memory Store")
            .fields(["storage.lookup"])
            .build()
            .new_form_section()
            .title("Cleanup")
            .fields([
                "account.purge.frequency",
                "changes.max-history",
                "email.auto-expunge",
            ])
            .build()
            .build()
            // E-mail Storage Quotas
            .new_schema("email-storage-quota")
            .new_field("object-quota.push-subscription")
            .label("Push Subscriptions")
            .help("The default maximum number of push subscriptions a user can create")
            .default("15")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.email")
            .label("Emails")
            .help("The default maximum number of emails a user can create")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.mailbox")
            .label("Mailboxes")
            .help("The default maximum number of mailboxes a user can create")
            .default("250")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.identity")
            .label("Email Identities")
            .help("The default maximum number of identities a user can create")
            .default("20")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.email-submission")
            .label("Email Submissions")
            .help("The default maximum number of email submissions a user can create")
            .default("500")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.sieve-script")
            .label("Sieve Scripts")
            .help("The default maximum number of sieve scripts a user can create")
            .default("100")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_form_section()
            .title("Default Object Quotas")
            .fields([
                "object-quota.mailbox",
                "object-quota.email",
                "object-quota.sieve-script",
                "object-quota.push-subscription",
                "object-quota.identity",
                "object-quota.email-submission",
            ])
            .build()
            .build()
            // E-mail Storage Quotas
            .new_schema("groupware-storage-quota")
            .new_field("object-quota.calendar")
            .label("Calendars")
            .help("The default maximum number of calendars a user can create")
            .default("250")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.calendar-event")
            .label("Calendar Events")
            .help("The default maximum number of calendar events a user can create")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.address-book")
            .label("Address Books")
            .help("The default maximum number of address books a user can create")
            .default("250")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.contact-card")
            .label("Contact Cards")
            .help("The default maximum number of contact cards a user can create")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("object-quota.file-node")
            .label("File Nodes")
            .help("The default maximum number of file nodes a user can create")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_form_section()
            .title("Default Object Quotas")
            .fields([
                "object-quota.calendar",
                "object-quota.calendar-event",
                "object-quota.address-book",
                "object-quota.contact-card",
                "object-quota.file-node",
            ])
            .build()
            .build()
    }
}
