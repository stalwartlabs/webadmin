/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

use super::{tracing::EVENT_NAMES, HTTP_VARS};

impl Builder<Schemas, ()> {
    pub fn build_server(self) -> Self {
        let http_expr = ExpressionValidator::new(HTTP_VARS, &[]);

        self.new_schema("network")
            // Default hostname
            .new_field("server.hostname")
            .label("Hostname")
            .help("The default fully-qualified system hostname")
            .placeholder("mail.example.com")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
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
            // Network fields
            .add_network_fields(false)
            // Forms
            .new_form_section()
            .title("Network settings")
            .fields([
                "server.hostname",
                "server.max-connections",
                "server.proxy.trusted-networks",
            ])
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
            // HTTP settings
            .new_schema("http")
            // HTTP base URL
            .new_field("server.http.url")
            .label("Base URL")
            .help("The base URL for the HTTP server")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(http_expr)],
            )
            .default("protocol + '://' + config_get('server.hostname') + ':' + local_port")
            .build()
            // HTTP endpoint security
            .new_field("server.http.allowed-endpoint")
            .label("Allowed endpoints")
            .help(concat!(
                "An expression that determines whether access to an endpoint is allowed. ",
                "The expression should an HTTP status code (200, 403, etc.)"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(http_expr)],
            )
            .default("200")
            .build()
            // Use X-Forwarded-For
            .new_field("server.http.use-x-forwarded")
            .label("Obtain remote IP from Forwarded header")
            .help(concat!(
                "Specifies whether to use the Forwarded or X-Forwarded-For header to ",
                "determine the client's IP address"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            // Permissive CORS
            .new_field("server.http.permissive-cors")
            .label("Permissive CORS policy")
            .help(concat!(
                "Specifies whether to allow all origins in the CORS policy ",
                "for the HTTP server"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            // HTTPS Strict Transport Security
            .new_field("server.http.hsts")
            .label("Enable HTTP Strict Transport Security")
            .help(concat!(
                "Specifies whether to enable HTTP Strict Transport Security ",
                "for the HTTP server."
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            // Webadmin auto-update
            .new_field("webadmin.auto-update")
            .label("Auto-update webadmin")
            .help(concat!(
                "Whether to automatically update the webadmin interface ",
                "when a new version is available."
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("webadmin.path")
            .label("Unpack path")
            .help(concat!(
                "The local path to unpack the webadmin bundle to. ",
                "If left empty, the webadmin will be unpacked to /tmp."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("webadmin.resource")
            .label("Update URL")
            .help(concat!(
                "Override the URL to download webadmin updates from. ",
                "By default webadmin updates are downloaded from ",
                "https://github.com/stalwartlabs/webadmin.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            // HTTP headers
            .new_field("server.http.headers")
            .label("Add headers")
            .help("Additional headers to include in HTTP responses")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("HTTP Settings")
            .fields([
                "server.http.url",
                "server.http.headers",
                "server.http.use-x-forwarded",
            ])
            .build()
            .new_form_section()
            .title("HTTP Security")
            .fields([
                "server.http.allowed-endpoint",
                "server.http.hsts",
                "server.http.permissive-cors",
            ])
            .build()
            .new_form_section()
            .title("Web-based Admin")
            .fields(["webadmin.path", "webadmin.resource", "webadmin.auto-update"])
            .build()
            .build()
            // Common settings
            .new_schema("system")
            // Local keys
            .new_field("config.local-keys")
            .label("Local settings")
            .help(concat!(
                "List of glob expressions for local configuration keys",
                " that should be stored locally in the configuration file.",
                "All other keys will be stored in the database. If left blank ",
                "the default settings will be used (check the documentation for more info)"
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::Required])
            .default(
                &[
                    "store.*",
                    "directory.*",
                    "tracer.*",
                    "!server.blocked-ip.*",
                    "!server.allowed-ip.*",
                    "server.*",
                    "config.local-keys.*",
                    "certificate.*",
                    "!cluster.key",
                    "cluster.*",
                    "storage.data",
                    "storage.blob",
                    "storage.lookup",
                    "storage.fts",
                    "storage.directory",
                    "authentication.fallback-admin.*",
                    "enterprise.license-key",
                ][..],
            )
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
            .title("Local configuration keys")
            .fields(["config.local-keys"])
            .build()
            .new_form_section()
            .title("Thread pool")
            .fields(["global.thread-pool"])
            .build()
            .build()
            // Caching
            .new_schema("cache")
            .new_field("cache.dns.txt.size")
            .label("TXT Records")
            .help(concat!("Maximum size of the TXT record cache"))
            .default("5242880")
            .typ(Type::Size)
            .input_check([], [Validator::Required, Validator::MinValue(2048.into())])
            .new_field("cache.dns.mx.size")
            .label("MX Records")
            .help(concat!("Maximum size of the MX record cache"))
            .default("5242880")
            .new_field("cache.dns.ipv4.size")
            .label("IPv4 Records")
            .help(concat!("Maximum size of the IPv4 record cache"))
            .default("5242880")
            .new_field("cache.dns.ipv6.size")
            .label("IPv6 Records")
            .help(concat!("Maximum size of the IPv6 record cache"))
            .default("5242880")
            .new_field("cache.dns.ptr.size")
            .label("PTR Records")
            .help(concat!("Maximum size of the PTR record cache"))
            .default("1048576")
            .new_field("cache.dns.tlsa.size")
            .label("TLSA Records")
            .help(concat!("Maximum size of the TLSA record cache"))
            .default("1048576")
            .new_field("cache.dns.mta-sts.size")
            .label("MTA-STS Records")
            .help(concat!("Maximum size of the MTA-STS record cache"))
            .default("1048576")
            .new_field("cache.dns.rbl.size")
            .label("RBL Records")
            .help(concat!("Maximum size of the DNSBl record cache"))
            .default("5242880")
            .new_field("cache.access-token.size")
            .label("Access Tokens")
            .help(concat!("Maximum size of the access tokens cache"))
            .default("10485760")
            .new_field("cache.http-auth.size")
            .label("HTTP Authorization")
            .help(concat!(
                "Maximum size of the HTTP authorization headers cache"
            ))
            .default("1048576")
            .new_field("cache.permission.size")
            .label("Permissions")
            .help(concat!("Maximum size of the effective permissions cache"))
            .default("5242880")
            .new_field("cache.account.size")
            .label("Account Data")
            .help(concat!("Maximum size of the IMAP account data cache"))
            .default("10485760")
            .new_field("cache.mailbox.size")
            .label("Mailbox Data")
            .help(concat!("Maximum size of the IMAP mailbox data cache"))
            .default("10485760")
            .new_field("cache.thread.size")
            .label("Thread Data")
            .help(concat!("Maximum size of the message thread cache"))
            .default("10485760")
            .new_field("cache.bayes.size")
            .label("Bayes Model")
            .help(concat!("Maximum size of the Bayes model cache"))
            .default("10485760")
            .build()
            .new_form_section()
            .title("Authorization Cache")
            .fields([
                "cache.access-token.size",
                "cache.http-auth.size",
                "cache.permission.size",
            ])
            .build()
            .new_form_section()
            .title("Message Cache")
            .fields([
                "cache.account.size",
                "cache.mailbox.size",
                "cache.thread.size",
            ])
            .build()
            .new_form_section()
            .title("Spam Filter Cache")
            .fields(["cache.bayes.size", "cache.dns.rbl.size"])
            .build()
            .new_form_section()
            .title("DNS Record Cache")
            .fields([
                "cache.dns.txt.size",
                "cache.dns.mx.size",
                "cache.dns.ipv4.size",
                "cache.dns.ipv6.size",
                "cache.dns.ptr.size",
                "cache.dns.tlsa.size",
                "cache.dns.mta-sts.size",
            ])
            .build()
            .build()
            // Blocked IP addresses
            .new_schema("blocked-ip")
            .reload_prefix("server.blocked-ip")
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
            .no_list_action(Action::Modify)
            .build()
            // Allowed IP addresses
            .new_schema("allowed-ip")
            .reload_prefix("server.allowed-ip")
            .names("address", "addresses")
            .prefix("server.allowed-ip")
            .new_id_field()
            .label("IP Address(es)")
            .help("The IP address or mask to allow")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsIpOrMask],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Allowed IP addresses")
            .list_subtitle("Manage allowed IP addresses")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Auto-ban settings
            .new_schema("auto-ban")
            .new_field("server.auto-ban.auth.rate")
            .label("Auth failures")
            .help("The maximum number of failed login attempts before the IP is banned")
            .typ(Type::Rate)
            .default("100/1d")
            .build()
            .new_field("server.auto-ban.scan.rate")
            .label("Scanning attempts")
            .help(concat!(
                "The maximum number of port scanning attempts before the IP is banned"
            ))
            .typ(Type::Rate)
            .default("30/1d")
            .build()
            .new_field("server.auto-ban.abuse.rate")
            .label("Abuse attempts")
            .help(concat!(
                "The maximum number of abuse attempts (relaying or failed ",
                "RCPT TO attempts) before the IP is banned"
            ))
            .typ(Type::Rate)
            .default("35/1d")
            .build()
            .new_field("server.auto-ban.loiter.rate")
            .label("Loitering")
            .help("The maximum number of loitering disconnections before the IP is banned")
            .typ(Type::Rate)
            .default("150/1d")
            .build()
            .new_field("server.auto-ban.scan.paths")
            .label("HTTP banned paths")
            .help(concat!(
                "The paths that will trigger an immediate ban if accessed. ",
                "Each path should be a glob expression"
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .default(
                &[
                    "*.php*",
                    "*.cgi*",
                    "*.asp*",
                    "*/wp-*",
                    "*/php*",
                    "*/cgi-bin*",
                    "*xmlrpc*",
                    "*../*",
                    "*/..*",
                    "*joomla*",
                    "*wordpress*",
                    "*drupal*",
                ][..],
            )
            .build()
            .new_form_section()
            .title("Automatic banning")
            .fields([
                "server.auto-ban.auth.rate",
                "server.auto-ban.abuse.rate",
                "server.auto-ban.loiter.rate",
            ])
            .build()
            .new_form_section()
            .title("Port scanning ban")
            .fields(["server.auto-ban.scan.rate", "server.auto-ban.scan.paths"])
            .build()
            .build()
            // Clustering
            .new_schema("cluster")
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
            // Bind address
            .new_field("cluster.bind-addr")
            .label("Bind Address")
            .help(concat!("The address the gossip protocol will bind to"))
            .placeholder("[::]")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            // Advertise address
            .new_field("cluster.advertise-addr")
            .label("Advertise Address")
            .help(concat!(
                "The address the gossip protocol will advertise",
                " to other nodes in the cluster"
            ))
            .placeholder("10.0.0.1")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsIpOrMask])
            .build()
            // Bind port
            .new_field("cluster.bind-port")
            .label("Port")
            .help(concat!(
                "The UDP port the gossip protocol will bind to. ",
                "Must be the same on all nodes"
            ))
            .default("1179")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsPort])
            .build()
            // Seed nodes
            .new_field("cluster.seed-nodes")
            .label("Seed Nodes")
            .help(concat!("The initial nodes to connect to in the cluster"))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::IsIpOrMask])
            .build()
            // Heartbeat interval
            .new_field("cluster.heartbeat")
            .label("Heartbeat")
            .help(concat!("The interval between heartbeats in the cluster"))
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [])
            .build()
            // Encryption key
            .new_field("cluster.key")
            .label("Encryption Key")
            .help(concat!(
                "The key used to encrypt gossip messages. ",
                "Must be the same on all nodes"
            ))
            .typ(Type::Secret)
            .build()
            // Roles
            .new_field("cluster.roles.purge.stores")
            .label("Purge Stores")
            .help(concat!(
                "List of node ids that are responsible for purging stores"
            ))
            .typ(Type::Array)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(65534.into()),
                ],
            )
            .new_field("cluster.roles.purge.accounts")
            .label("Purge Accounts")
            .help(concat!(
                "List of node ids that are responsible for purging accounts"
            ))
            .new_field("cluster.roles.acme.renew")
            .label("Renew ACME")
            .help(concat!(
                "List of node ids that are responsible for renewing ACME certificates"
            ))
            .new_field("cluster.roles.metrics.calculate")
            .label("Calculate Metrics")
            .help(concat!(
                "List of node ids that are responsible for calculating metrics"
            ))
            .new_field("cluster.roles.metrics.push")
            .label("Push Metrics")
            .help(concat!(
                "List of node ids that are responsible for pushing metrics"
            ))
            .build()
            // Forms
            .new_form_section()
            .title("Cluster settings")
            .fields(["cluster.node-id"])
            .build()
            .new_form_section()
            .title("Cluster service")
            .fields([
                "cluster.bind-addr",
                "cluster.advertise-addr",
                "cluster.bind-port",
            ])
            .build()
            .new_form_section()
            .title("Membership protocol")
            .fields(["cluster.key", "cluster.heartbeat", "cluster.seed-nodes"])
            .build()
            .new_form_section()
            .title("Node Roles")
            .fields([
                "cluster.roles.purge.stores",
                "cluster.roles.purge.accounts",
                "cluster.roles.acme.renew",
                "cluster.roles.metrics.calculate",
                "cluster.roles.metrics.push",
            ])
            .build()
            .build()
            // Web hooks
            .new_schema("web-hooks")
            .prefix("webhook")
            .suffix("url")
            .names("webhook", "webhooks")
            .new_id_field()
            .label("Webhook Id")
            .help("Unique identifier for this webhook")
            .build()
            .new_field("url")
            .label("Endpoint URL")
            .help(concat!("URL of the webhook endpoint"))
            .placeholder("https://127.0.0.1/webhook")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether Stalwart should connect to a webhook ",
                "endpoint that has an invalid TLS certificate"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout")
            .label("Timeout")
            .help(concat!(
                "Maximum amount of time that Stalwart will wait for a response ",
                "from this webhook"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("throttle")
            .label("Throttle")
            .help(concat!(
                "The minimum amount of time that must pass between ",
                "each request to the webhook endpoint"
            ))
            .default("1s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("signature-key")
            .label("Signature Key")
            .help(concat!(
                "The HMAC key used to sign the webhook request body ",
                "to prevent tampering"
            ))
            .typ(Type::Secret)
            .build()
            .new_field("headers")
            .typ(Type::Array)
            .label("HTTP Headers")
            .help("The headers to be sent with webhook requests")
            .build()
            .new_field("auth.username")
            .label("Username")
            .help(concat!(
                "The username to use when authenticating with the webhook endpoint"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("auth.secret")
            .label("Secret")
            .help(concat!(
                "The secret to use when authenticating with the webhook endpoint"
            ))
            .typ(Type::Secret)
            .build()
            .new_field("events")
            .label("Events")
            .help("Which events should trigger this webhook")
            .typ(Type::Select {
                typ: SelectType::ManyWithSearch,
                source: Source::StaticId(EVENT_NAMES),
            })
            .build()
            .new_form_section()
            .title("Webhook settings")
            .fields(["_id", "url", "signature-key", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["auth.username", "auth.secret"])
            .build()
            .new_form_section()
            .title("Triggers")
            .fields(["events"])
            .build()
            .new_form_section()
            .title("Options")
            .fields(["throttle", "timeout", "headers"])
            .build()
            .list_title("Webhooks")
            .list_subtitle("Manage Webhooks")
            .list_fields(["_id", "url"])
            .build()
            // Enterprise settings
            .new_schema("enterprise")
            // License key
            .new_field("enterprise.license-key")
            .label("License Key")
            .help(concat!(
                "Upgrade to the enterprise version of Stalwart by ",
                "entering your license key here. Obtain your license at ",
                "https://license.stalw.art/buy"
            ))
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("enterprise.api-key")
            .label("API Key")
            .help(concat!(
                "API key for license retrieval and automatic renewals. ",
                "Obtain your API key at https://license.stalw.art.",
            ))
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("enterprise.logo-url")
            .label("Default logo URL")
            .help(concat!(
                "URL to the default logo to use in the Webadmin interface. ",
                "(Enterprise feature)"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsUrl])
            .enterprise_feature()
            .build()
            .new_form_section()
            .title("Licensing")
            .fields(["enterprise.license-key", "enterprise.api-key"])
            .build()
            .new_form_section()
            .title("Branding")
            .fields(["enterprise.logo-url"])
            .build()
            .build()
            // Contact form settings
            .new_schema("form")
            .new_field("form.deliver-to")
            .label("Recipients")
            .help(concat!(
                "List of local e-mail addresses to deliver the contact form to.",
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::IsEmail])
            .build()
            .new_field("form.email.field")
            .label("E-mail field")
            .help(concat!(
                "The name of the field in the contact form that contains the ",
                "e-mail address of the sender."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsEmail])
            .build()
            .new_field("form.name.field")
            .label("Name field")
            .help(concat!(
                "The name of the field in the contact form that contains the ",
                "name of the sender."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("form.subject.field")
            .label("Subject field")
            .help(concat!(
                "The name of the field in the contact form that contains the ",
                "subject of the message."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("form.honey-pot.field")
            .label("Honey Pot field")
            .help(concat!(
                "The name of the field in the contact form that is used as a ",
                "honey pot to catch spam bots."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("form.email.default")
            .label("E-mail default")
            .help(concat!(
                "The default e-mail address to use when the sender does not ",
                "provide one."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsEmail])
            .default("postmaster@localhost")
            .build()
            .new_field("form.subject.default")
            .label("Subject default")
            .help(concat!(
                "The default subject to use when the sender does not ",
                "provide one."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .default("Contact form submission")
            .build()
            .new_field("form.name.default")
            .label("Name default")
            .help(concat!(
                "The default name to use when the sender does not ",
                "provide one."
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .default("Anonymous")
            .build()
            .new_field("form.rate-limit")
            .label("Rate limit")
            .help(concat!(
                "Maximum number of contact form submissions that can be made ",
                "in a timeframe by a given IP address."
            ))
            .default("5/1h")
            .typ(Type::Rate)
            .build()
            .new_field("form.max-size")
            .label("Max Size")
            .help(concat!(
                "Maximum size of the contact form submission in bytes."
            ))
            .typ(Type::Size)
            .default("102400")
            .build()
            .new_field("form.enable")
            .label("Enable form submissions")
            .help(concat!("Whether to enable contact form submissions."))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("form.validate-domain")
            .label("Validate email domain")
            .help(concat!(
                "Whether to validate the domain of the sender's email address."
            ))
            .typ(Type::Boolean)
            .default("true")
            .build()
            .new_form_section()
            .title("Form submission settings")
            .fields(["form.deliver-to", "form.enable"])
            .build()
            .new_form_section()
            .title("Fields")
            .fields([
                "form.email.field",
                "form.name.field",
                "form.subject.field",
                "form.honey-pot.field",
            ])
            .build()
            .new_form_section()
            .title("Security")
            .fields(["form.rate-limit", "form.max-size", "form.validate-domain"])
            .build()
            .new_form_section()
            .title("Defaults")
            .fields([
                "form.email.default",
                "form.name.default",
                "form.subject.default",
            ])
            .build()
            .build()
            // AI models
            .new_schema("ai-models")
            .prefix("enterprise.ai")
            .suffix("url")
            .names("model", "models")
            .new_id_field()
            .label("Model Id")
            .help("Unique identifier for this AI model")
            .enterprise_feature()
            .build()
            .new_field("url")
            .label("Endpoint URL")
            .help(concat!("URL of the OpenAI compatible endpoint"))
            .placeholder("https://api.openai.com/v1/chat/completions")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .enterprise_feature()
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether Stalwart should connect to an ",
                "endpoint that has an invalid TLS certificate"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .enterprise_feature()
            .build()
            .new_field("timeout")
            .label("Timeout")
            .help(concat!(
                "Maximum amount of time that Stalwart will wait for a response ",
                "from this endpoint"
            ))
            .default("2m")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .enterprise_feature()
            .build()
            .new_field("auth.token")
            .label("API token")
            .help(concat!(
                "The API token used to authenticate with the AI model endpoint"
            ))
            .typ(Type::Secret)
            .enterprise_feature()
            .build()
            .new_field("headers")
            .typ(Type::Array)
            .label("HTTP Headers")
            .help("The headers to be sent with requests")
            .enterprise_feature()
            .build()
            .new_field("default-temperature")
            .label("Temperature")
            .help(concat!(
                "The temperature of the AI model, which controls the randomness ",
                "of the output. A higher temperature will produce more random output."
            ))
            .typ(Type::Input)
            .default("0.7")
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(NumberType::Float(0.0)),
                    Validator::MaxValue(NumberType::Float(1.0)),
                ],
            )
            .enterprise_feature()
            .build()
            .new_field("model")
            .label("Model")
            .help(concat!("The name of the AI model to use.",))
            .typ(Type::Input)
            .placeholder("gpt-4")
            .input_check([Transformer::Trim], [Validator::Required])
            .enterprise_feature()
            .build()
            .new_field("type")
            .label("Type")
            .help("API type")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[("chat", "Chat Completion"), ("text", "Text Generation")]),
            })
            .default("chat")
            .enterprise_feature()
            .build()
            .new_form_section()
            .title("AI Endpoint settings")
            .fields(["_id", "url", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Model")
            .fields(["type", "model"])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["auth.token"])
            .build()
            .new_form_section()
            .title("Options")
            .fields(["timeout", "headers"])
            .build()
            .list_title("AI Models")
            .list_subtitle("Manage AI Models")
            .list_fields(["_id", "model", "type"])
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
        .typ(Type::Input)
        .input_check([Transformer::Trim], vec![Validator::MinValue(1.into())])
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
