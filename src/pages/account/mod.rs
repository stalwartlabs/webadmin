/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use serde::{Deserialize, Serialize};

pub mod app_password;
pub mod crypto;
pub mod mfa;
pub mod password;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum AccountAuthRequest {
    SetPassword { password: String },
    EnableOtpAuth { url: String },
    DisableOtpAuth { url: Option<String> },
    AddAppPassword { name: String, password: String },
    RemoveAppPassword { name: Option<String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountAuthResponse {
    #[serde(rename = "otpEnabled")]
    pub otp_auth: bool,
    #[serde(rename = "appPasswords")]
    pub app_passwords: Vec<String>,
}
