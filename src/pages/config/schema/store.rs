/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

const PGSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = $1 AND active = true";
const PGSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = $1";
const PGSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = $1 ORDER BY name ASC";
const PGSQL_EMAILS: &str = "SELECT address FROM emails WHERE name = $1 ORDER BY address ASC";
const PGSQL_SECRETS: &str = "SELECT secret FROM secrets WHERE name = $1";

const MYSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const MYSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const MYSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ? ORDER BY name ASC";
const MYSQL_EMAILS: &str = "SELECT address FROM emails WHERE name = ? ORDER BY address ASC";
const MYSQL_SECRETS: &str = "SELECT secret FROM secrets WHERE name = ?";

const SQLITE_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const SQLITE_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const SQLITE_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ?";
const SQLITE_EMAILS: &str = "SELECT address FROM emails WHERE name = ? ORDER BY address ASC";
const SQLITE_SECRETS: &str = "SELECT secret FROM secrets WHERE name = ?";

impl Builder<Schemas, ()> {
    pub fn build_store(self) -> Self {
        self.new_schema("store")
            .names("store", "stores")
            .prefix("store")
            .suffix("type")
            // Id
            .new_id_field()
            .label("Store Id")
            .help("Unique identifier for the store")
            .build()
            // Type
            .new_field("type")
            .readonly()
            .label("Type")
            .help("Storage backend type")
            .default("rocksdb")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("rocksdb", "RocksDB"),
                    ("foundationdb", "FoundationDB"),
                    ("postgresql", "PostgreSQL"),
                    ("mysql", "mySQL"),
                    ("sqlite", "SQLite"),
                    ("s3", "S3-compatible"),
                    ("redis", "Redis/Valkey"),
                    ("nats", "NATS PubSub"),
                    ("elasticsearch", "ElasticSearch"),
                    ("azure", "Azure blob storage"),
                    ("fs", "Filesystem"),
                    ("sql-read-replica", "SQL with Replicas"),
                    ("sharded-blob", "Sharded Blob Store"),
                    ("sharded-in-memory", "Sharded In-Memory Store"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            // Compression
            .new_field("compression")
            .readonly()
            .label("Compression")
            .help("Algorithm to use to compress large binary objects")
            .default("lz4")
            .typ(Type::Select {
                source: Source::Static(&[("none", "None"), ("lz4", "LZ4")]),
                typ: SelectType::Single,
            })
            .display_if_ne("type", ["redis", "memory", "elasticsearch"])
            .build()
            // Path
            .new_field("path")
            .label("Path")
            .help("Where to store the data in the server's filesystem")
            .display_if_eq("type", ["rocksdb", "sqlite", "fs"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Host
            .new_field("host")
            .label("Hostname")
            .help("Hostname of the database server")
            .display_if_eq("type", ["postgresql", "mysql"])
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            .build()
            // Port
            .new_field("port")
            .label("Port")
            .help("Port of the database server")
            .display_if_eq("type", ["postgresql", "mysql"])
            .default_if_eq("type", ["postgresql"], "5432")
            .default_if_eq("type", ["mysql"], "3307")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .build()
            // Database name
            .new_field("database")
            .label("Database")
            .help("Name of the database")
            .default("stalwart")
            .display_if_eq("type", ["postgresql", "mysql"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Redis type
            .new_field("redis-type")
            .label("Server Type")
            .help("Type of Redis server")
            .display_if_eq("type", ["redis"])
            .default("single")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("single", "Redis single node"),
                    ("cluster", "Redis Cluster"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            // Redis protocol version
            .new_field("protocol-version")
            .label("Protocol version")
            .help("Protocol Version")
            .display_if_eq("redis-type", ["cluster"])
            .default("resp2")
            .typ(Type::Select {
                source: Source::Static(&[("resp2", "RESP2"), ("resp3", "RESP3")]),
                typ: SelectType::Single,
            })
            .build()
            // Username
            .new_field("user")
            .label("Username")
            .help("Username to connect to the database")
            .default("stalwart")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch", "nats"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            // Password
            .new_field("password")
            .label("Password")
            .help("Password to connect to the database")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch", "nats"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Secret)
            .build()
            // Timeout
            .new_field("timeout")
            .label("Timeout")
            .help("Connection timeout to the database")
            .display_if_eq("type", ["postgresql", "mysql", "redis", "s3", "azure"])
            .typ(Type::Duration)
            .default("15s")
            .build()
            // Purge frequency
            .new_field("purge.frequency")
            .label("Purge Frequency")
            .help("How often to purge the database. Expects a cron expression")
            .display_if_ne("type", ["redis", "memory", "elasticsearch"])
            .default("0 3 *")
            .typ(Type::Cron)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Workers
            .new_field("pool.workers")
            .label("Thread Pool Size")
            .help("Number of worker threads to use for the store, defaults to the number of cores")
            .display_if_eq("type", ["rocksdb", "sqlite"])
            .placeholder("8")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(64.into()),
                ],
            )
            .build()
            // Number of connections
            .new_field("pool.max-connections")
            .label("Max Connections")
            .help("Maximum number of connections to the store")
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .default("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(8192.into()),
                ],
            )
            .build()
            .new_field("pool.min-connections")
            .label("Min Connections")
            .help("Minimum number of connections to the store")
            .display_if_eq("type", ["mysql"])
            .default("5")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(8192.into()),
                ],
            )
            .build()
            // TLS
            .new_field("tls.enable")
            .label("Enable TLS")
            .help("Use TLS to connect to the store")
            .display_if_eq("type", ["postgresql", "mysql", "nats"])
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help("Allow invalid TLS certificates when connecting to the store")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .default("false")
            .typ(Type::Boolean)
            .build()
            // URL
            .new_field("url")
            .label("URL")
            .help("URL of the store")
            .display_if_eq("type", ["elasticsearch"])
            .default_if_eq("type", ["elasticsearch"], "https://localhost:9200")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            // Maximum number of retries
            .new_field("max-retries")
            .label("Retry limit")
            .help(concat!(
                "The maximum number of times to retry failed requests. ",
                "Set to 0 to disable retries"
            ))
            .display_if_eq("type", ["s3", "azure"])
            .placeholder("3")
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(10.into()),
                ],
            )
            .build()
            // Key prefix (for blob stores)
            .new_field("key-prefix")
            .label("Key Prefix")
            .help("A prefix that will be added to the keys of all objects stored in the blob store")
            .display_if_eq("type", ["s3", "azure"])
            .input_check([Transformer::Trim], [])
            .build()
            // SQL directory specific
            .new_field("query.name")
            .label("Account by Name")
            .help("Query to obtain the account details by login name")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .placeholder_if_eq("type", ["postgresql"], PGSQL_NAME)
            .placeholder_if_eq("type", ["mysql"], MYSQL_NAME)
            .placeholder_if_eq("type", ["sqlite"], SQLITE_NAME)
            .new_field("query.members")
            .label("Members by Name")
            .help("Query to obtain the members of a group by account name")
            .placeholder_if_eq("type", ["postgresql"], PGSQL_MEMBERS)
            .placeholder_if_eq("type", ["mysql"], MYSQL_MEMBERS)
            .placeholder_if_eq("type", ["sqlite"], SQLITE_MEMBERS)
            .new_field("query.recipients")
            .label("Name by E-mail")
            .help(concat!(
                "Query to obtain the account name ",
                "associated with an e-mail address."
            ))
            .placeholder_if_eq("type", ["postgresql"], PGSQL_RECIPIENTS)
            .placeholder_if_eq("type", ["mysql"], MYSQL_RECIPIENTS)
            .placeholder_if_eq("type", ["sqlite"], SQLITE_RECIPIENTS)
            .new_field("query.emails")
            .label("E-mails by Name")
            .help(concat!(
                "Query to obtain the e-mail address(es) of an account. ",
                "Optional, you may also obtain a single ",
                "address from the 'email' column."
            ))
            .placeholder_if_eq("type", ["postgresql"], PGSQL_EMAILS)
            .placeholder_if_eq("type", ["mysql"], MYSQL_EMAILS)
            .placeholder_if_eq("type", ["sqlite"], SQLITE_EMAILS)
            .new_field("query.secrets")
            .label("Passwords by Name")
            .help(concat!(
                "Query to obtain all the account's secrets. ",
                "Optional, you may also obtain a single secret ",
                "from the 'secret' column."
            ))
            .placeholder_if_eq("type", ["postgresql"], PGSQL_SECRETS)
            .placeholder_if_eq("type", ["mysql"], MYSQL_SECRETS)
            .placeholder_if_eq("type", ["sqlite"], SQLITE_SECRETS)
            .build()
            // RocksDB specific
            .new_field("settings.min-blob-size")
            .label("Min blob size")
            .help(concat!(
                "Minimum size of a blob to store in the blob store, ",
                "smaller blobs are stored in the metadata store"
            ))
            .display_if_eq("type", ["rocksdb"])
            .default("16834")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1024.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                ],
            )
            .new_field("settings.write-buffer-size")
            .label("Write buffer size")
            .help(concat!(
                "Size of the write buffer in bytes, ",
                "used to batch writes to the store"
            ))
            .default("134217728")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(8192.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
                ],
            )
            .build()
            // FoundationDB specific
            .new_field("cluster-file")
            .label("Cluster file")
            .help("Path to the cluster file for the FoundationDB cluster")
            .display_if_eq("type", ["foundationdb"])
            .placeholder("/etc/foundationdb/fdb.cluster")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .new_field("transaction.timeout")
            .label("Timeout")
            .help("Transaction timeout")
            .placeholder("5s")
            .typ(Type::Duration)
            .new_field("transaction.max-retry-delay")
            .label("Max Retry Delay")
            .help("Transaction maximum retry delay")
            .placeholder("1s")
            .typ(Type::Duration)
            .new_field("transaction.retry-limit")
            .label("Retry limit")
            .help("Transaction retry limit")
            .placeholder("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(1000.into()),
                ],
            )
            .new_field("ids.machine")
            .label("Machine Id")
            .help("Machine ID in the FoundationDB cluster (optional)")
            .placeholder("my-server-id")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsId])
            .new_field("ids.data-center")
            .label("Data Center Id")
            .help("Data center ID (optional)")
            .placeholder("my-datacenter-id")
            .build()
            // mySQL specific
            .new_field("max-allowed-packet")
            .label("Max Allowed Packet")
            .help("Maximum size of a packet in bytes")
            .display_if_eq("type", ["mysql"])
            .placeholder("1073741824")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1024.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
                ],
            )
            .build()
            // ElasticSearch specific
            .new_field("cloud-id")
            .label("Cloud Id")
            .help("Cloud ID for the ElasticSearch cluster")
            .display_if_eq("type", ["elasticsearch"])
            .placeholder("my-cloud-id")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .new_field("index.shards")
            .label("Number of Shards")
            .help("Number of shards for the index")
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                ],
            )
            .new_field("index.replicas")
            .label("Number of Replicas")
            .help("Number of replicas for the index")
            .default("0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(0.into()),
                    Validator::MaxValue(2048.into()),
                ],
            )
            .build()
            // Redis specific
            .new_field("urls")
            .label("URL(s)")
            .help("URL(s) of the Redis server(s)")
            .display_if_eq("type", ["redis"])
            .default("redis://127.0.0.1")
            .typ(Type::Array(ArrayType::Text))
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("retry.total")
            .label("Retries")
            .help("Number of retries to connect to the Redis cluster")
            .display_if_eq("redis-type", ["cluster"])
            .placeholder("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(1024.into()),
                ],
            )
            .new_field("retry.max-wait")
            .label("Max Wait")
            .help("Maximum time to wait between retries")
            .placeholder("1s")
            .typ(Type::Duration)
            .new_field("retry.min-wait")
            .label("Min Wait")
            .help("Minimum time to wait between retries")
            .placeholder("500ms")
            .build()
            .new_field("read-from-replicas")
            .label("Read from replicas")
            .help("Whether to read from replicas")
            .default("true")
            .typ(Type::Boolean)
            .build()
            // Nats specific
            .new_field("address")
            .label("Server Address")
            .help("Address of the NATS server")
            .display_if_eq("type", ["nats"])
            .default("127.0.0.1:4444")
            .typ(Type::Array(ArrayType::Text))
            .build()
            .new_field("no-echo")
            .label("No Echo")
            .help("Disables delivering messages that were published from the same connection.")
            .display_if_eq("type", ["nats"])
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("max-reconnects")
            .label("Max Reconnects")
            .help("Maximum number of times to attempt to reconnect to the server")
            .display_if_eq("type", ["nats"])
            .build()
            .new_field("timeout.connection")
            .label("Connection Timeout")
            .help("Timeout for establishing a connection to the server")
            .display_if_eq("type", ["nats"])
            .default("5s")
            .typ(Type::Duration)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("timeout.request")
            .label("Request Timeout")
            .help("Timeout for requests to the server")
            .display_if_eq("type", ["nats"])
            .default("10s")
            .typ(Type::Duration)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("ping-interval")
            .label("Ping Interval")
            .help("Interval between pings to the server")
            .display_if_eq("type", ["nats"])
            .default("60s")
            .typ(Type::Duration)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("credentials")
            .label("JWT Credentials")
            .help("String containing the JWT credentials")
            .display_if_eq("type", ["nats"])
            .typ(Type::Text)
            .build()
            .new_field("capacity.client")
            .label("Client Capacity")
            .help(concat!(
                "By default, Client dispatches op's to the Client onto the ",
                "channel with capacity of 2048. This option enables overriding it"
            ))
            .display_if_eq("type", ["nats"])
            .default("2048")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("capacity.subscription")
            .label("Subscription Capacity")
            .help(concat!(
                "Sets the capacity for Subscribers. Exceeding it will ",
                "trigger slow consumer error callback and drop messages."
            ))
            .display_if_eq("type", ["nats"])
            .default("65536")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("capacity.read-buffer")
            .label("Read Buffer Capacity")
            .help(concat!(
                "Sets the initial capacity of the read buffer. Which ",
                "is a buffer used to gather partial protocol messages."
            ))
            .display_if_eq("type", ["nats"])
            .default("65535")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            // S3 specific
            .new_field("bucket")
            .typ(Type::Input)
            .label("Name")
            .help("The S3 bucket where blobs (e-mail messages, Sieve scripts, etc.) will be stored")
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("stalwart")
            .display_if_eq("type", ["s3"])
            .new_field("region")
            .label("Region")
            .help("The geographical region where the bucket resides")
            .placeholder("us-east-1")
            .new_field("endpoint")
            .help(concat!(
                "The network address (hostname and optionally a port) of the S3 service. ",
                "If you are using a well-known S3 service like Amazon S3, this setting can ",
                "be left blank, and the endpoint will be derived from the region. For ",
                "S3-compatible services, you will need to specify the endpoint explicitly"
            ))
            .label("Endpoint")
            .new_field("access-key")
            .label("Access Key")
            .help("Identifies the S3 account")
            .new_field("secret-key")
            .label("Secret Key")
            .help("The secret key for the S3 account")
            .typ(Type::Secret)
            .new_field("security-token")
            .label("Security Token")
            .input_check([Transformer::Trim], [])
            .new_field("profile")
            .label("Profile")
            .typ(Type::Input)
            .help(concat!(
                "Used when retrieving credentials from a shared credentials file. If specified, ",
                "the server will use the access key ID, secret access key, and session token (if ",
                "available) associated with the given profile"
            ))
            .build()
            // Azure specific
            .new_field("storage-account")
            .typ(Type::Input)
            .label("Storage Account Name")
            .help(concat!(
                "The Azure Storage Account where blobs (e-mail messages, ",
                "Sieve scripts, etc.) will be stored"
            ))
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("mycompany")
            .display_if_eq("type", ["azure"])
            .new_field("container")
            .typ(Type::Input)
            .label("Container")
            .help("The name of the container in the Storage Account")
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("stalwart")
            .new_field("azure-access-key")
            .label("Access Key")
            .help("The access key for the Azure Storage Account")
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .new_field("sas-token")
            .label("SAS Token")
            .help("SAS Token, when not using access-key based authentication")
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .build()
            // FS specific
            .new_field("depth")
            .label("Nested Depth")
            .help("Maximum depth of nested directories")
            .display_if_eq("type", ["fs"])
            .default("2")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(0.into()), Validator::MaxValue(5.into())],
            )
            .build()
            // SQL read replicas
            .new_field("primary")
            .label("Primary SQL")
            .help("Primary SQL store where the data is written")
            .display_if_eq("type", ["sql-read-replica"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&["mysql", "postgresql"])
            .input_check([], [Validator::Required])
            .build()
            .new_field("replicas")
            .label("Read replicas")
            .help("The read replicas where the data is read from")
            .display_if_eq("type", ["sql-read-replica"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::ManyWithSearch,
            })
            .source_filter(&["mysql", "postgresql"])
            .input_check([], [Validator::Required])
            .build()
            // Sharded blobs
            .new_field("stores")
            .label("Blob stores")
            .help("Blob stores to use for the sharded blob store")
            .display_if_eq("type", ["sharded-blob"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::ManyWithSearch,
            })
            .source_filter(&["s3", "fs"])
            .input_check([], [Validator::Required])
            .build()
            // Sharded In-memory
            .new_field("stores")
            .label("In-memory stores")
            .help("In-memory stores to use for the sharded in-memory store")
            .display_if_eq("type", ["sharded-in-memory"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::ManyWithSearch,
            })
            .source_filter(&["redis"])
            .input_check([], [Validator::Required])
            .build()
            // Form layouts
            .new_form_section()
            .title("Configuration")
            .fields([
                "_id",
                "type",
                "path",
                "cluster-file",
                "redis-type",
                "address",
                "host",
                "port",
                "database",
                "url",
                "urls",
                "protocol-version",
                "max-allowed-packet",
                "region",
                "endpoint",
                "cloud-id",
                "profile",
                "timeout",
                "primary",
                "replicas",
                "stores",
                "timeout.connection",
                "timeout.request",
                "max-reconnects",
                "ping-interval",
            ])
            .build()
            .new_form_section()
            .title("Bucket")
            .display_if_eq("type", ["s3"])
            .fields(["bucket", "key-prefix"])
            .build()
            .new_form_section()
            .title("PubSub")
            .display_if_eq("type", ["nats"])
            .fields([
                "capacity.client",
                "capacity.subscription",
                "capacity.read-buffer",
                "no-echo",
            ])
            .build()
            .new_form_section()
            .title("Storage Account")
            .display_if_eq("type", ["azure"])
            .fields(["storage-account", "container", "key-prefix"])
            .build()
            .new_form_section()
            .title("Authentication")
            .display_if_eq(
                "type",
                [
                    "postgresql",
                    "mysql",
                    "elasticsearch",
                    "s3",
                    "azure",
                    "nats",
                ],
            )
            .display_if_eq("redis-type", ["cluster"])
            .fields([
                "user",
                "password",
                "access-key",
                "secret-key",
                "security-token",
                "azure-access-key",
                "sas-token",
                "credentials",
            ])
            .build()
            .new_form_section()
            .title("Storage settings")
            .display_if_eq(
                "type",
                [
                    "postgresql",
                    "mysql",
                    "sqlite",
                    "rocksdb",
                    "foundationdb",
                    "fs",
                    "s3",
                    "azure",
                    "sql-read-replica",
                    "sharded-blob",
                ],
            )
            .fields([
                "compression",
                "settings.min-blob-size",
                "settings.write-buffer-size",
                "max-retries",
                "depth",
                "purge.frequency",
            ])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch", "nats"])
            .fields(["tls.enable", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Pools")
            .display_if_eq("type", ["rocksdb", "sqlite", "postgresql", "mysql"])
            .fields([
                "pool.workers",
                "pool.max-connections",
                "pool.min-connections",
            ])
            .build()
            .new_form_section()
            .title("Cluster Settings")
            .display_if_eq("redis-type", ["cluster"])
            .fields([
                "read-from-replicas",
                "retry.total",
                "retry.max-wait",
                "retry.min-wait",
            ])
            .build()
            .new_form_section()
            .title("Cluster Ids")
            .display_if_eq("type", ["foundationdb"])
            .fields(["ids.machine", "ids.data-center"])
            .build()
            .new_form_section()
            .title("Transaction Settings")
            .display_if_eq("type", ["foundationdb"])
            .fields([
                "transaction.timeout",
                "transaction.max-retry-delay",
                "transaction.retry-limit",
            ])
            .build()
            .new_form_section()
            .title("Directory Queries")
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .fields([
                "query.name",
                "query.members",
                "query.recipients",
                "query.emails",
                "query.secrets",
            ])
            .build()
            .new_form_section()
            .title("Index")
            .display_if_eq("type", ["elasticsearch"])
            .fields(["index.shards", "index.replicas"])
            .build()
            .list_title("Stores")
            .list_subtitle("Manage data, blob, full-text, and lookup stores")
            .list_fields(["_id", "type"])
            .build()
            // HTTP lookups
            .new_schema("http-lookup")
            .names("list", "lists")
            .prefix("http-lookup")
            .suffix("url")
            .new_id_field()
            .label("List ID")
            .help("Unique identifier for the HTTP list")
            .build()
            .new_field("enable")
            .label("Enable list")
            .help("Whether to enable this HTTP list")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("url")
            .label("URL")
            .help("URL of the list")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("format")
            .label("Format")
            .help("Format of the list")
            .default("csv")
            .typ(Type::Select {
                source: Source::Static(&[("list", "List"), ("csv", "CSV")]),
                typ: SelectType::Single,
            })
            .build()
            .new_field("separator")
            .label("Separator")
            .help(concat!(
                "The separator character used to parse the HTTP list.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .default(",")
            .display_if_eq("format", ["csv"])
            .build()
            .new_field("index.key")
            .label("Key Index")
            .help(concat!("The position of the key field in the HTTP List.",))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .default("0")
            .display_if_eq("format", ["csv"])
            .build()
            .new_field("index.value")
            .label("Value Index")
            .help(concat!("The position of the value field in the HTTP List.",))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("format", ["csv"])
            .build()
            .new_field("skip-first")
            .label("Skip header")
            .help("Whether to skip the first line of the list")
            .default("false")
            .typ(Type::Boolean)
            .display_if_eq("format", ["csv"])
            .build()
            .new_field("retry")
            .label("Retry")
            .help(concat!(
                "How long to wait before retrying to download the list ",
                "in case of failure."
            ))
            .default("1h")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .new_field("refresh")
            .label("Refresh")
            .help("How often to refresh the list")
            .default("12h")
            .new_field("timeout")
            .label("Timeout")
            .help("How long to wait for the list to download before timing out")
            .default("30s")
            .build()
            .new_field("gzipped")
            .label("Gzipped")
            .help("Whether to use gzip compression when downloading the list")
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("limits.size")
            .label("Size")
            .help(concat!(
                "Maximum size of the list. ",
                "The list is truncated if it exceeds this size."
            ))
            .default("104857600")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(10.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
                    Validator::Required,
                ],
            )
            .build()
            .new_field("limits.entries")
            .label("Max entries")
            .help(concat!(
                "Maximum number of entries allowed in the list. ",
                "The list is truncated if it exceeds this limit."
            ))
            .default("100000")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                    Validator::Required,
                ],
            )
            .build()
            .new_field("limits.entry-size")
            .label("Entry length")
            .help(concat!("Maximum length of an entry in the list. "))
            .default("512")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                    Validator::Required,
                ],
            )
            .build()
            .new_form_section()
            .title("HTTP List Settings")
            .fields(["_id", "url", "format", "gzipped", "enable"])
            .build()
            .new_form_section()
            .title("CSV Parsing")
            .fields(["separator", "index.key", "index.value", "skip-first"])
            .display_if_eq("format", ["csv"])
            .build()
            .new_form_section()
            .title("Configuration")
            .fields(["retry", "refresh", "timeout"])
            .build()
            .new_form_section()
            .title("Limits")
            .fields(["limits.size", "limits.entries", "limits.entry-size"])
            .build()
            .list_title("HTTP Lists")
            .list_subtitle("Manage HTTP list lookups")
            .list_fields(["_id", "url", "enable"])
            .build()
    }
}
