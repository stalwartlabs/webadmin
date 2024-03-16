use crate::core::schema::*;

impl Builder<Schemas, ()> {
    pub fn build_tls(self) -> Self {
        self.new_schema("acme")
            .names("ACME provider", "ACME providers")
            .prefix("acme")
            .suffix("directory")
            // Id
            .new_id_field()
            .label("Directory Id")
            .help("Unique identifier for the ACME provider")
            .build()
            // Directory
            .new_field("directory")
            .label("Directory URL")
            .help("The URL of the ACME directory endpoint")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .default("https://acme-v02.api.letsencrypt.org/directory")
            .build()
            // Contact
            .new_field("contact")
            .label("Contact Email")
            .help(concat!(
                "the contact email address, which is used for important ",
                "communications regarding your ACME account and certificates"
            ))
            .typ(Type::Array)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsEmail],
            )
            .build()
            // Port
            .new_field("port")
            .label("TLS ALPN Port")
            .typ(Type::Input)
            .help(concat!(
                "When running Stalwart behind a reverse proxy, specify ",
                "here the port to use for the TLS-ALPN-01 challenge"
            ))
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .default("443")
            .build()
            // Renew before
            .new_field("renew-before")
            .typ(Type::Duration)
            .label("Renew before")
            .help("Determines how early before expiration the certificate should be renewed.")
            .input_check([Transformer::Trim], [Validator::Required])
            .default("30d")
            .build()
            // Lists
            .list_title("ACME providers")
            .list_subtitle("Manage ACME TLS certificate providers")
            .list_fields(["_id", "contact", "renew-before"])
            // Form
            .new_form_section()
            .title("ACME provider")
            .fields(["_id", "directory", "contact", "port", "renew-before"])
            .build()
            .build()
            // ---- TLS certificates ----
            .new_schema("certificate")
            .names("certificate", "certificates")
            .prefix("certificate")
            .suffix("cert")
            // Id
            .new_id_field()
            .label("Certificate Id")
            .help("Unique identifier for the TLS certificate")
            .build()
            // Cert
            .new_field("cert")
            .label("Certificate")
            .typ(Type::Text)
            .help("TLS certificate in PEM format")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // PK
            .new_field("private-key")
            .label("Private Key")
            .typ(Type::Text)
            .help("Private key in PEM format")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("sni-subjects")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::IsDomain])
            .label("Subject Alternative Names")
            .help("Subject Alternative Names (SAN) for the certificate")
            .build()
            .list_title("TLS certificates")
            .list_subtitle("Manage TLS certificates")
            .list_fields(["_id", "sni-subjects"])
            .new_form_section()
            .title("TLS certificate")
            .fields(["_id", "cert", "private-key", "sni-subjects"])
            .build()
            .build()
            // ---- TLS settings ----
            .new_schema("tls")
            // Mechanism
            .new_field("server.tls.enable")
            .label("Mechanism")
            .help("How to obtain the TLS certificates")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("manual", "Manual"),
                    ("acme", "ACME"),
                    ("false", "Disabled"),
                ]),
            })
            .default("manual")
            .input_check([], [Validator::Required])
            .build()
            // Certificate to use
            .new_field("server.tls.certificate")
            .label("Certificate")
            .help("Which certificate to use")
            .typ(Type::Select {
                multi: false,
                source: Source::Dynamic {
                    schema: "certificate",
                    field: "_id",
                    filter: Default::default(),
                },
            })
            .display_if_eq("server.tls.enable", ["manual"])
            .input_check([], [Validator::Required])
            .build()
            // Provider
            .new_field("server.tls.acme")
            .label("ACME provider")
            .help("Which ACME provider to use to obtain the certificate")
            .typ(Type::Select {
                multi: false,
                source: Source::Dynamic {
                    schema: "acme",
                    field: "directory",
                    filter: Default::default(),
                },
            })
            .display_if_eq("server.tls.enable", ["acme"])
            .input_check([], [Validator::Required])
            .build()
            // TLS fields
            .add_tls_fields(false)
            // Forms
            .new_form_section()
            .title("TLS settings")
            .fields([
                "server.tls.enable",
                "server.tls.certificate",
                "server.tls.acme",
                "server.tls.implicit",
                "server.tls.timeout",
            ])
            .build()
            .new_form_section()
            .title("Protocol options")
            .fields([
                "server.tls.disable-protocols",
                "server.tls.disable-ciphers",
                "server.tls.ignore-client-order",
            ])
            .build()
            .build()
    }
}

impl Builder<Schemas, Schema> {
    pub fn add_tls_fields(self, is_listener: bool) -> Self {
        let do_override: &'static [&'static str] =
            if is_listener { &["true"][..] } else { &[][..] };

        // Implicit
        self.new_field(if is_listener {
            "tls.implicit"
        } else {
            "server.tls.implicit"
        })
        .label("Implicit TLS")
        .help("Whether to use implicit TLS")
        .typ(Type::Boolean)
        .default("false")
        .build()
        // Ignore client order
        .new_field(if is_listener {
            "tls.ignore-client-order"
        } else {
            "server.tls.ignore-client-order"
        })
        .label("Ignore client order")
        .help("Whether to ignore the client's cipher order")
        .typ(Type::Boolean)
        .default("true")
        .display_if_eq("tls.override", do_override.iter().copied())
        .build()
        // Timeout
        .new_field(if is_listener {
            "tls.timeout"
        } else {
            "server.tls.timeout"
        })
        .label("Handshake Timeout")
        .help("TLS handshake timeout")
        .typ(Type::Duration)
        .default("1m")
        .display_if_eq("tls.override", do_override.iter().copied())
        .build()
        // Protocols
        .new_field(if is_listener {
            "tls.disable-protocols"
        } else {
            "server.tls.disable-protocols"
        })
        .label("Disabled Protocols")
        .help("Which TLS protocols to disable")
        .typ(Type::Select {
            multi: true,
            source: Source::Static(TLS_PROTOCOLS),
        })
        .display_if_eq("tls.override", do_override.iter().copied())
        .build()
        // Ciphersuites
        .new_field(if is_listener {
            "tls.disable-ciphers"
        } else {
            "server.tls.disable-ciphers"
        })
        .label("Disabled Ciphersuites")
        .help("Which ciphersuites to disable")
        .typ(Type::Select {
            multi: true,
            source: Source::Static(TLS_CIPHERSUITES),
        })
        .display_if_eq("tls.override", do_override.iter().copied())
        .build()
    }
}

pub static TLS_PROTOCOLS: &[(&str, &str)] = &[
    ("TLSv1.2", "TLS version 1.2"),
    ("TLSv1.3", "TLS version 1.3"),
];

pub static TLS_CIPHERSUITES: &[(&str, &str)] = &[
    ("TLS13_AES_256_GCM_SHA384", "TLS1.3 AES256 GCM SHA384"),
    ("TLS13_AES_128_GCM_SHA256", "TLS1.3 AES128 GCM SHA256"),
    (
        "TLS13_CHACHA20_POLY1305_SHA256",
        "TLS1.3 CHACHA20 POLY1305 SHA256",
    ),
    (
        "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
        "ECDHE ECDSA AES256 GCM SHA384",
    ),
    (
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        "ECDHE ECDSA AES128 GCM SHA256",
    ),
    (
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
        "ECDHE ECDSA CHACHA20 POLY1305 SHA256",
    ),
    (
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
        "ECDHE RSA AES256 GCM SHA384",
    ),
    (
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "ECDHE RSA AES128 GCM SHA256",
    ),
    (
        "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        "ECDHE RSA CHACHA20 POLY1305 SHA256",
    ),
];
