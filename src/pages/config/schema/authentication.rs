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
                multi: false,
            })
            .input_check([], [Validator::Required])
            .build()
            .new_field("authentication.fail2ban")
            .label("Ban rate")
            .help("The maximum number of failed login attempts before the IP is banned")
            .typ(Type::Rate)
            .default("100/1d")
            .build()
            .new_field("authentication.rate-limit")
            .label("Limit rate")
            .help(concat!(
                "Amount of authentication requests that can be made in a ",
                "timeframe by a given IP address"
            ))
            .typ(Type::Rate)
            .default("10/1m")
            .build()
            .new_form_section()
            .title("Authentication")
            .fields(["storage.directory"])
            .build()
            .new_form_section()
            .title("Security")
            .fields(["authentication.rate-limit", "authentication.fail2ban"])
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
            .new_form_section()
            .title("OAuth Settings")
            .fields(["oauth.key"])
            .fields(["oauth.auth.max-attempts"])
            .build()
            .new_form_section()
            .title("Expiry")
            .fields([
                "oauth.expiry.user-code",
                "oauth.expiry.auth-code",
                "oauth.expiry.token",
                "oauth.expiry.refresh-token",
                "oauth.expiry.refresh-token-renew",
            ])
            .build()
            .build()
    }
}
