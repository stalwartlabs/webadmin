use crate::core::schema::*;
/*

- Inbound
  - Connect stage
  - EHLO stage
  - AUTH stage
  - MAIL stage
  - RCPT stage
  - DATA stage
  - Extensions
  - Limits
  - Throttling
  - Milter
  - Pipes
- Outbound
  - Queue
  - Routing (+source IP)
  - TLS
  - Limits & Timeouts
  - Resolver
  - Remote Servers
  - Throttling
  - Quotas
- DKIM
  - Signatures
  - Verify
  - Sign
  - Report
- SPF
  - Verify (EHLO, MAIL FROM)
  - Report
- ARC
  - Verify
  - Seal
- DMARC
   - Verify
   - Report
- Reporting
   - Analysis
   - DSN
   - TLS
*/
impl Builder<Schemas, ()> {
    pub fn build_smtp(self) -> Self {
        let connect_expr = ExpressionValidator::default()
            .variables(CONNECT_VARIABLES)
            .functions(FUNCTIONS_MAP);
        let extensions_expr = ExpressionValidator::default()
            .variables(EXTENSIONS_VARIABLES)
            .functions(FUNCTIONS_MAP);
        let mail_expr = ExpressionValidator::default()
            .variables(MAIL_VARIABLES)
            .functions(FUNCTIONS_MAP);
        let rcpt_expr = ExpressionValidator::default()
            .variables(RCPT_VARIABLES)
            .functions(FUNCTIONS_MAP);
        let data_expr = ExpressionValidator::default()
            .variables(DATA_VARIABLES)
            .functions(FUNCTIONS_MAP);

        // Connect
        self.new_schema("smtp-in-connect")
            .new_field("session.connect.script")
            .typ(Type::Expression)
            .label("Run Script")
            .help("Which Sieve script to run when a client connects")
            .input_check([], [Validator::IsValidExpression(connect_expr)])
            .build()
            .new_field("auth.iprev.verify")
            .typ(Type::Expression)
            .label("IPRev Verify")
            .help("How strict to be when verifying the reverse DNS of the client IP")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr.constants(VERIFY_CONSTANTS)),
                ],
            )
            .default("relaxed")
            .build()
            .new_form_section()
            .title("Connect Stage")
            .fields(["session.connect.script", "auth.iprev.verify"])
            .build()
            .build()
            // EHLO stage
            .new_schema("smtp-in-ehlo")
            .new_field("session.ehlo.require")
            .label("Require EHLO")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr),
                ],
            )
            .default("true")
            .help(concat!(
                "Whether the remote client must send an EHLO command ",
                "before starting an SMTP transaction"
            ))
            .build()
            .new_field("session.ehlo.reject-non-fqdn")
            .label("Reject Non-FQDN")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr),
                ],
            )
            .help(concat!(
                "Whether to reject EHLO commands that do not include a ",
                "fully-qualified domain name as a parameter"
            ))
            .default("true")
            .build()
            .new_field("session.ehlo.script")
            .label("Run Script")
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(connect_expr)])
            .help("Which Sieve script to run after the client sends an EHLO command")
            .build()
            .new_form_section()
            .title("EHLO Stage")
            .fields([
                "session.ehlo.require",
                "session.ehlo.reject-non-fqdn",
                "session.ehlo.script",
            ])
            .build()
            .build()
            // Limits
            .new_schema("smtp-in-limits")
            .new_field("session.timeout")
            .label("Timeout")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr),
                ],
            )
            .default("5m")
            .help("How long to wait for a client to send a command before timing out")
            .new_field("session.transfer-limit")
            .label("Bytes Limit")
            .default("262144000")
            .help("The maximum number of bytes that can be transferred per session")
            .new_field("session.duration")
            .label("Duration")
            .default("10m")
            .help("The maximum duration of a session")
            .build()
            .new_form_section()
            .title("SMTP Session Limits")
            .fields([
                "session.timeout",
                "session.transfer-limit",
                "session.duration",
            ])
            .build()
            .build()
            // Extensions
            .new_schema("smtp-in-extensions")
            .new_field("session.extensions.pipelining")
            .label("Pipelining")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(extensions_expr),
                ],
            )
            .default("true")
            .help(concat!(
                "Enables SMTP pipelining (RFC 2920), which enables multiple ",
                "commands to be sent in a single request to speed up communication ",
                "between the client and server"
            ))
            .new_field("session.extensions.chunking")
            .label("Chunking")
            .help(concat!(
                "Enables chunking (RFC 1830), an extension that allows large ",
                "messages to be transferred in chunks which may reduce the load ",
                "on the network and server."
            ))
            .default("true")
            .new_field("session.extensions.requiretls")
            .label("Require TLS")
            .help(concat!(
                "Enables require TLS (RFC 8689), an extension that allows",
                " clients to require TLS encryption for the SMTP session"
            ))
            .default("true")
            .new_field("session.extensions.no-soliciting")
            .label("No-Soliciting")
            .help(concat!(
                "Specifies the text to include in the NOSOLICITING (RFC 3865) ",
                "message, which indicates that the server does not accept unsolicited ",
                "commercial email (UCE or spam)"
            ))
            .default("\"\"")
            .new_field("session.extensions.dsn")
            .label("DSN")
            .help(concat!(
                "Enables delivery status notifications (RFC 3461), which allows ",
                "the sender to request a delivery status notification (DSN) from ",
                "the recipient's mail server"
            ))
            .default("false")
            .new_field("session.extensions.expn")
            .label("EXPN")
            .help(concat!(
                "Specifies whether to enable the EXPN command, which allows ",
                "the sender to request the membership of a mailing list. It is ",
                "recommended to disable this command to prevent spammers ",
                "from harvesting email addresses"
            ))
            .default("false")
            .new_field("session.extensions.vrfy")
            .label("VRFY")
            .help(concat!(
                "Specifies whether to enable the VRFY command, which allows ",
                "the sender to verify the existence of a mailbox. It is recommended ",
                "to disable this command to prevent spammers from ",
                "harvesting email addresses"
            ))
            .default("false")
            .new_field("session.extensions.future-release")
            .label("Future Release")
            .help(concat!(
                "Specifies the maximum time that a message can be held for ",
                "delivery using the FUTURERELEASE (RFC 4865) extension"
            ))
            .default("false")
            .new_field("session.extensions.deliver-by")
            .label("Deliver By")
            .help(concat!(
                "Specifies the maximum delivery time for a message using the ",
                "DELIVERBY (RFC 2852) extension, which allows the sender to request ",
                "a specific delivery time for a message"
            ))
            .default("false")
            .new_field("session.extensions.mt-priority")
            .label("MT Priority")
            .help(concat!(
                "Specifies the priority assignment policy to advertise on the ",
                "MT-PRIORITY (RFC 6710) extension, which allows the sender to specify ",
                "a priority for a message. Available policies are mixer, stanag4406 a",
                "nd nsep, or false to disable this extension"
            ))
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(extensions_expr.constants(&[
                        "mixer",
                        "stanag4406",
                        "nsep",
                    ])),
                ],
            )
            .default("false")
            .build()
            .new_form_section()
            .title("SMTP Extensions")
            .fields([
                "session.extensions.pipelining",
                "session.extensions.chunking",
                "session.extensions.requiretls",
                "session.extensions.no-soliciting",
                "session.extensions.dsn",
                "session.extensions.expn",
                "session.extensions.vrfy",
                "session.extensions.future-release",
                "session.extensions.deliver-by",
                "session.extensions.mt-priority",
            ])
            .build()
            .build()
            // AUTH stage
            .new_schema("smtp-in-auth")
            .new_field("session.auth.directory")
            .label("Directory")
            .help("Specifies the directory to use for authentication")
            .default("false")
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(connect_expr),
                ],
            )
            .new_field("session.auth.require")
            .label("Require Authentication")
            .help(concat!(
                "Specifies whether authentication is necessary to send email messages"
            ))
            .default("false")
            .new_field("session.auth.allow-plain-text")
            .label("Allow Plain Auth")
            .help(concat!(
                "Specifies whether to allow authentication using the PLAIN mechanism ",
                "over an unencrypted connection"
            ))
            .default("false")
            .new_field("session.auth.must-match-sender")
            .label("Must match sender")
            .help(concat!(
                "Specifies whether the authenticated user or any of their associated ",
                "e-mail addresses must match the sender of the email message"
            ))
            .default("true")
            .new_field("session.auth.errors.total")
            .label("Max Errors")
            .help(concat!(
                "Maximum number of authentication errors allowed before the session ",
                "is disconnected"
            ))
            .default("3")
            .new_field("session.auth.errors.wait")
            .label("Error wait")
            .help("Time interval to wait after an authentication failure")
            .default("5s")
            .new_field("session.auth.mechanisms")
            .label("Allowed Mechanisms")
            .help(concat!(
                "A list of SASL authentication mechanisms offered to clients, or an ",
                "empty list to disable authentication. Stalwart SMTP currently supports PLAIN, ",
                "LOGIN, and OAUTHBEARER mechanisms"
            ))
            .default("false")
            .input_check(
                [],
                [Validator::IsValidExpression(
                    connect_expr.constants(AUTH_CONSTANTS),
                )],
            )
            .build()
            .new_form_section()
            .title("AUTH Stage")
            .fields([
                "session.auth.directory",
                "session.auth.require",
                "session.auth.allow-plain-text",
                "session.auth.must-match-sender",
                "session.auth.mechanisms",
            ])
            .build()
            .new_form_section()
            .title("Authentication Errors")
            .fields(["session.auth.errors.total", "session.auth.errors.wait"])
            .build()
            .build()
            // MAIL stage
            .new_schema("smtp-in-mail")
            .new_field("session.mail.rewrite")
            .label("Sender Rewriting")
            .help("Expression to rewrite the sender address")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(mail_expr)],
            )
            .default("false")
            .new_field("session.mail.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a MAIL command")
            .input_check([], [Validator::IsValidExpression(mail_expr)])
            .build()
            .new_form_section()
            .title("MAIL FROM Stage")
            .fields(["session.mail.rewrite", "session.mail.script"])
            .build()
            .build()
            // RCPT stage
            .new_schema("smtp-in-rcpt")
            .new_field("session.rcpt.directory")
            .label("Directory")
            .help("Directory to use to validate local recipients")
            .default("\"\"")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_expr)],
            )
            .new_field("session.rcpt.relay")
            .label("Allow Relaying")
            .help("Whether to allow relaying for non-local recipients")
            .default("false")
            .new_field("session.rcpt.max-recipients")
            .label("Max Recipients")
            .help("Maximum number of recipients per message")
            .default("25")
            .new_field("session.rcpt.rewrite")
            .label("Recipient Rewriting")
            .help("Expression to rewrite the recipient address")
            .default("false")
            .new_field("session.rcpt.errors.total")
            .label("Max Errors")
            .help(concat!(
                "Maximum number of recipient errors before ",
                "the session is disconnected"
            ))
            .default("5")
            .new_field("session.rcpt.errors.wait")
            .label("Error wait")
            .help("Amount of time to wait after a recipient error")
            .default("5s")
            .new_field("session.rcpt.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a RCPT command")
            .input_check([], [Validator::IsValidExpression(rcpt_expr)])
            .build()
            .new_form_section()
            .title("RCPT TO Stage")
            .fields([
                "session.rcpt.directory",
                "session.rcpt.relay",
                "session.rcpt.max-recipients",
                "session.rcpt.rewrite",
                "session.rcpt.script",
            ])
            .build()
            .new_form_section()
            .title("Recipient Errors")
            .fields(["session.rcpt.errors.total", "session.rcpt.errors.wait"])
            .build()
            .build()
            // DATA stage
            .new_schema("smtp-in-data")
            .new_field("session.data.script")
            .label("Run Script")
            .help("Which Sieve script to run after the client sends a DATA command")
            .typ(Type::Expression)
            .input_check([], [Validator::IsValidExpression(data_expr)])
            .new_field("session.data.limits.messages")
            .label("Messages")
            .help("Maximum number of messages that can be submitted per SMTP session")
            .default("10")
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(data_expr)],
            )
            .new_field("session.data.limits.size")
            .label("Size")
            .help("Maximum size of a message in bytes")
            .default("104857600")
            .new_field("session.data.limits.received-headers")
            .label("Received Headers")
            .help(concat!(
                "Maximum limit on the number of Received headers, ",
                "which helps to prevent message loops"
            ))
            .default("50")
            .new_field("session.data.add-headers.received")
            .label("Received")
            .help("Whether to add a Received header to the message")
            .default("false")
            .new_field("session.data.add-headers.received-spf")
            .label("Received-SPF")
            .help("Whether to add a Received-SPF header to the message")
            .default("false")
            .new_field("session.data.add-headers.auth-results")
            .label("Authentication-Results")
            .help("Whether to add an Authentication-Results header to the message")
            .default("false")
            .new_field("session.data.add-headers.message-id")
            .label("Message-Id")
            .help("Whether to add a Message-Id header to the message")
            .default("false")
            .new_field("session.data.add-headers.date")
            .label("Date")
            .help("Whether to add a Date header to the message")
            .default("false")
            .new_field("session.data.add-headers.return-path")
            .label("Return-Path")
            .help("Whether to add a Return-Path header to the message")
            .default("false")
            .build()
            .new_form_section()
            .title("DATA Stage")
            .fields(["session.data.script"])
            .build()
            .new_form_section()
            .title("Limits")
            .fields([
                "session.data.limits.messages",
                "session.data.limits.size",
                "session.data.limits.received-headers",
            ])
            .build()
            .new_form_section()
            .title("Add Headers")
            .fields([
                "session.data.add-headers.received",
                "session.data.add-headers.received-spf",
                "session.data.add-headers.auth-results",
                "session.data.add-headers.message-id",
                "session.data.add-headers.date",
                "session.data.add-headers.return-path",
            ])
            .build()
            .build()
            // Throttle
            .new_schema("smtp-in-throttle")
            .prefix("session.throttle")
            .names("throttle", "throttles")
            .suffix("enable")
            .new_id_field()
            .label("Throttle ID")
            .help("Unique identifier for the throttle")
            .build()
            .new_field("enable")
            .label("Enabled")
            .help("Whether to enable this throttle")
            .typ(Type::Boolean)
            .default("true")
            .build()
            .new_field("key")
            .label("Keys")
            .help(concat!(
                "Optional list of context variables that determine ",
                "where this throttle should be applied"
            ))
            .typ(Type::Select {
                multi: true,
                source: Source::Static(&[
                    (V_LISTENER, "Listener"),
                    (V_REMOTE_IP, "Remote IP"),
                    (V_LOCAL_IP, "Local IP"),
                    (V_AUTHENTICATED_AS, "Authenticated As"),
                    (V_HELO_DOMAIN, "EHLO Domain"),
                    (V_SENDER, "Sender"),
                    (V_SENDER_DOMAIN, "Sender Domain"),
                    (V_RECIPIENT, "Recipient"),
                    (V_RECIPIENT_DOMAIN, "Recipient Domain"),
                ]),
            })
            .build()
            .new_field("match")
            .label("Match condition")
            .help(concat!(
                "enable the imposition of concurrency and rate limits only ",
                "when a specific condition is met"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::IsValidExpression(
                        ExpressionValidator::default()
                            .variables(THROTTLE_IN_VARIABLES)
                            .functions(FUNCTIONS_MAP),
                    ),
                    Validator::MaxItems(1),
                ],
            )
            .build()
            .new_field("concurrency")
            .label("Concurrency")
            .help(concat!(
                "Maximum number of concurrent connections that ",
                "the throttle will allow"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::MinValue(1.into())])
            .build()
            .new_field("rate")
            .label("Rate limit")
            .help(concat!(
                "Number of incoming requests over a period of time ",
                "that the rate limiter will allow"
            ))
            .typ(Type::Rate)
            .build()
            .new_form_section()
            .title("Throttle")
            .fields(["_id", "key", "concurrency", "rate", "match", "enable"])
            .build()
            .list_title("Throttles")
            .list_subtitle("Manage inbound concurrency and rate limits")
            .list_fields(["_id", "concurrency", "rate", "enable"])
            .build()
            // Milter
            .new_schema("milter")
            .prefix("session.data.milter")
            .suffix("hostname")
            .names("milter", "milters")
            .new_id_field()
            .label("Milter Id")
            .help("Unique identifier for this milter")
            .build()
            .new_field("enable")
            .label("Enable")
            .help("Expression that determines whether to enable this milter")
            .default("true")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(data_expr)],
            )
            .build()
            .new_field("hostname")
            .label("Hostname")
            .help(concat!(
                "Hostname or IP address of the server where the Milter ",
                "filter is running"
            ))
            .placeholder("127.0.0.1")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            .build()
            .new_field("port")
            .label("Port")
            .help("Network port on the Milter filter host server")
            .placeholder("11332")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .build()
            .new_field("tls")
            .label("Enable TLS")
            .help(concat!(
                "Whether to use Transport Layer Security (TLS) for the connection ",
                "between Stalwart SMTP and the Milter filter"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help(concat!(
                "Whether Stalwart SMTP should accept connections to a Milter filter ",
                "server that has an invalid TLS certificate"
            ))
            .default("false")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.connect")
            .label("Connection")
            .help(concat!(
                "Maximum amount of time that Stalwart SMTP will wait to establish ",
                "a connection with a Milter server"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.command")
            .label("Command")
            .help(concat!(
                "How long Stalwart SMTP will wait to send a command to the ",
                "Milter server"
            ))
            .default("30s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("timeout.data")
            .label("Data")
            .help(concat!(
                "Maximum amount of time Stalwart SMTP will wait for a response",
                " from the Milter server"
            ))
            .default("60s")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.tempfail-on-error")
            .label("TempFail on Error")
            .help(concat!(
                "Whether to respond with a temporary failure (typically a 4xx ",
                "SMTP status code) when Stalwart encounters an error while ",
                "communicating with a Milter server"
            ))
            .default("true")
            .typ(Type::Boolean)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.max-response-size")
            .label("Max Response")
            .help(concat!(
                "Maximum size, in bytes, of a response that Stalwart will accept",
                " from a Milter server"
            ))
            .default("52428800")
            .typ(Type::Size)
            .input_check([], [Validator::Required])
            .build()
            .new_field("options.version")
            .label("Protocol Version")
            .help(concat!(
                "Version of the Milter protocol that Stalwart SMTP should use when",
                " communicating with the Milter server"
            ))
            .default("6")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[("2", "Version 2"), ("6", "Version 6")]),
            })
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Milter settings")
            .fields(["_id", "hostname", "port", "enable"])
            .build()
            .new_form_section()
            .title("TLS")
            .fields(["tls", "allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Options")
            .fields([
                "options.max-response-size",
                "options.version",
                "options.tempfail-on-error",
            ])
            .build()
            .new_form_section()
            .title("Timeouts")
            .fields(["timeout.connect", "timeout.command", "timeout.data"])
            .build()
            .list_title("Milter filters")
            .list_subtitle("Manage Milter filters")
            .list_fields(["_id", "hostname", "port"])
            .build()
            // Pipes
            .new_schema("pipe")
            .prefix("session.data.pipe")
            .suffix("command")
            .new_id_field()
            .label("Pipe Id")
            .help("Unique identifier for this pipe")
            .build()
            .new_field("command")
            .label("Command")
            .help("Command name to execute")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(data_expr)],
            )
            .new_field("arguments")
            .label("Arguments")
            .help("Arguments to pass to the command")
            .default("[]")
            .new_field("timeout")
            .label("Timeout")
            .help("Maximum time to wait for the command to complete")
            .default("30s")
            .build()
            .new_form_section()
            .title("Pipe settings")
            .fields(["_id", "command", "arguments", "timeout"])
            .build()
            .list_title("Pipes")
            .list_subtitle("Manage external filters (pipes)")
            .list_fields(["_id", "command", "arguments"])
            .build()
    }
}

const V_RECIPIENT: &str = "rcpt";
const V_RECIPIENT_DOMAIN: &str = "rcpt_domain";
const V_SENDER: &str = "sender";
const V_SENDER_DOMAIN: &str = "sender_domain";
const V_MX: &str = "mx";
const V_HELO_DOMAIN: &str = "helo_domain";
const V_AUTHENTICATED_AS: &str = "authenticated_as";
const V_LISTENER: &str = "listener";
const V_REMOTE_IP: &str = "remote_ip";
const V_LOCAL_IP: &str = "local_ip";
const V_PRIORITY: &str = "priority";

const CONNECT_VARIABLES: &[&str] = &[V_LISTENER, V_REMOTE_IP, V_LOCAL_IP];
const SENDER_VARIABLES: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_PRIORITY,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
];
const MAIL_VARIABLES: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_HELO_DOMAIN,
];
const RCPT_VARIABLES: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_RECIPIENT,
    V_RECIPIENT_DOMAIN,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_HELO_DOMAIN,
];
const DATA_VARIABLES: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_PRIORITY,
    V_HELO_DOMAIN,
];
const EXTENSIONS_VARIABLES: &[&str] = &[
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_SENDER,
    V_SENDER_DOMAIN,
    V_AUTHENTICATED_AS,
];
const THROTTLE_IN_VARIABLES: &[&str] = &[
    V_SENDER,
    V_SENDER_DOMAIN,
    V_RECIPIENT,
    V_RECIPIENT_DOMAIN,
    V_AUTHENTICATED_AS,
    V_LISTENER,
    V_REMOTE_IP,
    V_LOCAL_IP,
    V_PRIORITY,
    V_HELO_DOMAIN,
];
const FUNCTIONS_MAP: &[(&str, u32)] = &[
    ("is_local_domain", 2),
    ("is_local_address", 2),
    ("key_get", 2),
    ("key_exists", 2),
    ("key_set", 3),
    ("counter_incr", 3),
    ("counter_get", 2),
    ("dns_query", 2),
    ("sql_query", 3),
];

const VERIFY_CONSTANTS: &[&str] = &["relaxed", "strict", "disable", "disabled", "never", "none"];
const AUTH_CONSTANTS: &[&str] = &["plain", "login", "xoauth2", "oauthbearer"];
