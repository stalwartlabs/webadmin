use super::*;

const PGSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = $1 AND active = true";
const PGSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = $1";
const PGSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = $1 ORDER BY name ASC";
const PGSQL_EMAILS: &str =
    "SELECT address FROM emails WHERE name = $1 AND type != 'list' ORDER BY type DESC, address ASC";
const PGSQL_VRFY : &str = "SELECT address FROM emails WHERE address LIKE '%' || $1 || '%' AND type = 'primary' ORDER BY address LIMIT 5";
const PGSQL_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = $1 AND l.type = 'list' ORDER BY p.address LIMIT 50";
const PGSQL_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE '%@' || $1 LIMIT 1";

const MYSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const MYSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const MYSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ? ORDER BY name ASC";
const MYSQL_EMAILS: &str =
    "SELECT address FROM emails WHERE name = ? AND type != 'list' ORDER BY type DESC, address ASC";
const MYSQL_VRFY : &str = "SELECT address FROM emails WHERE address LIKE CONCAT('%', ?, '%') AND type = 'primary' ORDER BY address LIMIT 5";
const MYSQL_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = ? AND l.type = 'list' ORDER BY p.address LIMIT 50";
const MYSQL_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE CONCAT('%@', ?) LIMIT 1";

const SQLITE_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const SQLITE_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const SQLITE_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ?";
const SQLITE_EMAILS: &str =
    "SELECT address FROM emails WHERE name = ? AND type != 'list' ORDER BY type DESC, address ASC";
const SQLITE_VRFY : &str = "SELECT address FROM emails WHERE address LIKE '%' || ? || '%' AND type = 'primary' ORDER BY address LIMIT 5";
const SQLITE_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = ? AND l.type = 'list' ORDER BY p.address LIMIT 50";
const SQLITE_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE '%@' || ? LIMIT 1";

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
            .typ(Type::Select(Source::Static(&[
                ("rocksdb", "RocksDB"),
                ("foundationdb", "FoundationDB"),
                ("postgresql", "PostgreSQL"),
                ("mysql", "mySQL"),
                ("sqlite", "SQLite"),
                ("s3", "S3-compatible"),
                ("redis", "Redis/Memcached"),
                ("elasticsearch", "ElasticSearch"),
                ("fs", "Filesystem"),
            ])))
            .build()
            // Compression
            .new_field("compression")
            .readonly()
            .label("Compression")
            .help("Algorithm to use to compress large binary objects")
            .default("lz4")
            .typ(Type::Select(Source::Static(&[
                ("none", "None"),
                ("lz4", "LZ4"),
            ])))
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
            .typ(Type::Select(Source::Static(&[
                ("single", "Redis single node"),
                ("cluster", "Redis Cluster"),
            ])))
            .build()
            // Username
            .new_field("user")
            .label("Username")
            .help("Username to connect to the database")
            .default("stalwart")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            // Password
            .new_field("password")
            .label("Password")
            .help("Password to connect to the database")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Secret)
            .build()
            // Timeout
            .new_field("timeout")
            .label("Timeout")
            .help("Connection timeout to the database")
            .display_if_eq("type", ["postgresql", "mysql", "redis"])
            .typ(Type::Duration)
            .default("15s")
            .build()
            // Purge frequency
            .new_field("purge.frequency")
            .label("Purge Frequency")
            .help("How often to purge the database. Expects a cron expression")
            .display_if_ne("type", ["redis", "memory", "elasticsearch"])
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsCron],
            )
            .build()
            // Workers
            .new_field("pool.workers")
            .label("Thread Pool Size")
            .help("Number of worker threads to use for the store, defaults to the number of cores")
            .display_if_eq("type", ["rocksdb", "sqlite"])
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue(1),
                    Validator::MaxValue(64),
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
                [Validator::MinValue(1), Validator::MaxValue(8192)],
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
                [Validator::MinValue(1), Validator::MaxValue(8192)],
            )
            .build()
            // TLS
            .new_field("tls.enable")
            .label("Enable TLS")
            .help("Use TLS to connect to the store")
            .display_if_eq("type", ["postgresql"])
            .default("false")
            .typ(Type::Checkbox)
            .build()
            .new_field("tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help("Allow invalid TLS certificates when connecting to the store")
            .display_if_eq("type", ["postgresql", "elasticsearch"])
            .default("false")
            .typ(Type::Checkbox)
            .build()
            // URL
            .new_field("url")
            .label("URL")
            .help("URL of the store")
            .display_if_eq("type", ["elasticsearch"])
            .default("https://localhost:9200")
            .typ(Type::InputMulti)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            // SQL directory specific
            .new_field("query.name")
            .label("Account by Name")
            .help("Query to obtain the account details by login name")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .default_if_eq("type", ["postgresql"], PGSQL_NAME)
            .default_if_eq("type", ["mysql"], MYSQL_NAME)
            .default_if_eq("type", ["sqlite"], SQLITE_NAME)
            .new_field("query.members")
            .label("Members")
            .help("Query to obtain the members of a group by account name")
            .default_if_eq("type", ["postgresql"], PGSQL_MEMBERS)
            .default_if_eq("type", ["mysql"], MYSQL_MEMBERS)
            .default_if_eq("type", ["sqlite"], SQLITE_MEMBERS)
            .new_field("query.recipients")
            .label("Recipients")
            .help("Query to obtain the recipient(s) of an e-mail address")
            .default_if_eq("type", ["postgresql"], PGSQL_RECIPIENTS)
            .default_if_eq("type", ["mysql"], MYSQL_RECIPIENTS)
            .default_if_eq("type", ["sqlite"], SQLITE_RECIPIENTS)
            .new_field("query.emails")
            .label("E-mails")
            .help("Query to obtain the e-mail address(es) of an account")
            .default_if_eq("type", ["postgresql"], PGSQL_EMAILS)
            .default_if_eq("type", ["mysql"], MYSQL_EMAILS)
            .default_if_eq("type", ["sqlite"], SQLITE_EMAILS)
            .new_field("query.verify")
            .label("Verify (VRFY)")
            .help("Query to verify an e-mail address with the VRFY SMTP command")
            .default_if_eq("type", ["postgresql"], PGSQL_VRFY)
            .default_if_eq("type", ["mysql"], MYSQL_VRFY)
            .default_if_eq("type", ["sqlite"], SQLITE_VRFY)
            .new_field("query.expand")
            .label("Expand (EXPN)")
            .help("Query to expand an e-mail address with the EXPN SMTP command")
            .default_if_eq("type", ["postgresql"], PGSQL_EXPN)
            .default_if_eq("type", ["mysql"], MYSQL_EXPN)
            .default_if_eq("type", ["sqlite"], SQLITE_EXPN)
            .new_field("query.domains")
            .label("Local domains")
            .help("Query to verify whether a domain is local")
            .default_if_eq("type", ["postgresql"], PGSQL_DOMAINS)
            .default_if_eq("type", ["mysql"], MYSQL_DOMAINS)
            .default_if_eq("type", ["sqlite"], SQLITE_DOMAINS)
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
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(1024), Validator::MaxValue(1024 * 1024)],
            )
            .new_field("settings.write-buffer-size")
            .label("Write buffer size")
            .help("Size of the write buffer in bytes, used to batch writes to the store")
            .default("134217728")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(8192),
                    Validator::MaxValue(1024 * 1024 * 1024),
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
                [Validator::MinValue(1), Validator::MaxValue(1000)],
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
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1024),
                    Validator::MaxValue(1024 * 1024 * 1024),
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
            .placeholder("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(1), Validator::MaxValue(1024 * 1024)],
            )
            .new_field("index.replicas")
            .label("Number of Replicas")
            .help("Number of replicas for the index")
            .placeholder("0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(0), Validator::MaxValue(2048)],
            )
            .build()
            // Redis specific
            .new_field("urls")
            .label("URL(s)")
            .help("URL(s) of the Redis server(s)")
            .display_if_eq("type", ["redis"])
            .default("redis://127.0.0.1")
            .typ(Type::InputMulti)
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
                [Validator::MinValue(1), Validator::MaxValue(1024)],
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
            .new_field("read-from-replicas")
            .label("Read from replicas")
            .help("Whether to read from replicas")
            .default("true")
            .typ(Type::Checkbox)
            .build()
            // FS specific
            .new_field("depth")
            .label("Nested Depth")
            .help("Maximum depth of nested directories")
            .display_if_eq("type", ["fs"])
            .default("2")
            .typ(Type::InputMulti)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(0), Validator::MaxValue(5)],
            )
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
                "host",
                "port",
                "database",
                "url",
                "urls",
                "max-allowed-packet",
                "cloud-id",
                "timeout",
            ])
            .title("Authentication")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .display_if_eq("redis-type", ["cluster"])
            .fields(["user", "password"])
            .build()
            .new_form_section()
            .title("Storage settings")
            .display_if_eq(
                "type",
                ["postgresql", "mysql", "sqlite", "rocksdb", "foundationdb"],
            )
            .fields([
                "compression",
                "settings.min-blob-size",
                "settings.write-buffer-size",
                "depth",
                "purge.frequency",
            ])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["postgresql", "elasticsearch"])
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
                "query.verify",
                "query.expand",
                "query.domains",
            ])
            .build()
            .list_title("Stores")
            .list_subtitle("Manage data, blob, full-text, and lookup stores")
            .list_fields(["_id", "type"])
            .build()
    }
}
