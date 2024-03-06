use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine};
use leptos::{expect_context, RwSignal};
use serde::{Deserialize, Serialize};

use crate::components::messages::alert::Alert;

use super::http::{self, HttpRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    pub base_url: Arc<String>,
    pub access_token: Arc<String>,
    pub refresh_token: Arc<String>,
    pub is_valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCodeRequest {
    pub client_id: String,
    pub redirect_uri: Option<String>,
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
    user: &str,
    password: &str,
) -> Result<OAuthGrant, Alert> {
    match HttpRequest::post(format!("{base_url}/api/oauth/code"))
        .with_header(
            "Authorization",
            format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{}", user, password).as_bytes())
            ),
        )
        .with_body(OAuthCodeRequest {
            client_id: "webadmin".to_string(),
            redirect_uri: None,
        })
        .unwrap()
        .send::<String>()
        .await
    {
        Ok(code) => {
            match HttpRequest::post(format!("{base_url}/auth/token"))
                .with_raw_body(
                    form_urlencoded::Serializer::new(String::with_capacity(code.len() + 64))
                        .append_pair("grant_type", "authorization_code")
                        .append_pair("client_id", "webadmin")
                        .append_pair("code", &code)
                        .append_pair("redirect_uri", "")
                        .finish(),
                )
                .send_raw()
                .await
                .and_then(|response| {
                    serde_json::from_slice::<OAuthResponse>(response.as_bytes()).map_err(Into::into)
                }) {
                Ok(OAuthResponse::Granted(grant)) => Ok(grant),
                Ok(OAuthResponse::Error { error }) => Err(Alert::error("OAuth failure")
                    .with_details(format!("Server returned error code {error:?}"))),
                Err(err) => Err(Alert::from(err)),
            }
        }
        Err(http::Error::Unauthorized) => {
            Err(Alert::warning("Incorrect username or password")
                .with_timeout(Duration::from_secs(3)))
        }
        Err(err) => Err(Alert::from(err)),
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
}

impl AsRef<AuthToken> for AuthToken {
    fn as_ref(&self) -> &AuthToken {
        self
    }
}
