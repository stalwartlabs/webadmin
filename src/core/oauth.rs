/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, time::Duration};

use leptos::{expect_context, RwSignal};
use serde::{Deserialize, Serialize};

use crate::components::messages::alert::Alert;

use super::http::{self, HttpRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    pub base_url: Arc<String>,
    pub access_token: Arc<String>,
    pub refresh_token: Arc<String>,
    pub username: Arc<String>,
    pub is_valid: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OAuthCodeRequest {
    Code {
        client_id: String,
        redirect_uri: Option<String>,
    },
    Device {
        code: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCodeResponse {
    pub code: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OAuthResponse {
    Granted(OAuthGrant),
    Error { error: ErrorType },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthGrant {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorType {
    #[serde(rename = "invalid_grant")]
    InvalidGrant,
    #[serde(rename = "invalid_client")]
    InvalidClient,
    #[serde(rename = "invalid_scope")]
    InvalidScope,
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "unauthorized_client")]
    UnauthorizedClient,
    #[serde(rename = "unsupported_grant_type")]
    UnsupportedGrantType,
    #[serde(rename = "authorization_pending")]
    AuthorizationPending,
    #[serde(rename = "slow_down")]
    SlowDown,
    #[serde(rename = "access_denied")]
    AccessDenied,
    #[serde(rename = "expired_token")]
    ExpiredToken,
}

pub enum AuthenticationResult<T> {
    Success(T),
    TotpRequired,
    Error(Alert),
}

pub struct AuthenticationResponse {
    pub grant: OAuthGrant,
    pub is_admin: bool,
}

pub async fn oauth_authenticate(
    base_url: &str,
    username: &str,
    password: &str,
) -> AuthenticationResult<AuthenticationResponse> {
    let response =
        match oauth_user_authentication(base_url, username, password, "webadmin", None).await {
            AuthenticationResult::Success(response) => response,
            AuthenticationResult::TotpRequired => return AuthenticationResult::TotpRequired,
            AuthenticationResult::Error(err) => return AuthenticationResult::Error(err),
        };
    let is_admin = response.is_admin;
    match HttpRequest::post(format!("{base_url}/auth/token"))
        .with_raw_body(
            serde_urlencoded::to_string([
                ("grant_type", "authorization_code"),
                ("client_id", "webadmin"),
                ("code", &response.code),
                ("redirect_uri", ""),
            ])
            .unwrap(),
        )
        .send_raw()
        .await
        .and_then(|response| {
            serde_json::from_slice::<OAuthResponse>(response.as_slice()).map_err(Into::into)
        }) {
        Ok(OAuthResponse::Granted(grant)) => {
            AuthenticationResult::Success(AuthenticationResponse { grant, is_admin })
        }
        Ok(OAuthResponse::Error { error }) => AuthenticationResult::Error(
            Alert::error("OAuth failure")
                .with_details(format!("Server returned error code {error:?}")),
        ),
        Err(err) => AuthenticationResult::Error(Alert::from(err)),
    }
}

pub async fn oauth_user_authentication(
    base_url: &str,
    username: &str,
    password: &str,
    client_id: &str,
    redirect_uri: Option<&str>,
) -> AuthenticationResult<OAuthCodeResponse> {
    match HttpRequest::post(format!("{base_url}/api/oauth"))
        .with_basic_authorization(username, password)
        .with_body(OAuthCodeRequest::Code {
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.map(ToOwned::to_owned),
        })
        .unwrap()
        .send::<OAuthCodeResponse>()
        .await
    {
        Ok(response) => AuthenticationResult::Success(response),
        Err(http::Error::Unauthorized) => AuthenticationResult::Error(
            Alert::warning("Incorrect username or password").with_timeout(Duration::from_secs(3)),
        ),
        Err(http::Error::Forbidden) => {
            // Password matched but TOTP required
            AuthenticationResult::TotpRequired
        }
        Err(err) => AuthenticationResult::Error(Alert::from(err)),
    }
}

pub async fn oauth_device_authentication(
    base_url: &str,
    username: &str,
    password: &str,
    code: &str,
) -> AuthenticationResult<bool> {
    match HttpRequest::post(format!("{base_url}/api/oauth"))
        .with_basic_authorization(username, password)
        .with_body(OAuthCodeRequest::Device {
            code: code.to_string(),
        })
        .unwrap()
        .send::<bool>()
        .await
    {
        Ok(is_valid) => AuthenticationResult::Success(is_valid),
        Err(http::Error::Unauthorized) => AuthenticationResult::Error(
            Alert::warning("Incorrect username or password").with_timeout(Duration::from_secs(3)),
        ),
        Err(http::Error::Forbidden) => AuthenticationResult::TotpRequired,
        Err(err) => AuthenticationResult::Error(Alert::from(err)),
    }
}

pub async fn oauth_refresh_token(base_url: &str, refresh_token: &str) -> Option<OAuthGrant> {
    log::debug!("Refreshing OAuth token");

    match HttpRequest::post(format!("{base_url}/auth/token"))
        .with_raw_body(
            serde_urlencoded::to_string([
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .unwrap(),
        )
        .send_raw()
        .await
        .and_then(|response| {
            serde_json::from_slice::<OAuthResponse>(response.as_slice()).map_err(Into::into)
        }) {
        Ok(OAuthResponse::Granted(grant)) => Some(grant),
        Ok(OAuthResponse::Error { error }) => {
            log::error!("OAuth failure: Server returned error code {error:?}");
            None
        }
        Err(err) => {
            log::error!("OAuth failure: {err:?}");
            None
        }
    }
}

pub fn use_authorization() -> RwSignal<AuthToken> {
    expect_context::<RwSignal<AuthToken>>()
}

impl AuthToken {
    pub fn is_logged_in(&self) -> bool {
        !self.access_token.is_empty()
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin && self.is_logged_in()
    }
}

impl AsRef<AuthToken> for AuthToken {
    fn as_ref(&self) -> &AuthToken {
        self
    }
}
