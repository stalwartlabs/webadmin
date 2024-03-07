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
        self.with_source_static(
            "stores",
            [
                ("rocksdb", "RocksDB"),
                ("foundationdb", "FoundationDB"),
                ("postgresql", "PostgreSQL"),
                ("mysql", "mySQL"),
                ("sqlite", "SQLite"),
                ("s3", "S3-compatible"),
                ("redis", "Redis/Memcached"),
                ("elasticsearch", "ElasticSearch"),
                ("fs", "Filesystem"),
                ("memory", "Memory"),
            ],
        )
        .with_source_static(
            "redis-type",
            [
                ("single", "Redis single node"),
                ("cluster", "Redis Cluster"),
            ],
        )
        .with_source_static("compression", [("none", "None"), ("lz4", "LZ4")])
        .new_schema("store")
        .with_names("store", "stores")
        .with_prefix("store")
        .with_suffix("type")
        // Type
        .new_field("type")
        .readonly()
        .with_label("Type")
        .with_help("Storage backend type")
        .with_default("rocksdb")
        .with_type_select("stores")
        .build()
        // Compression
        .new_field("compression")
        .readonly()
        .with_label("Compression")
        .with_help("Compression algorithm for blobs")
        .with_default("lz4")
        .with_type_select("compression")
        .with_display_if_none("type", ["redis", "memory", "elasticsearch"])
        .build()
        // Path
        .new_field("path")
        .with_label("Path")
        .with_help("Where to store the data in the server's filesystem")
        .with_display_if_any("type", ["rocksdb", "sqlite", "fs"])
        .with_type_input([Transformer::Trim], [Validator::Required])
        .build()
        // Host
        .new_field("host")
        .with_label("Hostname")
        .with_help("Hostname of the database server")
        .with_display_if_any("type", ["postgresql", "mysql"])
        .with_type_input(
            [Transformer::Trim],
            [Validator::Required, Validator::IsHost],
        )
        .build()
        // Port
        .new_field("port")
        .with_label("Port")
        .with_help("Port of the database server")
        .with_display_if_any("type", ["postgresql", "mysql"])
        .with_default_if_any("type", ["postgresql"], "5432")
        .with_default_if_any("type", ["mysql"], "3307")
        .with_type_input(
            [Transformer::Trim],
            [Validator::Required, Validator::IsPort],
        )
        .build()
        // Database name
        .new_field("database")
        .with_label("Database")
        .with_help("Name of the database")
        .with_default("stalwart")
        .with_display_if_any("type", ["postgresql", "mysql"])
        .with_type_input([Transformer::Trim], [Validator::Required])
        .build()
        // Username
        .new_field("user")
        .with_label("Username")
        .with_help("Username to connect to the database")
        .with_default("stalwart")
        .with_display_if_any("type", ["postgresql", "mysql", "elasticsearch"])
        .with_display_if_any("redis-type", ["cluster"])
        .with_type_input([Transformer::Trim], [])
        .build()
        // Password
        .new_field("password")
        .with_label("Password")
        .with_help("Password to connect to the database")
        .with_display_if_any("type", ["postgresql", "mysql", "elasticsearch"])
        .with_display_if_any("redis-type", ["cluster"])
        .with_type_secret([], [])
        .build()
        // Timeout
        .new_field("timeout")
        .with_label("Timeout")
        .with_help("Connection timeout to the database")
        .with_display_if_any("type", ["postgresql", "mysql", "redis"])
        .with_type_duration()
        .with_default("15s")
        .build()
        // Purge frequency
        .new_field("purge.frequency")
        .with_label("Purge Frequency")
        .with_help("How often to purge the database. Expects a cron expression")
        .with_display_if_none("type", ["redis", "memory", "elasticsearch"])
        .with_type_input(
            [Transformer::Trim],
            [Validator::Required, Validator::IsCron],
        )
        .build()
        // Workers
        .new_field("pool.workers")
        .with_label("Thread Pool Size")
        .with_help("Number of worker threads to use for the store, defaults to the number of cores")
        .with_display_if_any("type", ["rocksdb", "sqlite"])
        .with_type_input(
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
        .with_label("Max Connections")
        .with_help("Maximum number of connections to the store")
        .with_display_if_any("type", ["postgresql", "mysql", "sqlite"])
        .with_default("10")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1), Validator::MaxValue(8192)],
        )
        .build()
        .new_field("pool.min-connections")
        .with_label("Min Connections")
        .with_help("Minimum number of connections to the store")
        .with_display_if_any("type", ["mysql"])
        .with_default("5")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1), Validator::MaxValue(8192)],
        )
        .build()
        // TLS
        .new_field("tls.enable")
        .with_label("Enable TLS")
        .with_help("Use TLS to connect to the store")
        .with_display_if_any("type", ["postgresql"])
        .with_default("false")
        .with_type_checkbox()
        .build()
        .new_field("tls.allow-invalid-certs")
        .with_label("Allow Invalid Certs")
        .with_help("Allow invalid TLS certificates when connecting to the store")
        .with_display_if_any("type", ["postgresql", "elasticsearch"])
        .with_default("false")
        .with_type_checkbox()
        .build()
        // URL
        .new_field("url")
        .with_label("URL")
        .with_help("URL of the store")
        .with_display_if_any("type", ["elasticsearch"])
        .with_default_if_any("type", ["elasticsearch"], "https://localhost:9200")
        .with_type_multi_input([Transformer::Trim], [Validator::Required, Validator::IsUrl])
        .build()
        // SQL directory specific
        .new_field("query.name")
        .with_label("Account by Name")
        .with_help("Query to obtain the account details by login name")
        .with_type_input([Transformer::Trim], [])
        .with_display_if_any("type", ["postgresql", "mysql", "sqlite"])
        .with_default_if_any("type", ["postgresql"], PGSQL_NAME)
        .with_default_if_any("type", ["mysql"], MYSQL_NAME)
        .with_default_if_any("type", ["sqlite"], SQLITE_NAME)
        .new_field("query.members")
        .with_label("Members")
        .with_help("Query to obtain the members of a group by account name")
        .with_default_if_any("type", ["postgresql"], PGSQL_MEMBERS)
        .with_default_if_any("type", ["mysql"], MYSQL_MEMBERS)
        .with_default_if_any("type", ["sqlite"], SQLITE_MEMBERS)
        .new_field("query.recipients")
        .with_label("Recipients")
        .with_help("Query to obtain the recipient(s) of an e-mail address")
        .with_default_if_any("type", ["postgresql"], PGSQL_RECIPIENTS)
        .with_default_if_any("type", ["mysql"], MYSQL_RECIPIENTS)
        .with_default_if_any("type", ["sqlite"], SQLITE_RECIPIENTS)
        .new_field("query.emails")
        .with_label("E-mails")
        .with_help("Query to obtain the e-mail address(es) of an account")
        .with_default_if_any("type", ["postgresql"], PGSQL_EMAILS)
        .with_default_if_any("type", ["mysql"], MYSQL_EMAILS)
        .with_default_if_any("type", ["sqlite"], SQLITE_EMAILS)
        .new_field("query.verify")
        .with_label("Verify (VRFY)")
        .with_help("Query to verify an e-mail address with the VRFY SMTP command")
        .with_default_if_any("type", ["postgresql"], PGSQL_VRFY)
        .with_default_if_any("type", ["mysql"], MYSQL_VRFY)
        .with_default_if_any("type", ["sqlite"], SQLITE_VRFY)
        .new_field("query.expand")
        .with_label("Expand (EXPN)")
        .with_help("Query to expand an e-mail address with the EXPN SMTP command")
        .with_default_if_any("type", ["postgresql"], PGSQL_EXPN)
        .with_default_if_any("type", ["mysql"], MYSQL_EXPN)
        .with_default_if_any("type", ["sqlite"], SQLITE_EXPN)
        .new_field("query.domains")
        .with_label("Local domains")
        .with_help("Query to verify whether a domain is local")
        .with_default_if_any("type", ["postgresql"], PGSQL_DOMAINS)
        .with_default_if_any("type", ["mysql"], MYSQL_DOMAINS)
        .with_default_if_any("type", ["sqlite"], SQLITE_DOMAINS)
        .build()
        // RocksDB specific
        .new_field("settings.min-blob-size")
        .with_label("Min blob size")
        .with_help(concat!(
            "Minimum size of a blob to store in the blob store, ",
            "smaller blobs are stored in the metadata store"
        ))
        .with_display_if_any("type", ["rocksdb"])
        .with_default("16834")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1024), Validator::MaxValue(1024 * 1024)],
        )
        .new_field("settings.write-buffer-size")
        .with_label("Write buffer size")
        .with_help("Size of the write buffer in bytes, used to batch writes to the store")
        .with_default("134217728")
        .with_type_input(
            [Transformer::Trim],
            [
                Validator::MinValue(8192),
                Validator::MaxValue(1024 * 1024 * 1024),
            ],
        )
        .build()
        // FoundationDB specific
        .new_field("cluster-file")
        .with_label("Cluster file")
        .with_help("Path to the cluster file for the FoundationDB cluster")
        .with_display_if_any("type", ["foundationdb"])
        .with_placeholder("/etc/foundationdb/fdb.cluster")
        .with_type_input([Transformer::Trim], [])
        .new_field("transaction.timeout")
        .with_label("Timeout")
        .with_help("Transaction timeout")
        .with_placeholder("5s")
        .with_type_duration()
        .new_field("transaction.max-retry-delay")
        .with_label("Max Retry Delay")
        .with_help("Transaction maximum retry delay")
        .with_placeholder("1s")
        .with_type_duration()
        .new_field("transaction.retry-limit")
        .with_label("Retry limit")
        .with_help("Transaction retry limit")
        .with_placeholder("10")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1), Validator::MaxValue(1000)],
        )
        .new_field("ids.machine")
        .with_label("Machine Id")
        .with_help("Machine ID in the FoundationDB cluster (optional)")
        .with_placeholder("my-server-id")
        .with_type_input([Transformer::Trim], [Validator::IsId])
        .new_field("ids.data-center")
        .with_label("Data Center Id")
        .with_help("Data center ID (optional)")
        .with_placeholder("my-datacenter-id")
        .build()
        // mySQL specific
        .new_field("max-allowed-packet")
        .with_label("Max Allowed Packet")
        .with_help("Maximum size of a packet in bytes")
        .with_display_if_any("type", ["mysql"])
        .with_placeholder("1073741824")
        .with_type_input(
            [Transformer::Trim],
            [
                Validator::MinValue(1024),
                Validator::MaxValue(1024 * 1024 * 1024),
            ],
        )
        .build()
        // ElasticSearch specific
        .new_field("cloud-id")
        .with_label("Cloud Id")
        .with_help("Cloud ID for the ElasticSearch cluster")
        .with_display_if_any("type", ["elasticsearch"])
        .with_placeholder("my-cloud-id")
        .with_type_input([Transformer::Trim], [])
        .new_field("index.shards")
        .with_label("Number of Shards")
        .with_help("Number of shards for the index")
        .with_placeholder("3")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1), Validator::MaxValue(1024 * 1024)],
        )
        .new_field("index.replicas")
        .with_label("Number of Replicas")
        .with_help("Number of replicas for the index")
        .with_placeholder("0")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(0), Validator::MaxValue(2048)],
        )
        .build()
        // Redis specific
        .new_field("urls")
        .with_label("URL(s)")
        .with_help("URL(s) of the Redis server(s)")
        .with_display_if_any("type", ["redis"])
        .with_default("redis://127.0.0.1")
        .with_type_multi_input([Transformer::Trim], [Validator::Required, Validator::IsUrl])
        .build()
        .new_field("redis-type")
        .with_label("Server Type")
        .with_help("Type of Redis server")
        .with_display_if_any("type", ["redis"])
        .with_default("single")
        .with_type_select("redis-type")
        .build()
        .new_field("retry.total")
        .with_label("Retries")
        .with_help("Number of retries to connect to the Redis cluster")
        .with_display_if_any("redis-type", ["cluster"])
        .with_placeholder("3")
        .with_type_input(
            [Transformer::Trim],
            [Validator::MinValue(1), Validator::MaxValue(1024)],
        )
        .new_field("retry.max-wait")
        .with_label("Max Wait")
        .with_help("Maximum time to wait between retries")
        .with_placeholder("1s")
        .with_type_duration()
        .new_field("retry.min-wait")
        .with_label("Min Wait")
        .with_help("Minimum time to wait between retries")
        .with_placeholder("500ms")
        .new_field("read-from-replicas")
        .with_label("Read from replicas")
        .with_help("Whether to read from replicas")
        .with_default("true")
        .with_type_checkbox()
        .build()
        // FS specific
        .new_field("depth")
        .with_label("Nested Depth")
        .with_help("Maximum depth of nested directories")
        .with_display_if_any("type", ["fs"])
        .with_default("2")
        .with_type_multi_input(
            [Transformer::Trim],
            [Validator::MinValue(0), Validator::MaxValue(5)],
        )
        .build()
        .new_form_section()
        .with_title("Configuration")
        .with_fields([
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
        .with_title("Authentication")
        .with_display_if_any("type", ["postgresql", "mysql", "elasticsearch"])
        .with_display_if_any("redis-type", ["cluster"])
        .with_fields(["user", "password"])
        .build()
        .new_form_section()
        .with_title("Storage settings")
        .with_display_if_any(
            "type",
            ["postgresql", "mysql", "sqlite", "rocksdb", "foundationdb"],
        )
        .with_fields([
            "compression",
            "settings.min-blob-size",
            "settings.write-buffer-size",
            "depth",
            "purge",
        ])
        .build()
        .new_form_section()
        .with_title("TLS")
        .with_display_if_any("type", ["postgresql", "elasticsearch"])
        .with_fields(["tls.enable", "tls.allow-invalid-certs"])
        .build()
        .new_form_section()
        .with_title("Pools")
        .with_display_if_any("type", ["rocksdb", "sqlite", "postgresql", "mysql"])
        .with_fields([
            "pool.workers",
            "pool.max-connections",
            "pool.min-connections",
        ])
        .build()
        .new_form_section()
        .with_title("Cluster Settings")
        .with_display_if_any("redis-type", ["cluster"])
        .with_fields([
            "read-from-replicas",
            "retry.total",
            "retry.max-wait",
            "retry.min-wait",
        ])
        .build()
        .new_form_section()
        .with_title("Cluster Ids")
        .with_display_if_any("type", ["foundationdb"])
        .with_fields(["ids.machine", "ids.data-center"])
        .build()
        .new_form_section()
        .with_title("Transaction Settings")
        .with_display_if_any("type", ["foundationdb"])
        .with_fields([
            "transaction.timeout",
            "transaction.max-retry-delay",
            "transaction.retry-limit",
        ])
        .build()
        .new_form_section()
        .with_title("Directory Queries")
        .with_display_if_any("type", ["postgresql", "mysql", "sqlite"])
        .with_fields([
            "query.name",
            "query.members",
            "query.recipients",
            "query.emails",
            "query.verify",
            "query.expand",
            "query.domains",
        ])
        .build()
        .with_list_title("Stores")
        .with_list_subtitle("Manage the data, blob, full-text, and lookup stores")
        .with_list_fields(["_id", "type"])
        .build()
    }
}
