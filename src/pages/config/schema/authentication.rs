/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

impl Builder<Schemas, ()> {
    pub fn build_authentication(self) -> Self {
        // Authentication
        self.new_schema("authentication")
            .new_field("storage.directory")
            .label("Directory")
            .help("The directory to use for authentication and authorization")
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "directory",
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .input_check([], [Validator::Required])
            .build()
            // Fallback admin
            .new_field("authentication.fallback-admin.user")
            .label("Username")
            .help(concat!(
                "A rescue admin user can access the server in case the ",
                "directory becomes unavailable"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("authentication.fallback-admin.secret")
            .label("Password")
            .help(concat!(
                "A rescue admin secret that can access the server ",
                "in case the directory becomes unavailable"
            ))
            .typ(Type::Secret)
            .input_check([Transformer::Trim, Transformer::HashSecret], [])
            .build()
            // Master user
            .new_field("authentication.master.user")
            .label("Username")
            .help(concat!(
                "The master user can access any user account ",
                "using 'user-login%master-user' as the login name. ",
                "Leave blank to disable"
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("authentication.master.secret")
            .label("Password")
            .help(concat!(
                "The master user secret to access any user account ",
            ))
            .typ(Type::Secret)
            .input_check([Transformer::Trim, Transformer::HashSecret], [])
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["storage.directory"])
            .build()
            .new_form_section()
            .title("Fallback Administrator")
            .fields([
                "authentication.fallback-admin.user",
                "authentication.fallback-admin.secret",
            ])
            .build()
            .new_form_section()
            .title("Master User")
            .fields(["authentication.master.user", "authentication.master.secret"])
            .build()
            .build()
            // OAuth
            .new_schema("oauth")
            .new_field("oauth.key")
            .label("Key")
            .help("Encryption key to use for OAuth")
            .typ(Type::Secret)
            .input_check([], [Validator::Required])
            .build()
            .new_field("oauth.auth.max-attempts")
            .label("Max attempts")
            .help("Number of failed login attempts before an authorization code is invalidated")
            .typ(Type::Input)
            .default("3")
            .input_check([], [Validator::Required, Validator::MinValue(1.into())])
            .build()
            .new_field("oauth.expiry.user-code")
            .label("User code")
            .help("Expiration time of a user code issued by the device authentication flow")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("30m")
            .build()
            .new_field("oauth.expiry.auth-code")
            .label("Auth code")
            .help("Expiration time of an authorization code issued by the authorization code flow")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("10m")
            .build()
            .new_field("oauth.expiry.token")
            .label("Token")
            .help("Expiration time of an OAuth access token")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("1h")
            .build()
            .new_field("oauth.expiry.refresh-token")
            .label("Refresh token")
            .help("Expiration time of an OAuth refresh token")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("30d")
            .build()
            .new_field("oauth.expiry.refresh-token-renew")
            .label("Refresh token renew")
            .help("Remaining time in a refresh token before a new one is issued to the client")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .default("4d")
            .build()
            .new_field("oauth.client-registration.require")
            .label("Require client registration")
            .help("Whether to require OAuth client_ids to be registered before they can be used")
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_field("oauth.client-registration.anonymous")
            .label("Allow anonymous registration")
            .help("Whether to allow OAuth clients to register without authentication")
            .typ(Type::Boolean)
            .default("false")
            .build()
            .new_form_section()
            .title("OAuth Settings")
            .fields(["oauth.key"])
            .fields(["oauth.auth.max-attempts"])
            .build()
            .new_form_section()
            .title("Token Expiration")
            .fields([
                "oauth.expiry.user-code",
                "oauth.expiry.auth-code",
                "oauth.expiry.token",
                "oauth.expiry.refresh-token",
                "oauth.expiry.refresh-token-renew",
            ])
            .build()
            .new_form_section()
            .title("Dynamic Client Registration")
            .fields([
                "oauth.client-registration.require",
                "oauth.client-registration.anonymous",
            ])
            .build()
            .build()
            // OpenID
            .new_schema("openid")
            .new_field("oauth.oidc.signature-algorithm")
            .label("Signature algorithm")
            .help(concat!(
                "JWT signature algorithm to use for OpenID Connect."
            ))
            .default("relaxed/relaxed")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::StaticId(&[
                    "ES256", "ES384", "PS256", "PS384", "PS512", "RS256", "RS384", "RS512",
                    "HS256", "HS384", "HS512",
                ]),
            })
            .default("HS256")
            .input_check([], [Validator::Required])
            .build()
            .new_field("oauth.oidc.signature-key")
            .label("Signature Key")
            .help(concat!(
                "Contents of the private key PEM used to sign JWTs for OpenID Connect."
            ))
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("OpenID Connect")
            .fields(["oauth.oidc.signature-algorithm", "oauth.oidc.signature-key"])
            .build()
            .build()
    }
}
