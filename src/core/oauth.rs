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

pub async fn oauth_authenticate(
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<(OAuthGrant, bool), Alert> {
    let response =
        oauth_user_authentication(base_url, username, password, "webadmin", None).await?;
    let is_admin = response.is_admin;
    match HttpRequest::post(format!("{base_url}/auth/token"))
        .with_raw_body(
            form_urlencoded::Serializer::new(String::with_capacity(response.code.len() + 64))
                .append_pair("grant_type", "authorization_code")
                .append_pair("client_id", "webadmin")
                .append_pair("code", &response.code)
                .append_pair("redirect_uri", "")
                .finish(),
        )
        .send_raw()
        .await
        .and_then(|response| {
            serde_json::from_slice::<OAuthResponse>(response.as_bytes()).map_err(Into::into)
        }) {
        Ok(OAuthResponse::Granted(grant)) => Ok((grant, is_admin)),
        Ok(OAuthResponse::Error { error }) => Err(Alert::error("OAuth failure")
            .with_details(format!("Server returned error code {error:?}"))),
        Err(err) => Err(Alert::from(err)),
    }
}

pub async fn oauth_user_authentication(
    base_url: &str,
    username: &str,
    password: &str,
    client_id: &str,
    redirect_uri: Option<&str>,
) -> Result<OAuthCodeResponse, Alert> {
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
        Ok(response) => Ok(response),
        Err(http::Error::Unauthorized) => {
            Err(Alert::warning("Incorrect username or password")
                .with_timeout(Duration::from_secs(3)))
        }
        Err(err) => Err(Alert::from(err)),
    }
}

pub async fn oauth_device_authentication(
    base_url: &str,
    username: &str,
    password: &str,
    code: &str,
) -> Alert {
    match HttpRequest::post(format!("{base_url}/api/oauth"))
        .with_basic_authorization(username, password)
        .with_body(OAuthCodeRequest::Device {
            code: code.to_string(),
        })
        .unwrap()
        .send::<bool>()
        .await
    {
        Ok(is_valid) => {
            if is_valid {
                Alert::success("Device authenticated")
                    .with_details("You have successfully authenticated your device")
                    .without_timeout()
            } else {
                Alert::warning("Device authentication failed")
                    .with_details("The code you entered is invalid or has expired")
            }
        }
        Err(http::Error::Unauthorized) => {
            Alert::warning("Incorrect username or password").with_timeout(Duration::from_secs(3))
        }
        Err(err) => Alert::from(err),
    }
}

pub async fn oauth_refresh_token(base_url: &str, refresh_token: &str) -> Option<OAuthGrant> {
    log::debug!("Refreshing OAuth token");

    match HttpRequest::post(format!("{base_url}/auth/token"))
        .with_raw_body(
            form_urlencoded::Serializer::new(String::with_capacity(refresh_token.len() + 64))
                .append_pair("grant_type", "refresh_token")
                .append_pair("refresh_token", refresh_token)
                .finish(),
        )
        .send_raw()
        .await
        .and_then(|response| {
            serde_json::from_slice::<OAuthResponse>(response.as_bytes()).map_err(Into::into)
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
