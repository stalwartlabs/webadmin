/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Duration;

use ahash::AHashSet;
use leptos::{expect_context, RwSignal};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::components::messages::alert::Alert;

use super::{
    http::{self, HttpRequest},
    AccessToken, Permission,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OAuthCodeRequest {
    Code {
        client_id: String,
        redirect_uri: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    Device {
        code: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCodeResponse {
    pub code: String,
    #[serde(default)]
    pub permissions: AHashSet<Permission>,

    #[serde(default)]
    #[serde(rename = "isEnterprise")]
    pub is_enterprise: bool,

    // Deprecated - remove in future
    #[serde(default)]
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
    pub permissions: AHashSet<Permission>,
    pub is_enterprise: bool,
}

const REDIRECT_URI: &str = "stalwart://auth";

pub async fn oauth_authenticate(
    base_url: &str,
    username: &str,
    password: &str,
) -> AuthenticationResult<AuthenticationResponse> {
    let response = match oauth_user_authentication(
        base_url,
        username,
        password,
        &OAuthCodeRequest::Code {
            client_id: "webadmin".to_string(),
            redirect_uri: REDIRECT_URI.to_string().into(),
            nonce: thread_rng()
                .sample_iter(Alphanumeric)
                .take(10)
                .map(char::from)
                .collect::<String>()
                .into(),
        },
    )
    .await
    {
        AuthenticationResult::Success(response) => response,
        AuthenticationResult::TotpRequired => return AuthenticationResult::TotpRequired,
        AuthenticationResult::Error(err) => return AuthenticationResult::Error(err),
    };
    let permissions = response.permissions;
    let is_enterprise = response.is_enterprise;
    match HttpRequest::post(format!("{base_url}/auth/token"))
        .with_raw_body(
            serde_urlencoded::to_string([
                ("grant_type", "authorization_code"),
                ("client_id", "webadmin"),
                ("code", &response.code),
                ("redirect_uri", REDIRECT_URI),
            ])
            .unwrap(),
        )
        .send_raw()
        .await
        .and_then(|response| {
            serde_json::from_slice::<OAuthResponse>(response.as_slice()).map_err(Into::into)
        }) {
        Ok(OAuthResponse::Granted(grant)) => {
            AuthenticationResult::Success(AuthenticationResponse {
                grant,
                permissions,
                is_enterprise,
            })
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
    request: &OAuthCodeRequest,
) -> AuthenticationResult<OAuthCodeResponse> {
    match HttpRequest::post(format!("{base_url}/api/oauth"))
        .with_basic_authorization(username, password)
        .with_body(request)
        .unwrap()
        .send::<OAuthCodeResponse>()
        .await
    {
        Ok(response) => AuthenticationResult::Success(response.legacy_admin()),
        Err(http::Error::Unauthorized) => AuthenticationResult::Error(
            Alert::warning("Incorrect username or password").with_timeout(Duration::from_secs(3)),
        ),
        Err(http::Error::Forbidden) => AuthenticationResult::Error(Alert::error(
            "You are not authorized to access this service.",
        )),
        Err(http::Error::TotpRequired) => {
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
    request: &OAuthCodeRequest,
) -> AuthenticationResult<bool> {
    match HttpRequest::post(format!("{base_url}/api/oauth"))
        .with_basic_authorization(username, password)
        .with_body(request)
        .unwrap()
        .send::<bool>()
        .await
    {
        Ok(is_valid) => AuthenticationResult::Success(is_valid),
        Err(http::Error::Unauthorized) => AuthenticationResult::Error(
            Alert::warning("Incorrect username or password").with_timeout(Duration::from_secs(3)),
        ),
        Err(http::Error::TotpRequired) => AuthenticationResult::TotpRequired,
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

pub fn use_authorization() -> RwSignal<AccessToken> {
    expect_context::<RwSignal<AccessToken>>()
}

impl OAuthCodeResponse {
    pub fn legacy_admin(mut self) -> Self {
        if self.is_admin && self.permissions.is_empty() {
            for permission in [
                Permission::Impersonate,
                Permission::UnlimitedRequests,
                Permission::UnlimitedUploads,
                Permission::DeleteSystemFolders,
                Permission::MessageQueueList,
                Permission::MessageQueueGet,
                Permission::MessageQueueUpdate,
                Permission::MessageQueueDelete,
                Permission::OutgoingReportList,
                Permission::OutgoingReportGet,
                Permission::OutgoingReportDelete,
                Permission::IncomingReportList,
                Permission::IncomingReportGet,
                Permission::IncomingReportDelete,
                Permission::SettingsList,
                Permission::SettingsUpdate,
                Permission::SettingsDelete,
                Permission::SettingsReload,
                Permission::IndividualList,
                Permission::IndividualGet,
                Permission::IndividualUpdate,
                Permission::IndividualDelete,
                Permission::IndividualCreate,
                Permission::GroupList,
                Permission::GroupGet,
                Permission::GroupUpdate,
                Permission::GroupDelete,
                Permission::GroupCreate,
                Permission::DomainList,
                Permission::DomainGet,
                Permission::DomainCreate,
                Permission::DomainUpdate,
                Permission::DomainDelete,
                Permission::TenantList,
                Permission::TenantGet,
                Permission::TenantCreate,
                Permission::TenantUpdate,
                Permission::TenantDelete,
                Permission::MailingListList,
                Permission::MailingListGet,
                Permission::MailingListCreate,
                Permission::MailingListUpdate,
                Permission::MailingListDelete,
                Permission::RoleList,
                Permission::RoleGet,
                Permission::RoleCreate,
                Permission::RoleUpdate,
                Permission::RoleDelete,
                Permission::PrincipalList,
                Permission::PrincipalGet,
                Permission::PrincipalCreate,
                Permission::PrincipalUpdate,
                Permission::PrincipalDelete,
                Permission::BlobFetch,
                Permission::PurgeBlobStore,
                Permission::PurgeDataStore,
                Permission::PurgeInMemoryStore,
                Permission::PurgeAccount,
                Permission::FtsReindex,
                Permission::Undelete,
                Permission::DkimSignatureCreate,
                Permission::DkimSignatureGet,
                Permission::SpamFilterUpdate,
                Permission::WebadminUpdate,
                Permission::LogsView,
                Permission::SpamFilterTrain,
                Permission::Restart,
                Permission::TracingList,
                Permission::TracingGet,
                Permission::TracingLive,
                Permission::MetricsList,
                Permission::MetricsLive,
                Permission::ManageEncryption,
                Permission::ManagePasswords,
                Permission::Troubleshoot,
            ] {
                self.permissions.insert(permission);
            }
        }

        self
    }
}
