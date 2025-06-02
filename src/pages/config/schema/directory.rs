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
                    ("oidc", "OpenID Connect"),
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
            .source_filter_if_eq(
                "type",
                ["sql"],
                &["mysql", "postgresql", "sqlite", "sql-read-replica"],
            )
            .input_check([], [Validator::Required])
            .build()
            // Caches
            .new_field("cache.size")
            .label("Cache size")
            .help("Maximum cache size in bytes")
            .default("1048576")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(0.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
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
            .new_field("columns.description")
            .label("Description")
            .help("Column name for account full name or description")
            .new_field("columns.quota")
            .label("Quota")
            .help("Column name for account quota")
            .input_check([Transformer::Trim], [])
            .new_field("columns.email")
            .label("E-mail")
            .help(concat!(
                "Column name for e-mail address. ",
                "Optional, you can use instead a query to obtain the account's addresses."
            ))
            .new_field("columns.secret")
            .label("Password")
            .help(concat!(
                "Column name for the account password. ",
                "Optional, you can use instead a query to obtain the account's secrets."
            ))
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
            .display_if_eq("type", ["ldap", "smtp", "lmtp", "imap", "oidc"])
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
            .build()
            .new_field("bind.auth.method")
            .label("Method")
            .help("Method used for verifying credentials with the LDAP server")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("default", "Lookup using bind DN"),
                    ("template", "Bind authentication with template"),
                    ("lookup", "Bind authentication after lookup"),
                ]),
                typ: SelectType::Single,
            })
            .default("default")
            .build()
            .new_field("bind.auth.template")
            .label("Bind DN template")
            .help(concat!(
                "The distinguished name (DN) template used for binding to the ",
                "LDAP server. The {username} in the DN template is a placeholder that ",
                "will be replaced with the username provided during the ",
                "login process. If the username is an email address, ",
                "{local} and {domain} placeholders can be used to ",
                "extract the local part and domain from the email address."
            ))
            .typ(Type::Input)
            .display_if_eq("bind.auth.method", ["template"])
            .placeholder("cn={username},ou=svcaccts,dc=example,dc=org")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("bind.auth.search")
            .label("Reuse bind auth connection for search")
            .help(concat!(
                "Weather to perform LDAP searches with the bind auth DN connection. ",
                "If disabled, LDAP searches will be done using a separate connection ",
                "using the default Bind DN."
            ))
            .typ(Type::Boolean)
            .display_if_eq("bind.auth.method", ["template"])
            .default("true")
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
            .new_field("attributes.name")
            .label("Name")
            .help("LDAP attribute for the user's account name")
            .default("uid")
            .typ(Type::Array)
            .new_field("attributes.class")
            .label("Type")
            .help("LDAP attribute for the user's account type, if missing defaults to individual.")
            .default("objectClass")
            .new_field("attributes.email")
            .label("E-mail")
            .help("LDAP attribute for the user's primary email address")
            .default("mail")
            .new_field("attributes.description")
            .label("Description")
            .help("LDAP attributes used to store the user's description")
            .default("description")
            .new_field("attributes.secret")
            .input_check([Transformer::Trim], [])
            .label("Secret")
            .help(concat!(
                "LDAP attribute for the user's password hash. ",
                "This setting is required when binding as a service user. ",
                "When using bind authentication, configure the secret-changed ",
                "attribute instead."
            ))
            .default("userPassword")
            .new_field("attributes.secret-changed")
            .label("Secret Changed")
            .help(concat!(
                "LDAP attribute that provides a password change hash or a timestamp ",
                "indicating when the password was last changed. ",
                "When using bind authentication, this attribute is used to ",
                "determine when to invalidate OAuth tokens."
            ))
            .default("pwdChangeTime")
            .new_field("attributes.groups")
            .label("Groups")
            .help("LDAP attributes for the groups that a user belongs to")
            .default("memberOf")
            .new_field("attributes.email-alias")
            .label("E-mail Aliases")
            .help("LDAP attribute for the user's email alias(es)")
            .default("mailAlias")
            .new_field("attributes.quota")
            .label("Disk Quota")
            .help("DAP attribute for the user's disk quota")
            .default("diskQuota")
            .build()
            // OIDC
            // Type
            .new_field("endpoint.url")
            .label("URL")
            .help(concat!(
                "URL of the OpenID Connect provider. This is used to ",
                "retrieve user information from the OpenID Connect provider."
            ))
            .display_if_eq("type", ["oidc"])
            .placeholder("https://accounts.example.org/userinfo")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("endpoint.method")
            .label("Type")
            .help(concat!(
                "Type of endpoint to use for user information. ",
                "This is used to retrieve user information from the ",
                "OpenID Connect provider."
            ))
            .default("userinfo")
            .display_if_eq("type", ["oidc"])
            .typ(Type::Select {
                source: Source::Static(&[
                    ("userinfo", "OpenID Connect Userinfo"),
                    ("introspect", "OAuth Token Introspection"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("fields.email")
            .label("E-mail field")
            .help(concat!(
                "Field name in the OpenID Connect provider response ",
                "that contains the user's email address."
            ))
            .display_if_eq("type", ["oidc"])
            .placeholder("email")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("fields.username")
            .label("Username field")
            .help(concat!(
                "Field name in the OpenID Connect provider response ",
                "that contains the user's username. If not provided, ",
                "the email field will be used."
            ))
            .display_if_eq("type", ["oidc"])
            .placeholder("preferred_username")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("fields.full-name")
            .label("Name field")
            .help(concat!(
                "Field name in the OpenID Connect provider response ",
                "that contains the user's full name."
            ))
            .display_if_eq("type", ["oidc"])
            .placeholder("name")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("auth.method")
            .label("Method")
            .help(concat!(
                "Type of endpoint to use for user information. ",
                "This is used to retrieve user information from the ",
                "OpenID Connect provider."
            ))
            .default("none")
            .display_if_eq("endpoint.method", ["introspect"])
            .typ(Type::Select {
                source: Source::Static(&[
                    ("none", "No Authentication"),
                    ("basic", "Basic Authentication"),
                    ("token", "Bearer Token"),
                    ("user-token", "User Access Token"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("auth.token")
            .label("Auth token")
            .typ(Type::Secret)
            .help(concat!(
                "Bearer token used to authenticate with the OAuth introspect endpoint.",
            ))
            .display_if_eq("auth.method", ["token"])
            .build()
            .new_field("auth.username")
            .label("Auth username")
            .help(concat!(
                "Username used to authenticate with the OAuth introspect endpoint.",
            ))
            .typ(Type::Input)
            .display_if_eq("auth.method", ["basic"])
            .build()
            .new_field("auth.secret")
            .label("Auth secret")
            .help(concat!(
                "Password used to authenticate with the OAuth introspect endpoint.",
            ))
            .typ(Type::Secret)
            .display_if_eq("auth.method", ["basic"])
            .build()
            // Form layouts
            .new_form_section()
            .title("Configuration")
            .fields([
                "_id",
                "type",
                "store",
                "url",
                "host",
                "port",
                "endpoint.url",
                "endpoint.method",
                "timeout",
            ])
            .build()
            .new_form_section()
            .title("LDAP Binding")
            .display_if_eq("type", ["ldap"])
            .fields(["bind.dn", "bind.secret"])
            .build()
            .new_form_section()
            .title("LDAP Authentication Method")
            .display_if_eq("type", ["ldap"])
            .fields(["bind.auth.method", "bind.auth.template", "bind.auth.search"])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["ldap", "imap", "smtp", "lmtp"])
            .fields(["tls.enable", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Endpoint Authentication")
            .display_if_eq("endpoint.method", ["introspect"])
            .fields(["auth.method", "auth.token", "auth.username", "auth.secret"])
            .build()
            .new_form_section()
            .title("Field Mappings")
            .display_if_eq("type", ["oidc"])
            .fields(["fields.email", "fields.username", "fields.full-name"])
            .build()
            .new_form_section()
            .title("Column Mappings")
            .display_if_eq("type", ["sql"])
            .fields([
                "columns.class",
                "columns.description",
                "columns.secret",
                "columns.email",
                "columns.quota",
            ])
            .build()
            .new_form_section()
            .title("LDAP Filters")
            .display_if_eq("type", ["ldap"])
            .fields(["base-dn", "filter.name", "filter.email"])
            .build()
            .new_form_section()
            .title("Object Attributes")
            .display_if_eq("type", ["ldap"])
            .fields([
                "attributes.name",
                "attributes.class",
                "attributes.description",
                "attributes.secret",
                "attributes.secret-changed",
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
            .fields(["cache.size", "cache.ttl.positive", "cache.ttl.negative"])
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
