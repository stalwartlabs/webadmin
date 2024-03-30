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
                multi: false,
            })
            .source_filter(&["foundationdb", "mysql", "postgresql", "sqlite", "rocksdb"])
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
                multi: false,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "s3",
                "fs",
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
                multi: false,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "elasticsearch",
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
                multi: false,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "redis",
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
            .fields(["storage.blob"])
            .build()
            .new_form_section()
            .title("Full Text Index Store")
            .fields(["storage.fts", "storage.full-text.default-language"])
            .build()
            .new_form_section()
            .title("Lookup Store")
            .fields(["storage.lookup"])
            .build()
            .build()
    }
}
