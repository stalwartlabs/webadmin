use crate::core::schema::*;

use super::smtp::{FUNCTIONS_MAP, IN_CONNECT_VARIABLES};

impl Builder<Schemas, ()> {
    pub fn build_server(self) -> Self {
        let connect_expr = ExpressionValidator::default()
            .variables(IN_CONNECT_VARIABLES)
            .functions(FUNCTIONS_MAP);

        self.new_schema("network")
            // Max connections
            .new_field("server.max-connections")
            .label("Max connections")
            .help("The maximum number of concurrent connections the server will accept")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .default("8192")
            .build()
            // HTTP base URL
            .new_field("server.http.url")
            .label("Base URL")
            .help("The base URL for the HTTP server")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr),
                ],
            )
            .default("protocol + '://' + key_get('default', 'hostname') + ':' + local_port")
            .build()
            // Use X-Forwarded-For
            .new_field("server.http.use-x-forwarded")
            .label("Use X-Forwarded")
            .help(concat!(
                "Specifies whether to use the X-Forwarded-For header to ",
                "determine the client's IP address"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            // HTTP headers
            .new_field("server.http.headers")
            .label("Add headers")
            .help("Additional headers to include in HTTP responses")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            // Cluster node ID
            .new_field("cluster.node-id")
            .label("Node ID")
            .help(concat!("Unique identifier for this node in the cluster"))
            .default("1")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(0.into())],
            )
            .build()
            // Network fields
            .add_network_fields(false)
            // Forms
            .new_form_section()
            .title("HTTP Settings")
            .fields([
                "server.http.url",
                "server.http.headers",
                "server.http.use-x-forwarded",
            ])
            .build()
            .new_form_section()
            .title("Network settings")
            .fields(["server.max-connections", "server.proxy.trusted-networks"])
            .build()
            .new_form_section()
            .title("Cluster")
            .fields(["cluster.node-id"])
            .build()
            .new_form_section()
            .title("Socket options")
            .fields([
                "server.socket.backlog",
                "server.socket.ttl",
                "server.socket.linger",
                "server.socket.tos",
                "server.socket.send-buffer-size",
                "server.socket.recv-buffer-size",
                "server.socket.nodelay",
                "server.socket.reuse-addr",
                "server.socket.reuse-port",
            ])
            .build()
            .build()
            // Common settings
            .new_schema("system")
            // Default hostname
            .new_field("lookup.default.hostname")
            .label("Hostname")
            .help("The default system hostname")
            .placeholder("mail.example.com")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            // Default hostname
            .new_field("lookup.default.domain")
            .label("Domain")
            .help("The default domain name")
            .placeholder("example.com")
            .build()
            // Local keys
            .new_field("config.local-keys")
            .label("Local settings")
            .help(concat!(
                "List of glob expressions for local configuration keys",
                " that should be stored locally in the configuration file.",
                "All other keys will be stored in the database."
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::Required])
            .default(
                &[
                    "store.*",
                    "!store.*.query.*",
                    "server.listener.*",
                    "server.socket.*",
                    "server.tls.*",
                    "cluster.node-id",
                    "storage.data",
                    "storage.blob",
                    "storage.lookup",
                    "storage.fts",
                    "server.run-as.user",
                    "server.run-as.group",
                    "config.local-keys.*",
                ][..],
            )
            .build()
            // Run as user
            .new_field("server.run-as.user")
            .label("User")
            .help("The system user the server should run as")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsId])
            .build()
            // Run as group
            .new_field("server.run-as.group")
            .label("Group")
            .help("The system group the server should run as")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsId])
            .build()
            // Thread pool
            .new_field("global.thread-pool")
            .label("Pool size")
            .help(concat!(
                "The number of threads in the global thread pool for ",
                "CPU intensive tasks. Defaults to the number ",
                "of CPU cores"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .placeholder("8")
            .build()
            .new_form_section()
            .title("Defaults")
            .fields(["lookup.default.hostname", "lookup.default.domain"])
            .build()
            .new_form_section()
            .title("Run as")
            .fields(["server.run-as.user", "server.run-as.group"])
            .build()
            .new_form_section()
            .title("Thread pool")
            .fields(["global.thread-pool"])
            .build()
            .new_form_section()
            .title("Local configuration keys")
            .fields(["config.local-keys"])
            .build()
            .build()
            // Caching
            .new_schema("cache")
            .new_field("cache.capacity")
            .label("Initial capacity")
            .help("The initial capacity of the cache")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .default("512")
            .build()
            .new_field("cache.shard")
            .label("Shard size")
            .help("The number of shards in the cache")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(2.into())],
            )
            .default("32")
            .build()
            .new_field("cache.account.size")
            .label("Account")
            .help("The size of the account cache")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .default("2048")
            .build()
            .new_field("cache.mailbox.size")
            .label("Mailbox")
            .help("The size of the mailbox cache")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .default("2048")
            .build()
            .new_field("cache.thread.size")
            .label("Thread Ids")
            .help("The size of the thread id cache")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue(1.into())],
            )
            .default("2048")
            .build()
            .new_form_section()
            .title("Cache settings")
            .fields(["cache.capacity", "cache.shard"])
            .build()
            .new_form_section()
            .title("Message cache size")
            .fields([
                "cache.account.size",
                "cache.mailbox.size",
                "cache.thread.size",
            ])
            .build()
            .build()
            // Blocked IP addresses
            .new_schema("blocked-ip")
            .names("address", "addresses")
            .prefix("server.blocked-ip")
            .new_id_field()
            .label("IP Address(es)")
            .help("The IP address or mask to block")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsIpOrMask],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Blocked IP addresses")
            .list_subtitle("Manage blocked IP addresses")
            .list_fields(["_id"])
            .list_actions([Action::Create, Action::Delete, Action::Search])
            .build()
    }
}

impl Builder<Schemas, Schema> {
    pub fn add_network_fields(self, is_listener: bool) -> Self {
        let do_override: &'static [&'static str] =
            if is_listener { &["true"][..] } else { &[][..] };

        // Proxy networks
        self.new_field(if is_listener {
            "proxy.trusted-networks"
        } else {
            "server.proxy.trusted-networks"
        })
        .label("Proxy networks")
        .help("Enable proxy protocol for connections from these networks")
        .typ(Type::Array)
        .input_check([Transformer::Trim], [Validator::IsIpOrMask])
        .display_if_eq("proxy.override", do_override.iter().copied())
        .build()
        // Socket options
        // Backlog
        .new_field(if is_listener {
            "socket.backlog"
        } else {
            "server.socket.backlog"
        })
        .label("Backlog")
        .help(concat!(
            "The maximum number of incoming connections ",
            "that can be pending in the backlog queue"
        ))
        .default("1024")
        .typ(Type::Input)
        .input_check([Transformer::Trim], vec![Validator::MinValue(1.into())])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // TTL
        .new_field(if is_listener {
            "socket.ttl"
        } else {
            "server.socket.ttl"
        })
        .label("TTL")
        .help(concat!(
            "Time-to-live (TTL) value for the socket, which determines how ",
            "many hops a packet can make before it is discarded"
        ))
        .typ(Type::Duration)
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // Linger
        .new_field(if is_listener {
            "socket.linger"
        } else {
            "server.socket.linger"
        })
        .label("Linger")
        .help(concat!(
            "The time to wait before closing a socket when ",
            "there is still unsent data"
        ))
        .typ(Type::Duration)
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // ToS
        .new_field(if is_listener {
            "socket.tos"
        } else {
            "server.socket.tos"
        })
        .label("Type of Service")
        .help(concat!(
            "The type of service (TOS) value for the socket, ",
            "which determines the priority of the traffic sent through the socket"
        ))
        .typ(Type::Input)
        .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // Send buf size
        .new_field(if is_listener {
            "socket.send-buffer-size"
        } else {
            "server.socket.send-buffer-size"
        })
        .label("Send buffer")
        .help("The size of the buffer used for sending data")
        .typ(Type::Input)
        .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // Receive buf size
        .new_field(if is_listener {
            "socket.recv-buffer-size"
        } else {
            "server.socket.recv-buffer-size"
        })
        .label("Receive buffer")
        .help("The size of the buffer used for receiving data")
        .default("")
        .typ(Type::Input)
        .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // No delay
        .new_field(if is_listener {
            "socket.nodelay"
        } else {
            "server.socket.nodelay"
        })
        .label("No delay")
        .help("Whether the Nagle algorithm should be disabled for the socket")
        .default("true")
        .typ(Type::Boolean)
        .input_check([], [Validator::Required])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // Reuse addr
        .new_field(if is_listener {
            "socket.reuse-addr"
        } else {
            "server.socket.reuse-addr"
        })
        .label("Reuse Address")
        .help(concat!(
            "Whether the socket can be bound to an address that ",
            "is already in use by another socket"
        ))
        .default("true")
        .typ(Type::Boolean)
        .input_check([], [Validator::Required])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
        // Reuse port
        .new_field(if is_listener {
            "socket.reuse-port"
        } else {
            "server.socket.reuse-port"
        })
        .label("Reuse port")
        .help("Whether multiple sockets can be bound to the same address and port")
        .default("true")
        .typ(Type::Boolean)
        .input_check([], [Validator::Required])
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
    }
}
