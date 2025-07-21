/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{fmt::Display, sync::Arc};

use ahash::AHashSet;
use serde::{Deserialize, Serialize};

pub mod expr;
pub mod form;
pub mod http;
pub mod oauth;
pub mod schema;
pub mod url;

pub const MINIMUM_API_VERSION: Semver = Semver::new(0, 13, 0);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessToken {
    pub base_url: Arc<String>,
    pub access_token: Arc<String>,
    pub refresh_token: Arc<String>,
    pub username: Arc<String>,
    pub is_valid: bool,
    pub is_enterprise: bool,
    pub permissions: Permissions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions(Arc<AHashSet<Permission>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    // Admin
    Impersonate,
    UnlimitedRequests,
    UnlimitedUploads,
    DeleteSystemFolders,
    MessageQueueList,
    MessageQueueGet,
    MessageQueueUpdate,
    MessageQueueDelete,
    OutgoingReportList,
    OutgoingReportGet,
    OutgoingReportDelete,
    IncomingReportList,
    IncomingReportGet,
    IncomingReportDelete,
    SettingsList,
    SettingsUpdate,
    SettingsDelete,
    SettingsReload,
    IndividualList,
    IndividualGet,
    IndividualUpdate,
    IndividualDelete,
    IndividualCreate,
    GroupList,
    GroupGet,
    GroupUpdate,
    GroupDelete,
    GroupCreate,
    DomainList,
    DomainGet,
    DomainCreate,
    DomainUpdate,
    DomainDelete,
    TenantList,
    TenantGet,
    TenantCreate,
    TenantUpdate,
    TenantDelete,
    MailingListList,
    MailingListGet,
    MailingListCreate,
    MailingListUpdate,
    MailingListDelete,
    RoleList,
    RoleGet,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
    PrincipalList,
    PrincipalGet,
    PrincipalCreate,
    PrincipalUpdate,
    PrincipalDelete,
    BlobFetch,
    PurgeBlobStore,
    PurgeDataStore,
    PurgeInMemoryStore,
    PurgeAccount,
    FtsReindex,
    Undelete,
    DkimSignatureCreate,
    DkimSignatureGet,
    SpamFilterUpdate,
    WebadminUpdate,
    LogsView,
    SpamFilterTrain,
    Restart,
    TracingList,
    TracingGet,
    TracingLive,
    MetricsList,
    MetricsLive,
    Troubleshoot,

    // Account Management
    ManageEncryption,
    ManagePasswords,

    // API keys
    ApiKeyList,
    ApiKeyGet,
    ApiKeyCreate,
    ApiKeyUpdate,
    ApiKeyDelete,

    // OAuth clients
    OauthClientList,
    OauthClientGet,
    OauthClientCreate,
    OauthClientUpdate,
    OauthClientDelete,

    #[serde(other)]
    Unknown,
}

impl AccessToken {
    pub fn is_logged_in(&self) -> bool {
        !self.access_token.is_empty()
    }

    pub fn permissions(&self) -> &Permissions {
        &self.permissions
    }

    pub fn is_enterprise(&self) -> bool {
        self.is_enterprise
    }

    pub fn default_url(&self) -> &'static str {
        self.permissions.default_url(self.is_enterprise)
    }
}

impl Permissions {
    pub fn new(permissions: AHashSet<Permission>) -> Self {
        Self(Arc::new(permissions))
    }

    pub fn has_access_all(&self, permission: &[Permission]) -> bool {
        permission.iter().all(|p| self.0.contains(p))
    }

    pub fn has_access_any(&self, permission: &[Permission]) -> bool {
        permission.iter().any(|p| self.0.contains(p))
    }

    pub fn has_admin_access(&self) -> bool {
        self.0.iter().any(Permission::is_admin_permission)
    }

    pub fn has_access(&self, permission: Permission) -> bool {
        self.0.contains(&permission)
    }

    pub fn default_url(&self, is_enterprise: bool) -> &'static str {
        if is_enterprise
            && self.0.contains(&Permission::MetricsList)
            && self.0.contains(&Permission::MetricsLive)
        {
            "/manage/dashboard/overview"
        } else {
            for permission in [
                Permission::IndividualList,
                Permission::GroupList,
                Permission::DomainList,
                Permission::TenantList,
                Permission::MailingListList,
                Permission::RoleList,
                Permission::MessageQueueList,
                Permission::OutgoingReportList,
                Permission::IncomingReportList,
                Permission::LogsView,
                Permission::TracingList,
                Permission::TracingLive,
                Permission::ManageEncryption,
                Permission::ManagePasswords,
                Permission::SpamFilterTrain,
            ]
            .iter()
            {
                if self.0.contains(permission) {
                    return match permission {
                        Permission::IndividualList => "/manage/directory/accounts",
                        Permission::GroupList => "/manage/directory/groups",
                        Permission::DomainList => "/manage/directory/domains",
                        Permission::TenantList => "/manage/directory/tenants",
                        Permission::MailingListList => "/manage/directory/lists",
                        Permission::RoleList => "/manage/directory/roles",
                        Permission::MessageQueueList => "/manage/queue/messages",
                        Permission::OutgoingReportList => "/manage/queue/reports",
                        Permission::IncomingReportList => "/manage/reports/dmarc",
                        Permission::ManageEncryption => "/account/crypto",
                        Permission::ManagePasswords => "/account/password",
                        Permission::SpamFilterTrain => "/manage/spam/train",
                        Permission::LogsView => "/manage/logs",
                        Permission::TracingList => "/manage/tracing/received",
                        Permission::TracingLive => "/manage/tracing/live",
                        _ => unreachable!(),
                    };
                }
            }

            ""
        }
    }
}

impl AsRef<AccessToken> for AccessToken {
    fn as_ref(&self) -> &AccessToken {
        self
    }
}

impl Permission {
    pub fn is_admin_permission(&self) -> bool {
        matches!(
            self,
            Permission::Impersonate
                | Permission::UnlimitedRequests
                | Permission::UnlimitedUploads
                | Permission::DeleteSystemFolders
                | Permission::MessageQueueList
                | Permission::MessageQueueGet
                | Permission::MessageQueueUpdate
                | Permission::MessageQueueDelete
                | Permission::OutgoingReportList
                | Permission::OutgoingReportGet
                | Permission::OutgoingReportDelete
                | Permission::IncomingReportList
                | Permission::IncomingReportGet
                | Permission::IncomingReportDelete
                | Permission::SettingsList
                | Permission::SettingsUpdate
                | Permission::SettingsDelete
                | Permission::SettingsReload
                | Permission::IndividualList
                | Permission::IndividualGet
                | Permission::IndividualUpdate
                | Permission::IndividualDelete
                | Permission::IndividualCreate
                | Permission::GroupList
                | Permission::GroupGet
                | Permission::GroupUpdate
                | Permission::GroupDelete
                | Permission::GroupCreate
                | Permission::DomainList
                | Permission::DomainGet
                | Permission::DomainCreate
                | Permission::DomainUpdate
                | Permission::DomainDelete
                | Permission::TenantList
                | Permission::TenantGet
                | Permission::TenantCreate
                | Permission::TenantUpdate
                | Permission::TenantDelete
                | Permission::MailingListList
                | Permission::MailingListGet
                | Permission::MailingListCreate
                | Permission::MailingListUpdate
                | Permission::MailingListDelete
                | Permission::RoleList
                | Permission::RoleGet
                | Permission::RoleCreate
                | Permission::RoleUpdate
                | Permission::RoleDelete
                | Permission::PrincipalList
                | Permission::PrincipalGet
                | Permission::PrincipalCreate
                | Permission::PrincipalUpdate
                | Permission::PrincipalDelete
                | Permission::BlobFetch
                | Permission::PurgeBlobStore
                | Permission::PurgeDataStore
                | Permission::PurgeInMemoryStore
                | Permission::PurgeAccount
                | Permission::Undelete
                | Permission::DkimSignatureCreate
                | Permission::DkimSignatureGet
                | Permission::SpamFilterUpdate
                | Permission::WebadminUpdate
                | Permission::LogsView
                | Permission::Restart
                | Permission::TracingList
                | Permission::TracingGet
                | Permission::TracingLive
                | Permission::MetricsList
                | Permission::MetricsLive
                | Permission::Troubleshoot
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Semver(u64);

impl Semver {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        let mut version: u64 = 0;
        version |= (major as u64) << 32;
        version |= (minor as u64) << 16;
        version |= patch as u64;
        Semver(version)
    }

    pub fn unpack(&self) -> (u16, u16, u16) {
        let version = self.0;
        let major = ((version >> 32) & 0xFFFF) as u16;
        let minor = ((version >> 16) & 0xFFFF) as u16;
        let patch = (version & 0xFFFF) as u16;
        (major, minor, patch)
    }

    pub fn major(&self) -> u16 {
        (self.0 >> 32) as u16
    }

    pub fn minor(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub fn patch(&self) -> u16 {
        self.0 as u16
    }

    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl AsRef<u64> for Semver {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl From<u64> for Semver {
    fn from(value: u64) -> Self {
        Semver(value)
    }
}

impl TryFrom<&str> for Semver {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.splitn(3, '.');
        let major = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let minor = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let patch = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        Ok(Semver::new(major, minor, patch))
    }
}

impl Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (major, minor, patch) = self.unpack();
        write!(f, "{major}.{minor}.{patch}")
    }
}
