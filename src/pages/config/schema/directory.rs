/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_directory(self) -> Self {
        self.new_schema("directory")
            .names("directory", "directories")
            .prefix("directory")
            .suffix("type")
            // Id
            .new_id_field()
            .label("Directory Id")
            .help("Unique identifier for the directory")
            .build()
            // Type
            .new_field("type")
            .readonly()
            .label("Type")
            .help("Type of directory")
            .default("internal")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("internal", "Internal"),
                    ("ldap", "LDAP Directory"),
                    ("sql", "SQL Database"),
                    ("lmtp", "LMTP Server"),
                    ("smtp", "SMTP Server"),
                    ("imap", "IMAP4 Server"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            // Internal store
            .new_field("store")
            .label("Storage backend")
            .help("Storage backend where accounts, groups and lists are stored")
            .display_if_eq("type", ["internal", "sql"])
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter_if_eq(
                "type",
                ["internal"],
                &["foundationdb", "mysql", "postgresql", "sqlite", "rocksdb"],
            )
            .source_filter_if_eq("type", ["sql"], &["mysql", "postgresql", "sqlite"])
            .input_check([], [Validator::Required])
            .build()
            // Caches
            .new_field("cache.entries")
            .label("Cache size")
            .help("Maximum number of entries to cache")
            .default("500")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue(0.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                ],
            )
            .build()
            .new_field("cache.ttl.positive")
            .label("Positive TTL")
            .help("Time-to-live for positive cache entries")
            .typ(Type::Duration)
            .default("1h")
            .build()
            .new_field("cache.ttl.negative")
            .label("Negative TTL")
            .help("Time-to-live for negative cache entries")
            .typ(Type::Duration)
            .default("10m")
            .build()
            // SQL column mappings
            .new_field("columns.class")
            .label("Type")
            .help("Column name for account type")
            .display_if_eq("type", ["sql"])
            .input_check([Transformer::Trim], [Validator::Required])
            .new_field("columns.secret")
            .label("Password")
            .help("Column name for account password")
            .new_field("columns.description")
            .label("Name")
            .help("Column name for account name")
            .new_field("columns.quota")
            .label("Quota")
            .help("Column name for account quota")
            .input_check([Transformer::Trim], [])
            .build()
            // Host
            .new_field("host")
            .label("Hostname")
            .help("Hostname of the remote server")
            .display_if_eq("type", ["imap", "smtp", "lmtp"])
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            .build()
            // Port
            .new_field("port")
            .label("Port")
            .help("Port of the remote server")
            .display_if_eq("type", ["imap", "smtp", "lmtp"])
            .default_if_eq("type", ["lmtp"], "11200")
            .default_if_eq("type", ["smtp"], "25")
            .default_if_eq("type", ["imap"], "143")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .build()
            // TLS
            .new_field("tls.enable")
            .label("Enable TLS")
            .help("Use TLS to connect to the remote server")
            .display_if_eq("type", ["imap", "smtp", "lmtp", "ldap"])
            .default("false")
            .typ(Type::Boolean)
            .new_field("tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help("Allow invalid TLS certificates when connecting to the server")
            .default("false")
            .build()
            // Connection pools
            .new_field("pool.max-connections")
            .label("Max Connections")
            .help(concat!(
                "Maximum number of connections that can be ",
                "maintained simultaneously in the connection pool"
            ))
            .display_if_eq("type", ["imap", "smtp", "lmtp", "ldap"])
            .placeholder("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(0.into()),
                    Validator::MaxValue(8192.into()),
                ],
            )
            .new_field("pool.timeout.create")
            .typ(Type::Duration)
            .label("Create Timeout")
            .help(concat!(
                "Maximum amount of time that the connection pool ",
                "will wait for a new connection to be created"
            ))
            .placeholder("30s")
            .new_field("pool.timeout.wait")
            .label("Wait Timeout")
            .help(concat!(
                "Maximum amount of time that the connection pool ",
                "will wait for a connection to become available"
            ))
            .placeholder("30s")
            .new_field("pool.timeout.recycle")
            .label("Recycle Timeout")
            .help(concat!(
                "Maximum amount of time that the connection pool ",
                "manager will wait for a connection to be recycled"
            ))
            .build()
            // Local domains
            .new_field("lookup.domains")
            .label("Local Domains")
            .help("List of local domains")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::IsHost])
            .display_if_eq("type", ["lmtp", "smtp", "imap"])
            .build()
            // LMTP/SMTP limits
            .new_field("limits.auth-errors")
            .label("Max Auth Errors")
            .help("Maximum number of authentication errors before disconnecting")
            .default("3")
            .display_if_eq("type", ["lmtp", "smtp"])
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(0.into()),
                    Validator::MaxValue(1000.into()),
                ],
            )
            .new_field("limits.rcpt")
            .label("Max Recipients")
            .help("Maximum number of recipients to check per session")
            .default("5")
            .build()
            .new_field("timeout")
            .label("Timeout")
            .help("Connection timeout to the server")
            .typ(Type::Duration)
            .display_if_eq("type", ["ldap", "smtp", "lmtp", "imap"])
            .default("15s")
            .build()
            // LDAP settings
            .new_field("url")
            .label("URL")
            .help("URL of the LDAP server")
            .display_if_eq("type", ["ldap"])
            .default("ldap://localhost:389")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .new_field("base-dn")
            .label("Base DN")
            .help("The base distinguished name (DN) from where searches should begin")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("dc=example,dc=org")
            .new_field("bind.dn")
            .label("Bind DN")
            .help(concat!(
                "The distinguished name of the user account that the ",
                "server will bind as to connect to the LDAP directory"
            ))
            .placeholder("cn=serviceuser,ou=svcaccts,dc=example,dc=org")
            .input_check([Transformer::Trim], [])
            .new_field("bind.secret")
            .label("Bind Secret")
            .typ(Type::Secret)
            .new_field("bind.auth.enable")
            .label("Enable Bind Auth")
            .help("Use bind authentication for verifying credentials with the LDAP server")
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("bind.auth.dn")
            .label("Bind Auth DN")
            .help(concat!(
                "The distinguished name (DN) template used for binding to the ",
                "LDAP server. The ? in the DN template is a placeholder that ",
                "will be replaced with the username provided during the ",
                "login process."
            ))
            .typ(Type::Input)
            .display_if_eq("bind.auth.enable", ["true"])
            .placeholder("cn=?,ou=svcaccts,dc=example,dc=org")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("filter.name")
            .display_if_eq("type", ["ldap"])
            .input_check([Transformer::Trim], [Validator::Required])
            .label("Name")
            .default("(&(|(objectClass=posixAccount)(objectClass=posixGroup))(uid=?))")
            .help("Filter used to search for objects based on the account name")
            .new_field("filter.email")
            .label("E-mail")
            .default(concat!(
                "(&(|(objectClass=posixAccount)(objectClass=posixGroup))",
                "(|(mail=?)(mailAlias=?)(mailList=?)))"
            ))
            .help(concat!(
                "Searches for objects associated with a specific primary ",
                "addresses, alias or mailing lists address"
            ))
            .new_field("filter.verify")
            .label("Verify (VRFY)")
            .default(concat!(
                "(&(|(objectClass=posixAccount)(objectClass=posixGroup))",
                "(|(mail=*?*)(mailAlias=*?*)))"
            ))
            .help(concat!(
                "A wildcard search filter to retrieve the objects that contain",
                " a certain string in their email addresses. This ",
                "filter is used by the SMTP VRFY command."
            ))
            .new_field("filter.expand")
            .label("Expand (EXPN)")
            .default(concat!(
                "(&(|(objectClass=posixAccount)(objectClass=posixGroup))",
                "(mailList=?))"
            ))
            .help(concat!(
                "This filter is used to search for objects that belong ",
                "to a specific mailing list. This filter is ",
                "used by the SMTP EXPN command."
            ))
            .new_field("filter.domains")
            .label("Local Domains")
            .default(concat!(
                "(&(|(objectClass=posixAccount)(objectClass=posixGroup))",
                "(|(mail=*@?)(mailAlias=*@?)))"
            ))
            .help(concat!(
                "Searches for objects that have an email address ",
                "in a specific domain name. This filter is used ",
                "by the SMTP server to validate local domains during ",
                "the RCPT TO command."
            ))
            .new_field("attributes.name")
            .label("Name")
            .help("LDAP attribute for the user's account name")
            .default("uid")
            .typ(Type::Array)
            .new_field("attributes.class")
            .label("Type")
            .help("LDAP attribute for the user's account type, if missing defaults to individual.")
            .default("objectClass")
            .new_field("attributes.description")
            .label("Description")
            .help("LDAP attributes used to store the user's description")
            .default("description")
            .new_field("attributes.secret")
            .label("Secret")
            .help("LDAP attribute for the user's password")
            .default("userPassword")
            .new_field("attributes.groups")
            .label("Groups")
            .help("LDAP attributes for the groups that a user belongs to")
            .default("memberOf")
            .new_field("attributes.email")
            .label("E-mail")
            .help("LDAP attribute for the user's primary email address")
            .default("mail")
            .new_field("attributes.email-alias")
            .input_check([Transformer::Trim], [])
            .label("E-mail Aliases")
            .help("LDAP attribute for the user's email alias(es)")
            .default("mailAlias")
            .new_field("attributes.quota")
            .label("Disk Quota")
            .help("DAP attribute for the user's disk quota")
            .default("diskQuota")
            .build()
            // Form layouts
            .new_form_section()
            .title("Configuration")
            .fields([
                "_id", "type", "store", "url", "base-dn", "host", "port", "timeout",
            ])
            .build()
            .new_form_section()
            .title("Binding")
            .display_if_eq("type", ["ldap"])
            .fields(["bind.dn", "bind.secret", "bind.auth.enable", "bind.auth.dn"])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["ldap", "imap", "smtp", "lmtp"])
            .fields(["tls.enable", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Column Mappings")
            .display_if_eq("type", ["sql"])
            .fields([
                "columns.class",
                "columns.secret",
                "columns.description",
                "columns.quota",
            ])
            .build()
            .new_form_section()
            .title("Lookup Filters")
            .display_if_eq("type", ["ldap"])
            .fields([
                "filter.name",
                "filter.email",
                "filter.verify",
                "filter.expand",
                "filter.domains",
            ])
            .build()
            .new_form_section()
            .title("Object Attributes")
            .display_if_eq("type", ["ldap"])
            .fields([
                "attributes.name",
                "attributes.class",
                "attributes.description",
                "attributes.secret",
                "attributes.groups",
                "attributes.email",
                "attributes.email-alias",
                "attributes.quota",
            ])
            .build()
            .new_form_section()
            .title("Local Domains")
            .display_if_eq("type", ["lmtp", "smtp", "imap"])
            .fields(["lookup.domains"])
            .build()
            .new_form_section()
            .title("Caching")
            .fields(["cache.entries", "cache.ttl.positive", "cache.ttl.negative"])
            .build()
            .new_form_section()
            .title("Limits")
            .display_if_eq("type", ["lmtp", "smtp"])
            .fields(["limits.auth-errors", "limits.rcpt"])
            .build()
            .new_form_section()
            .title("Connection Pools")
            .display_if_eq("type", ["imap", "smtp", "lmtp", "ldap"])
            .fields([
                "pool.max-connections",
                "pool.timeout.create",
                "pool.timeout.wait",
                "pool.timeout.recycle",
            ])
            .build()
            .list_title("Directories")
            .list_subtitle("Manage directories")
            .list_fields(["_id", "type"])
            .build()
    }
}
