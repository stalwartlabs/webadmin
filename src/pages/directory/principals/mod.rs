/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod edit;
pub mod list;

use base64::{engine::general_purpose::STANDARD, Engine};

pub fn parse_app_password(secret: &str) -> Option<(String, &str)> {
    secret
        .strip_prefix("$app$")
        .and_then(|s| s.split_once('$'))
        .and_then(|(app, password)| {
            STANDARD
                .decode(app)
                .ok()
                .and_then(|app| String::from_utf8(app).ok())
                .map(|app| (app, password))
        })
}

pub fn build_app_password(app: &str, password: &str) -> String {
    format!("$app${}${}", STANDARD.encode(app), password)
}

pub trait SpecialSecrets {
    fn is_disabled(&self) -> bool;
    fn is_otp_auth(&self) -> bool;
    fn is_app_password(&self) -> bool;
    fn is_password(&self) -> bool;
}

impl<T> SpecialSecrets for T
where
    T: AsRef<str>,
{
    fn is_disabled(&self) -> bool {
        self.as_ref() == "$disabled$"
    }

    fn is_otp_auth(&self) -> bool {
        self.as_ref().starts_with("otpauth://")
    }

    fn is_app_password(&self) -> bool {
        self.as_ref().starts_with("$app$")
    }

    fn is_password(&self) -> bool {
        !self.is_disabled() && !self.is_otp_auth() && !self.is_app_password()
    }
}
