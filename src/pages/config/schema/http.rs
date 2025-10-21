/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::core::schema::*;

use super::HTTP_VARS;

impl Builder<Schemas, ()> {
    pub fn build_http(self) -> Self {
        let http_expr = ExpressionValidator::new(HTTP_VARS, &[]);

        // HTTP settings
        self.new_schema("http-settings")
            // HTTP base URL
            .new_field("http.url")
            .label("Base URL")
            .help("The base URL for the HTTP server")
            .typ(Type::Expression)
            .input_check(
                [],
                [Validator::Required, Validator::IsValidExpression(http_expr)],
            )
            .default("protocol + '://' + config_get('server.hostname') + ':' + local_port")
            .build()
            // Use X-Forwarded-For
            .new_field("http.use-x-forwarded")
            .label("Obtain remote IP from Forwarded header")
            .help(concat!(
                "Specifies whether to use the Forwarded or X-Forwarded-For header to ",
                "determine the client's IP address"
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
            .new_field("http.headers")
            .label("Response headers")
            .help("Additional headers to include in HTTP responses")
            .typ(Type::Array(ArrayType::Text))
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("HTTP Base URL")
            .fields(["http.url"])
            .build()
            .new_form_section()
            .title("HTTP Headers")
            .fields(["http.headers", "http.use-x-forwarded"])
            .build()
            .new_form_section()
            .title("Web-based Admin")
            .fields(["webadmin.path", "webadmin.resource", "webadmin.auto-update"])
            .build()
            .build()
            // HTTP Security
            .new_schema("http-security")
            // HTTP endpoint security
            .new_field("http.allowed-endpoint")
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
            // Permissive CORS
            .new_field("http.permissive-cors")
            .label("Permissive CORS policy")
            .help(concat!(
                "Specifies whether to allow all origins in the CORS policy ",
                "for the HTTP server"
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            // HTTPS Strict Transport Security
            .new_field("http.hsts")
            .label("Enable HTTP Strict Transport Security")
            .help(concat!(
                "Specifies whether to enable HTTP Strict Transport Security ",
                "for the HTTP server."
            ))
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_form_section()
            .title("HTTP Security")
            .fields(["http.allowed-endpoint", "http.hsts", "http.permissive-cors"])
            .build()
            .build()
            // Rate limit
            .new_schema("http-rate-limit")
            .new_field("http.rate-limit.account")
            .label("Authenticated")
            .help("Specifies the request rate limit for authenticated users")
            .default("1000/1m")
            .typ(Type::Rate)
            .build()
            .new_field("http.rate-limit.anonymous")
            .label("Anonymous")
            .help("Specifies the request rate limit for unauthenticated users")
            .default("100/1m")
            .typ(Type::Rate)
            .build()
            .new_form_section()
            .title("Rate Limit")
            .fields(["http.rate-limit.account", "http.rate-limit.anonymous"])
            .build()
            .build()
            // Contact form settings
            .new_schema("http-form")
            .new_field("form.deliver-to")
            .label("Recipients")
            .help("List of local e-mail addresses to deliver the contact form to.")
            .typ(Type::Array(ArrayType::Text))
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
            .help("Maximum size of the contact form submission in bytes.")
            .typ(Type::Size)
            .default("102400")
            .build()
            .new_field("form.enable")
            .label("Enable form submissions")
            .help("Whether to enable contact form submissions.")
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("form.validate-domain")
            .label("Validate email domain")
            .help("Whether to validate the domain of the sender's email address.")
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
    }
}
