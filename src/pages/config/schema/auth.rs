/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use crate::core::{form::Expression, schema::*};

use super::{smtp::*, CONNECTION_VARS, RCPT_DOMAIN_VARS, SMTP_RCPT_TO_VARS};

impl Builder<Schemas, ()> {
    pub fn build_mail_auth(self) -> Self {
        let conn_vars = ExpressionValidator::new(CONNECTION_VARS, &[]);
        let rcpt_domain = ExpressionValidator::new(RCPT_DOMAIN_VARS, &[]);
        let rcpt_vars = ExpressionValidator::new(SMTP_RCPT_TO_VARS, &[]);

        self.new_schema("signature")
            .prefix("signature")
            .suffix("algorithm")
            .names("signature", "signatures")
            .new_id_field()
            .label("Signature ID")
            .help("Unique identifier for the signature")
            .build()
            .new_field("algorithm")
            .label("Algorithm")
            .help(concat!("Encryption algorithm used for the DKIM signature"))
            .default("ed25519-sha256")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("ed25519-sha256", "Ed25519 SHA-256"),
                    ("rsa-sha256", "RSA SHA-256"),
                    ("rsa-sha1", "RSA SHA-1"),
                ]),
            })
            .build()
            .new_field("private-key")
            .label("Private Key")
            .help(concat!(
                "Contents of the private key PEM used to sign messages"
            ))
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_field("domain")
            .label("Domain Name")
            .help(concat!("Domain name associated with the DKIM signature"))
            .placeholder("example.com")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsDomain],
            )
            .build()
            .new_field("selector")
            .label("Selector")
            .help(concat!("Selector used to identify the DKIM public key"))
            .default("stalwart")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsId])
            .build()
            .new_field("headers")
            .label("Headers")
            .help(concat!("List of headers to be signed"))
            .default(&["From", "To", "Date", "Subject", "Message-ID"][..])
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("canonicalization")
            .label("Canonicalization")
            .help(concat!(
                "Method used to canonicalize the signed headers ",
                "and body of the message"
            ))
            .default("relaxed/relaxed")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("relaxed/relaxed", "Relaxed/Relaxed"),
                    ("simple/simple", "Simple/Simple"),
                    ("relaxed/simple", "Relaxed/Simple"),
                    ("simple/relaxed", "Simple/Relaxed"),
                ]),
            })
            .input_check([], [])
            .build()
            .new_field("expire")
            .label("Expiration")
            .help(concat!("Amount of time this DKIM signature is valid for"))
            .typ(Type::Duration)
            .build()
            .new_field("third-party")
            .label("Authorized Party")
            .help(concat!("Authorized third-party signature value"))
            .default("")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("third-party-algo")
            .label("Hash Algorithm")
            .help(concat!(
                "Hashing algorithm used to verify third-party ",
                "signature DNS records"
            ))
            .default("")
            .typ(Type::Select {
                multi: false,
                source: Source::Static(&[
                    ("", "Disabled"),
                    ("sha256", "SHA-256"),
                    ("sha1", "SHA-1"),
                ]),
            })
            .input_check([], [])
            .build()
            .new_field("auid")
            .label("Agent User ID")
            .help(concat!("Agent user identifier"))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("report")
            .label("Request Reports")
            .help(concat!(
                "Whether to request reports when the signature ",
                "verification fails"
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_form_section()
            .title("DKIM Signature")
            .fields([
                "_id",
                "algorithm",
                "domain",
                "selector",
                "headers",
                "canonicalization",
            ])
            .build()
            .new_form_section()
            .title("Key")
            .fields(["private-key"])
            .build()
            .new_form_section()
            .title("Options")
            .fields([
                "expire",
                "third-party",
                "third-party-algo",
                "auid",
                "report",
            ])
            .build()
            .new_form_section()
            .title("Authorized Third-Party Signatures")
            .fields(["third-party", "third-party-algo"])
            .build()
            .list_title("DKIM Signatures")
            .list_subtitle("Manage DKIM signatures used to sign and seal outgoing messages")
            .list_fields(["_id", "domain", "selector", "algorithm", "expire"])
            .build()
            // DKIM Settings
            .new_schema("dkim")
            .new_field("auth.dkim.verify")
            .label("Strategy")
            .help(concat!(
                "Whether DKIM verification is strict, relaxed or disabled"
            ))
            .default(Expression::new([], "relaxed"))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .new_field("auth.dkim.sign")
            .label("Signature")
            .help(concat!("List of DKIM signatures to use for signing"))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .default(Expression::new(
                [(
                    "is_local_domain('*', sender_domain)",
                    "['rsa-' + sender_domain, 'ed25519-' + sender_domain]",
                )],
                "false",
            ))
            .new_field("report.dkim.from-name")
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of the DKIM ",
                "report email"
            ))
            .default("'Report Subsystem'")
            .new_field("report.dkim.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "the DKIM report email"
            ))
            .default("'noreply-dkim@' + key_get('default', 'domain')")
            .new_field("report.dkim.subject")
            .label("Subject")
            .help(concat!(
                "Subject name that will be used in the DKIM report email"
            ))
            .default("'DKIM Authentication Failure Report'")
            .new_field("report.dkim.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing the DKIM ",
                "report"
            ))
            .default(
                "['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]",
            )
            .new_field("report.dkim.send")
            .label("Send rate")
            .help(concat!(
                "Rate at which DKIM reports will be sent to a given email ",
                "address. When this rate is exceeded, no further DKIM failure reports",
                " will be sent to that address"
            ))
            .default("[1, 1d]")
            .build()
            .new_form_section()
            .title("DKIM Verification")
            .fields(["auth.dkim.verify"])
            .build()
            .new_form_section()
            .title("DKIM Signing")
            .fields(["auth.dkim.sign"])
            .build()
            .new_form_section()
            .title("DKIM Reporting")
            .fields([
                "report.dkim.from-name",
                "report.dkim.from-address",
                "report.dkim.subject",
                "report.dkim.sign",
                "report.dkim.send",
            ])
            .build()
            .build()
            // ARC Settings
            .new_schema("arc")
            .new_field("auth.arc.verify")
            .label("Strategy")
            .help(concat!(
                "Whether ARC verification is strict, relaxed or disabled"
            ))
            .default(Expression::new([], "relaxed"))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .new_field("auth.arc.seal")
            .default(Expression::new(
                [],
                "['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]",
            ))
            .label("Signature")
            .help(concat!("List of DKIM signatures to use for sealing"))
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(rcpt_vars)],
            )
            .build()
            .new_form_section()
            .title("ARC Verification")
            .fields(["auth.arc.verify"])
            .build()
            .new_form_section()
            .title("ARC Sealing")
            .fields(["auth.arc.seal"])
            .build()
            .build()
            // SPF Settings
            .new_schema("spf")
            .new_field("auth.spf.verify.ehlo")
            .label("EHLO")
            .help(concat!(
                "Whether SPF EHLO verification is strict, relaxed or disabled"
            ))
            .default(Expression::new(
                [("local_port == 25", "relaxed")],
                "disable",
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(conn_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .new_field("auth.spf.verify.mail-from")
            .label("MAIL FROM")
            .help(concat!(
                "Whether SPF MAIL FROM verification is strict, relaxed or disabled"
            ))
            .default(Expression::new(
                [("local_port == 25", "relaxed")],
                "disable",
            ))
            .new_field("report.spf.from-name")
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of the SPF authentication failure ",
                "report email"
            ))
            .default("'Report Subsystem'")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .new_field("report.spf.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "the SPF authentication failure report email"
            ))
            .default("'noreply-spf@' + key_get('default', 'domain')")
            .new_field("report.spf.subject")
            .label("Subject")
            .help(concat!(
                "Subject name that will be used in the SPF authentication failure report email"
            ))
            .default("'SPF Authentication Failure Report'")
            .new_field("report.spf.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing the SPF ",
                "authentication failure report"
            ))
            .default(
                "['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]",
            )
            .new_field("report.spf.send")
            .label("Send rate")
            .help(concat!(
                "Rate at which SPF reports will be sent to a given email ",
                "address. When this rate is exceeded, no further SPF failure reports",
                " will be sent to that address"
            ))
            .default("[1, 1d]")
            .build()
            .new_form_section()
            .title("SPF Verification")
            .fields(["auth.spf.verify.ehlo", "auth.spf.verify.mail-from"])
            .build()
            .new_form_section()
            .title("SPF Authentication Failure Reporting")
            .fields([
                "report.spf.from-name",
                "report.spf.from-address",
                "report.spf.subject",
                "report.spf.sign",
                "report.spf.send",
            ])
            .build()
            .build()
            // DMARC Settings
            .new_schema("dmarc")
            .new_field("auth.dmarc.verify")
            .label("Strategy")
            .help(concat!(
                "Whether DMARC verification is strict, relaxed or disabled"
            ))
            .default(Expression::new(
                [("local_port == 25", "relaxed")],
                "disable",
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(VERIFY_CONSTANTS)),
                ],
            )
            .new_field("report.dmarc.from-name")
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of the DMARC ",
                "report email"
            ))
            .default("'Report Subsystem'")
            .new_field("report.dmarc.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "the DMARC authentication failure report email"
            ))
            .default("'noreply-dmarc@' + key_get('default', 'domain')")
            .new_field("report.dmarc.subject")
            .label("Subject")
            .help(concat!(
                "Subject name that will be used in the DMARC authentication failure report email"
            ))
            .default("'DMARC Authentication Failure Report'")
            .new_field("report.dmarc.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing the DMARC ",
                "authentication failure report"
            ))
            .default(
                "['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]",
            )
            .new_field("report.dmarc.send")
            .label("Send rate")
            .help(concat!(
                "Rate at which DMARC reports will be sent to a given email ",
                "address. When this rate is exceeded, no further DMARC failure reports",
                " will be sent to that address"
            ))
            .default("[1, 1d]")
            .new_field("report.dmarc.aggregate.from-name")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_domain),
                ],
            )
            .label("From Name")
            .help(concat!(
                "Name that will be used in the From header of the DMARC ",
                "aggregate report email"
            ))
            .default("'Report Subsystem'")
            .new_field("report.dmarc.aggregate.from-address")
            .label("From Address")
            .help(concat!(
                "Email address that will be used in the From header of ",
                "the DMARC aggregate report email"
            ))
            .default("'noreply-dmarc@' + key_get('default', 'domain')")
            .new_field("report.dmarc.aggregate.subject")
            .label("Subject")
            .help(concat!(
                "Subject name that will be used in the DMARC aggregate report email"
            ))
            .default("'DMARC Aggregate Report'")
            .new_field("report.dmarc.aggregate.sign")
            .label("Signature")
            .help(concat!(
                "List of DKIM signatures to use when signing the DMARC ",
                "aggregate report"
            ))
            .default(
                "['rsa-' + key_get('default', 'domain'), 'ed25519-' + key_get('default', 'domain')]",
            )
            .new_field("report.dmarc.aggregate.org-name")
            .label("Organization")
            .help(concat!(
                "Name of the organization to be included in the report"
            ))
            .default("key_get('default', 'domain')")
            .new_field("report.dmarc.aggregate.contact-info")
            .label("Contact")
            .help(concat!("Contact information to be included in the report"))
            .default("")
            .new_field("report.dmarc.aggregate.max-size")
            .label("Max Report Size")
            .help(concat!(
                "Maximum size of the DMARC aggregate report in bytes"
            ))
            .default("26214400")
            .new_field("report.dmarc.aggregate.send")
            .label("Frequency")
            .help(concat!(
                "Frequency at which the DMARC aggregate reports will be sent. The options ",
                "are hourly, daily, weekly, or never to disable reporting"
            ))
            .default("daily")
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(rcpt_vars.constants(AGGREGATE_FREQ_CONSTANTS)),
                ],
            )
            .build()
            .new_form_section()
            .title("DMARC Verification")
            .fields(["auth.dmarc.verify"])
            .build()
            .new_form_section()
            .title("DMARC Authentication Failure Reporting")
            .fields([
                "report.dmarc.from-name",
                "report.dmarc.from-address",
                "report.dmarc.subject",
                "report.dmarc.sign",
                "report.dmarc.send",
            ])
            .build()
            .new_form_section()
            .title("DMARC Aggregate Reporting")
            .fields([
                "report.dmarc.aggregate.from-name",
                "report.dmarc.aggregate.from-address",
                "report.dmarc.aggregate.subject",
                "report.dmarc.aggregate.sign",
                "report.dmarc.aggregate.org-name",
                "report.dmarc.aggregate.contact-info",
                "report.dmarc.aggregate.max-size",
                "report.dmarc.aggregate.send",
            ])
            .build()
            .build()
            // Reporting
            .new_schema("report")
            .new_field("lookup.default.domain")
            .label("Default Domain")
            .help("The default domain name used for DSNs and other reports")
            .placeholder("example.com")
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsDomain])
            .build()
            .new_field("report.analysis.addresses")
            .label("Report Addresses")
            .help(concat!(
                "List of addresses (which may include wildcards) from which ",
                "reports will be intercepted and analyzed"
            ))
            .default(&["dmarc@*", "abuse@*", "postmaster@*"][..])
            .typ(Type::Array)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("report.analysis.forward")
            .label("Forward")
            .help(concat!(
                "Whether reports should be forwarded to their final recipient ",
                "after analysis"
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("report.analysis.store")
            .label("Store duration")
            .help(concat!(
                "The duration for which reports should be stored before being ",
                "deleted, of None to disable storage"
            ))
            .default("30d")
            .typ(Type::Duration)
            .build()
            .new_field("report.submitter")
            .label("Submitter")
            .help(concat!(
                "Report submitter address or leave empty to use the default hostname"
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::IsValidExpression(ExpressionValidator::new(
                    RCPT_DOMAIN_VARS,
                    &[],
                ))],
            )
            .default("key_get('default', 'hostname')")
            .build()
            .new_form_section()
            .title("Inbound Report Analysis")
            .fields([
                "report.analysis.addresses",
                "report.analysis.store",
                "report.analysis.forward",
            ])
            .build()
            .new_form_section()
            .title("Outbound Report Settings")
            .fields(["lookup.default.domain", "report.submitter"])
            .build()
            .build()
    }
}
