use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_listener(self) -> Self {
        self.new_schema("listener")
            .names("listener", "listeners")
            .prefix("server.listener")
            .suffix("protocol")
            // Id
            .new_id_field()
            .label("Listener Id")
            .help("Unique identifier for the listener")
            .build()
            // Type
            .new_field("protocol")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("smtp", "SMTP"),
                    ("lmtp", "LMTP"),
                    ("http", "HTTP"),
                    ("imap", "IMAP4"),
                    ("managesieve", "ManageSieve"),
                ]),
            })
            .label("Protocol")
            .help("The protocol used by the listener")
            .input_check([], [Validator::Required])
            .default("smtp")
            .build()
            // Bind addresses
            .new_field("bind")
            .label("Bind addresses")
            .help("The addresses the listener will bind to")
            .typ(Type::Array)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsSocketAddr],
            )
            .build()
            // Override proxy protocol
            .new_field("proxy.override")
            .label("Override proxy networks")
            .help("Override the default proxy protocol networks")
            .typ(Type::Boolean)
            .default("false")
            .build()
            // Override socket options
            .new_field("socket.override")
            .label("Override socket options")
            .help("Override the default socket options")
            .typ(Type::Boolean)
            .default("false")
            .build()
            // Override TLS options
            .new_field("tls.override")
            .label("Override TLS options")
            .help("Override the default TLS options")
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("tls.implicit")
            .label("Implicit TLS")
            .help("Whether to use implicit TLS")
            .typ(Type::Boolean)
            .default("false")
            .build()
            // Add common fields
            .add_network_fields(true)
            .add_tls_fields(true)
            // Forms
            .new_form_section()
            .title("Listener settings")
            .fields(["_id", "protocol", "bind"])
            .build()
            .new_form_section()
            .title("TLS options")
            .fields([
                "tls.implicit",
                "tls.override",
                "tls.disable-protocols",
                "tls.disable-ciphers",
                "tls.timeout",
                "tls.ignore-client-order",
            ])
            .build()
            .new_form_section()
            .title("Proxy protocol")
            .fields(["proxy.override", "proxy.trusted-networks"])
            .build()
            .new_form_section()
            .title("Socket options")
            .fields([
                "socket.override",
                "socket.backlog",
                "socket.ttl",
                "socket.linger",
                "socket.tos",
                "socket.send-buffer-size",
                "socket.recv-buffer-size",
                "socket.nodelay",
                "socket.reuse-addr",
                "socket.reuse-port",
            ])
            .build()
            .list_title("Listeners")
            .list_subtitle("Manage SMTP, IMAP, HTTP, and other listeners")
            .list_fields(["_id", "protocol", "bind", "tls.implicit"])
            .build()
    }
}
