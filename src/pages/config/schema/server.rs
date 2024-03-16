use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_server(self) -> Self {
        self.new_schema("network")
            // Hostname
            .new_field("server.hostname")
            .label("Hostname")
            .help("The hostname of the server")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("mail.example.com")
            .build()
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
            // Fail2ban
            .new_field("server.security.fail2ban")
            .label("Fail2ban rate")
            .help("The maximum number of failed login attempts before the IP is banned")
            .typ(Type::Rate)
            .default("100/1d")
            .input_check([], [Validator::Required])
            .build()
            // Network fields
            .add_network_fields(false)
            // Forms
            .new_form_section()
            .title("Network settings")
            .fields(["server.hostname", "server.max-connections"])
            .build()
            .new_form_section()
            .title("Security")
            .fields(["server.security.fail2ban"])
            .build()
            .new_form_section()
            .title("Proxy protocol")
            .fields(["server.proxy.trusted-networks"])
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
            .title("Run as")
            .fields(["server.run-as.user", "server.run-as.group"])
            .build()
            .new_form_section()
            .title("Thread pool")
            .fields(["global.thread-pool"])
            .build()
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
        .input_check(
            [Transformer::Trim],
            if is_listener {
                vec![Validator::MinValue(1.into())]
            } else {
                vec![Validator::Required, Validator::MinValue(1.into())]
            },
        )
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
        .display_if_eq("socket.override", do_override.iter().copied())
        .build()
    }
}
