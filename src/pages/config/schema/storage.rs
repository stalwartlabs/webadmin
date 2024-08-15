/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

impl Builder<Schemas, ()> {
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
                "fs",
                "sql-read-replica",
                "distributed-blob",
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
            ])
            .input_check([], [Validator::Required])
            .build()
            .new_field("storage.encryption.enable")
            .label("Enable encryption at rest")
            .help(concat!(
                "Allow users to configure encryption at rest for their data"
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("storage.encryption.append")
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
            .new_field("jmap.account.purge.frequency")
            .label("Frequency")
            .help(concat!(
                "Specifies how often tombstoned messages are deleted ",
                "from the database"
            ))
            .default("0 0 *")
            .typ(Type::Cron)
            .input_check([], [Validator::Required])
            .build()
            .new_field("jmap.protocol.changes.max-history")
            .label("Changes history")
            .help(concat!(
                "How long to keep changes history for JMAP and IMAP clients"
            ))
            .default("30d")
            .typ(Type::Duration)
            .build()
            .new_field("jmap.email.auto-expunge")
            .label("Trash auto-expunge")
            .help(concat!(
                "How long to keep messages in the Trash and Junk Mail folders ",
                "before auto-expunging"
            ))
            .default("30d")
            .typ(Type::Duration)
            .build()
            .new_field("enterprise.undelete-period")
            .label("Un-delete period ⭐")
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
                "storage.encryption.enable",
                "storage.encryption.append",
            ])
            .build()
            .new_form_section()
            .title("Blob Store")
            .fields(["storage.blob", "enterprise.undelete-period"])
            .build()
            .new_form_section()
            .title("Full Text Index Store")
            .fields(["storage.fts", "storage.full-text.default-language"])
            .build()
            .new_form_section()
            .title("Lookup Store")
            .fields(["storage.lookup"])
            .build()
            .new_form_section()
            .title("Cleanup")
            .fields([
                "jmap.account.purge.frequency",
                "jmap.protocol.changes.max-history",
                "jmap.email.auto-expunge",
            ])
            .build()
            .build()
    }
}
