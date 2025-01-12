/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

use super::SMTP_RCPT_TO_VARS;

impl Builder<Schemas, ()> {
    pub fn build_sieve(self) -> Self {
        let rcpt_vars = ExpressionValidator::new(SMTP_RCPT_TO_VARS, &[]);

        self.new_schema("sieve-settings")
            .new_field("sieve.untrusted.disable-capabilities")
            .label("Disable Capabilities")
            .help(concat!(
                "List of capabilities to disable in the untrusted interpreter"
            ))
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("sieve.untrusted.notification-uris")
            .label("Notification URIs")
            .help(concat!("List of allowed URIs for the notify extension"))
            .default("mailto")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("sieve.untrusted.protected-headers")
            .label("Protected Headers")
            .help(concat!(
                "List of headers that cannot be deleted or added using the editheader extension"
            ))
            .default(
                &[
                    "Original-Subject",
                    "Original-From",
                    "Received",
                    "Auto-Submitted",
                ][..],
            )
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("sieve.untrusted.vacation.default-subject")
            .label("Default Subject")
            .help(concat!("Default subject of vacation responses"))
            .default("Automated reply")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("sieve.untrusted.vacation.subject-prefix")
            .label("Default Prefix")
            .help(concat!("Default subject prefix of vacation responses"))
            .default("Auto: ")
            .typ(Type::Input)
            .build()
            .new_field("sieve.untrusted.default-expiry.vacation")
            .label("Default Expiry")
            .help(concat!(
                "Default expiration time for IDs stored by the vacation ",
                "extension"
            ))
            .default("40d")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("sieve.untrusted.default-expiry.duplicate")
            .label("Untrusted Expiry")
            .help(concat!(
                "Default expiration time for IDs stored by the duplicate ",
                "extension from untrusted scripts"
            ))
            .default("7d")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("sieve.trusted.from-name")
            .label("From Name")
            .help(concat!(
                "Default name to use for the from field in email notifications ",
                "sent from a Sieve script"
            ))
            .default("'Automated Message'")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .new_field("sieve.trusted.from-addr")
            .label("From Address")
            .help(concat!(
                "Default email address to use for the from field in email ",
                "notifications sent from a Sieve script"
            ))
            .default("'MAILER-DAEMON@' + config_get('report.domain')")
            .new_field("sieve.trusted.return-path")
            .label("Return Path")
            .help(concat!(
                "Default return path to use in email notifications sent from ",
                "a Sieve script"
            ))
            .default("")
            .new_field("sieve.trusted.sign")
            .label("DKIM Signatures")
            .help(concat!(
                "DKIM signatures to add to email notifications sent from ",
                "a Sieve script"
            ))
            .default(
                "['rsa-' + config_get('report.domain'), 'ed25519-' + config_get('report.domain')]",
            )
            .build()
            .new_field("sieve.trusted.hostname")
            .label("Hostname")
            .help(concat!(
                "Override the default local hostname to use when generating ",
                "a Message-Id header"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("sieve.trusted.no-capability-check")
            .label("Allow undeclared capabilities")
            .help(concat!(
                "If enabled, language extensions can be used without being ",
                "explicitly declared using the require statement"
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("sieve.trusted.limits.duplicate-expiry")
            .label("Trusted Expiry")
            .help(concat!(
                "Default expiration time for IDs stored by the duplicate ",
                "extension from trusted scripts"
            ))
            .default("7d")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Untrusted Interpreter")
            .fields([
                "sieve.untrusted.notification-uris",
                "sieve.untrusted.protected-headers",
                "sieve.untrusted.disable-capabilities",
            ])
            .build()
            .new_form_section()
            .title("Trusted Interpreter")
            .fields([
                "sieve.trusted.from-name",
                "sieve.trusted.from-addr",
                "sieve.trusted.return-path",
                "sieve.trusted.sign",
                "sieve.trusted.hostname",
                "sieve.trusted.no-capability-check",
            ])
            .build()
            .new_form_section()
            .title("Vacation Extension")
            .fields([
                "sieve.untrusted.vacation.default-subject",
                "sieve.untrusted.vacation.subject-prefix",
                "sieve.untrusted.default-expiry.vacation",
            ])
            .build()
            .new_form_section()
            .title("Duplicate Extension")
            .fields([
                "sieve.untrusted.default-expiry.duplicate",
                "sieve.trusted.limits.duplicate-expiry",
            ])
            .build()
            .build()
            // Limits
            .new_schema("sieve-limits")
            .new_field("sieve.untrusted.limits.name-length")
            .label("Name Length")
            .help(concat!("Maximum length of a script name"))
            .default("512")
            .typ(Type::Input)
            .input_check([], [Validator::Required, Validator::MinValue(1.into())])
            .new_field("sieve.untrusted.limits.max-scripts")
            .label("Maximum Scripts")
            .help(concat!("Maximum number of scripts a user can have"))
            .default("256")
            .new_field("sieve.untrusted.limits.script-size")
            .label("Script Size")
            .help(concat!("Maximum size of a script"))
            .default("102400")
            .typ(Type::Size)
            .new_field("sieve.untrusted.limits.string-length")
            .label("String Length")
            .help(concat!("Maximum length of a string"))
            .default("4096")
            .typ(Type::Input)
            .new_field("sieve.untrusted.limits.variable-name-length")
            .label("Variable Name Length")
            .help(concat!("Maximum length of a variable name"))
            .default("32")
            .new_field("sieve.untrusted.limits.variable-size")
            .label("Variable Size")
            .help(concat!("Maximum size of a variable"))
            .default("4096")
            .new_field("sieve.untrusted.limits.nested-blocks")
            .label("Nested Blocks")
            .help(concat!("Maximum number of nested blocks"))
            .default("15")
            .new_field("sieve.untrusted.limits.nested-tests")
            .label("Nested Tests")
            .help(concat!("Maximum number of nested tests"))
            .default("15")
            .new_field("sieve.untrusted.limits.nested-foreverypart")
            .label("Nested Foreach")
            .help(concat!("Maximum number of nested foreach blocks"))
            .default("3")
            .new_field("sieve.untrusted.limits.match-variables")
            .label("Match Variables")
            .help(concat!("Maximum number of match variables"))
            .default("30")
            .new_field("sieve.untrusted.limits.local-variables")
            .label("Local Variables")
            .help(concat!("Maximum number of local variables"))
            .default("128")
            .new_field("sieve.untrusted.limits.header-size")
            .label("Header Size")
            .help(concat!("Maximum size of a header"))
            .default("1024")
            .new_field("sieve.untrusted.limits.includes")
            .label("Includes")
            .help(concat!("Maximum number of includes"))
            .default("3")
            .new_field("sieve.untrusted.limits.nested-includes")
            .label("Nested Includes")
            .help(concat!("Maximum number of nested includes"))
            .default("3")
            .new_field("sieve.untrusted.limits.cpu")
            .label("CPU")
            .help(concat!("Maximum number CPU cycles a script can use"))
            .default("5000")
            .new_field("sieve.untrusted.limits.received-headers")
            .label("Received Headers")
            .help(concat!("Maximum number of received headers"))
            .default("10")
            .new_field("sieve.untrusted.limits.redirects")
            .label("Redirects")
            .help(concat!("Maximum number of redirects"))
            .default("1")
            .input_check([], [Validator::Required, Validator::MinValue(0.into())])
            .new_field("sieve.untrusted.limits.outgoing-messages")
            .label("Outgoing Messages")
            .help(concat!("Maximum number of outgoing messages"))
            .default("3")
            .new_field("sieve.trusted.limits.redirects")
            .label("Redirects")
            .help(concat!("Maximum number of redirects"))
            .default("3")
            .new_field("sieve.trusted.limits.out-messages")
            .label("Outgoing Messages")
            .help(concat!("Maximum number of outgoing messages"))
            .default("5")
            .new_field("sieve.trusted.limits.received-headers")
            .label("Received Headers")
            .help(concat!("Maximum number of received headers"))
            .default("50")
            .input_check([], [Validator::Required, Validator::MinValue(1.into())])
            .new_field("sieve.trusted.limits.cpu")
            .label("CPU")
            .help(concat!("Maximum number CPU cycles a script can use"))
            .default("1048576")
            .typ(Type::Input)
            .new_field("sieve.trusted.limits.nested-includes")
            .label("Nested Includes")
            .help(concat!("Maximum number of nested includes"))
            .default("5")
            .build()
            .new_form_section()
            .title("Untrusted Limits")
            .fields([
                "sieve.untrusted.limits.name-length",
                "sieve.untrusted.limits.max-scripts",
                "sieve.untrusted.limits.script-size",
                "sieve.untrusted.limits.string-length",
                "sieve.untrusted.limits.variable-name-length",
                "sieve.untrusted.limits.variable-size",
                "sieve.untrusted.limits.nested-blocks",
                "sieve.untrusted.limits.nested-tests",
                "sieve.untrusted.limits.nested-foreverypart",
                "sieve.untrusted.limits.match-variables",
                "sieve.untrusted.limits.local-variables",
                "sieve.untrusted.limits.header-size",
                "sieve.untrusted.limits.includes",
                "sieve.untrusted.limits.nested-includes",
                "sieve.untrusted.limits.cpu",
                "sieve.untrusted.limits.redirects",
                "sieve.untrusted.limits.received-headers",
                "sieve.untrusted.limits.outgoing-messages",
            ])
            .build()
            .new_form_section()
            .title("Trusted Limits")
            .fields([
                "sieve.trusted.limits.redirects",
                "sieve.trusted.limits.out-messages",
                "sieve.trusted.limits.received-headers",
                "sieve.trusted.limits.cpu",
                "sieve.trusted.limits.nested-includes",
            ])
            .build()
            .build()
            // Trusted Scripts
            .new_schema("trusted-script")
            .prefix("sieve.trusted.scripts")
            .suffix("contents")
            .names("script", "scripts")
            .new_id_field()
            .label("Script Id")
            .help("Unique identifier for the script")
            .build()
            .new_field("name")
            .label("Description")
            .help("Brief description of the Sieve script")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("contents")
            .label("Contents")
            .help("Contents of the trusted Sieve script")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Trusted Sieve Script")
            .fields(["_id", "name", "contents"])
            .build()
            .list_title("System Sieve scripts")
            .list_subtitle("Manage Sieve scripts executed by the trusted interpreter")
            .list_fields(["_id", "name"])
            .build()
            // Untrusted Scripts
            .new_schema("untrusted-script")
            .prefix("sieve.untrusted.scripts")
            .suffix("contents")
            .names("script", "scripts")
            .new_id_field()
            .label("Script Id")
            .help("Unique identifier for the script")
            .build()
            .new_field("name")
            .label("Description")
            .help("Brief description of the Sieve script")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("contents")
            .label("Contents")
            .help("Contents of the Sieve script")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Untrusted Sieve Script")
            .fields(["_id", "name", "contents"])
            .build()
            .list_title("User Sieve scripts")
            .list_subtitle("Manage untrusted Sieve scripts that can be imported by users")
            .list_fields(["_id", "name"])
            .build()
    }
}
