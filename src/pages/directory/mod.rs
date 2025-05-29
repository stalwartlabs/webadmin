/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{fmt, str::FromStr};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use base64::{engine::general_purpose::STANDARD, Engine};

pub mod dns;
pub mod edit;
pub mod list;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typ: Option<PrincipalType>,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub quota: PrincipalValue,

    #[serde(rename = "usedQuota")]
    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub used_quota: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub name: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub description: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub tenant: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub picture: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub secrets: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub emails: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    pub urls: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "memberOf")]
    pub member_of: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "roles")]
    pub roles: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "lists")]
    pub lists: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "members")]
    pub members: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "enabledPermissions")]
    pub enabled_permissions: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "disabledPermissions")]
    pub disabled_permissions: PrincipalValue,

    #[serde(default, skip_serializing_if = "PrincipalValue::is_none")]
    #[serde(rename = "externalMembers")]
    pub external_members: PrincipalValue,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrincipalType {
    #[default]
    Individual = 0,
    Group = 1,
    Resource = 2,
    Location = 3,
    List = 5,
    Other = 6,
    Domain = 7,
    Tenant = 8,
    Role = 9,
    ApiKey = 10,
    OauthClient = 11,
}

pub const MAX_TYPE_ID: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrincipalField {
    Name,
    Type,
    Quota,
    UsedQuota,
    Description,
    Secrets,
    Emails,
    MemberOf,
    Members,
    Tenant,
    Roles,
    Lists,
    EnabledPermissions,
    DisabledPermissions,
    Picture,
    Urls,
    ExternalMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrincipalUpdate {
    action: PrincipalAction,
    field: PrincipalField,
    value: PrincipalValue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrincipalAction {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "addItem")]
    AddItem,
    #[serde(rename = "removeItem")]
    RemoveItem,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(untagged)]
pub enum PrincipalValue {
    String(String),
    StringList(Vec<String>),
    Integer(u64),
    IntegerList(Vec<u64>),
}

impl Principal {
    pub fn is_blank(&self) -> bool {
        self.id.is_none()
            && self.typ.is_none()
            && self.name.is_none()
            && self.emails.is_none()
            && self.member_of.is_none()
            && self.members.is_none()
            && self.description.is_none()
    }

    pub fn into_updates(self, changes: Principal) -> Vec<PrincipalUpdate> {
        let current = self;
        let mut updates = vec![];

        for (current, change, field) in [
            (current.name, changes.name, PrincipalField::Name),
            (
                current.description,
                changes.description,
                PrincipalField::Description,
            ),
            (current.tenant, changes.tenant, PrincipalField::Tenant),
            (current.picture, changes.picture, PrincipalField::Picture),
        ] {
            let current = current.unwrap_string();
            let change = change.unwrap_string();

            if current != change {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field,
                    value: PrincipalValue::String(change),
                });
            }
        }

        if current.quota != changes.quota
            && matches!(
                current.typ,
                Some(PrincipalType::Individual | PrincipalType::Group | PrincipalType::Tenant)
            )
        {
            updates.push(PrincipalUpdate {
                action: PrincipalAction::Set,
                field: PrincipalField::Quota,
                value: changes.quota,
            });
        }

        let mut changed_password = false;
        let current_secrets = current.secrets.unwrap_string_list();
        for new_secret in changes.secrets.as_string_list() {
            if !new_secret.is_app_password() && !current_secrets.contains(new_secret) {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::AddItem,
                    field: PrincipalField::Secrets,
                    value: PrincipalValue::String(new_secret.clone()),
                });

                if new_secret.is_password() {
                    changed_password = true;
                }
            }
        }

        for prev_secret in current_secrets {
            if !prev_secret.is_password() {
                let mut found = false;

                for new_secret in changes.secrets.as_string_list() {
                    if prev_secret.starts_with(new_secret) {
                        found = true;
                        break;
                    }
                }

                if !found {
                    updates.push(PrincipalUpdate {
                        action: PrincipalAction::RemoveItem,
                        field: PrincipalField::Secrets,
                        value: PrincipalValue::String(prev_secret),
                    });
                }
            } else if changed_password {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::RemoveItem,
                    field: PrincipalField::Secrets,
                    value: PrincipalValue::String(prev_secret),
                });
            }
        }

        for (field, current, change) in [
            (PrincipalField::Emails, current.emails, changes.emails),
            (
                PrincipalField::ExternalMembers,
                current.external_members,
                changes.external_members,
            ),
            (
                PrincipalField::MemberOf,
                current.member_of,
                changes.member_of,
            ),
            (PrincipalField::Members, current.members, changes.members),
            (PrincipalField::Roles, current.roles, changes.roles),
            (PrincipalField::Lists, current.lists, changes.lists),
            (
                PrincipalField::EnabledPermissions,
                current.enabled_permissions,
                changes.enabled_permissions,
            ),
            (
                PrincipalField::DisabledPermissions,
                current.disabled_permissions,
                changes.disabled_permissions,
            ),
            (PrincipalField::Urls, current.urls, changes.urls),
        ] {
            let current = current.unwrap_string_list();
            let change = change.unwrap_string_list();

            for item in &change {
                if !current.contains(item) {
                    updates.push(PrincipalUpdate {
                        action: PrincipalAction::AddItem,
                        field,
                        value: PrincipalValue::String(item.clone()),
                    });
                }
            }

            for item in current {
                if !change.contains(&item) {
                    updates.push(PrincipalUpdate {
                        action: PrincipalAction::RemoveItem,
                        field,
                        value: PrincipalValue::String(item),
                    });
                }
            }
        }

        updates
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_str()
    }

    pub fn name_or_empty(&self) -> String {
        self.name.as_str().unwrap_or_default().to_string()
    }

    pub fn email(&self) -> Option<&str> {
        self.emails.as_str()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_str()
    }

    pub fn description_or_name(&self) -> Option<&str> {
        self.description.as_str().or(self.name.as_str())
    }
}

impl PrincipalValue {
    pub fn is_none(&self) -> bool {
        matches!(self, PrincipalValue::String(s) if s.is_empty())
    }

    pub fn unwrap_string(self) -> String {
        match self {
            PrincipalValue::String(s) => s,
            PrincipalValue::StringList(l) => l.into_iter().next().unwrap_or_default(),
            _ => String::new(),
        }
    }
    pub fn try_unwrap_string(self) -> Option<String> {
        match self {
            PrincipalValue::String(s) if !s.is_empty() => Some(s),
            PrincipalValue::StringList(l) => l.into_iter().next(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PrincipalValue::String(s) if !s.is_empty() => Some(s),
            PrincipalValue::StringList(l) => l.first().map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn unwrap_string_list(self) -> Vec<String> {
        match self {
            PrincipalValue::String(s) if !s.is_empty() => vec![s],
            PrincipalValue::StringList(l) => l,
            _ => vec![],
        }
    }

    pub fn as_string_list(&self) -> &[String] {
        match self {
            PrincipalValue::String(s) if !s.is_empty() => std::slice::from_ref(s),
            PrincipalValue::StringList(l) => l,
            _ => &[],
        }
    }

    pub fn as_int(&self) -> Option<u64> {
        match self {
            PrincipalValue::Integer(i) => Some(*i),
            PrincipalValue::IntegerList(l) => l.first().copied(),
            _ => None,
        }
    }

    pub fn as_int_non_zero(&self) -> Option<u64> {
        self.as_int().filter(|&i| i > 0)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            PrincipalValue::String(s) if !s.is_empty() => 1,
            PrincipalValue::StringList(l) => l.len(),
            _ => 0,
        }
    }

    pub fn count(&self) -> usize {
        match self {
            PrincipalValue::Integer(v) => *v as usize,
            PrincipalValue::IntegerList(l) => l.len(),
            PrincipalValue::String(s) => {
                if s.is_empty() {
                    0
                } else {
                    1
                }
            }
            PrincipalValue::StringList(l) => l.len(),
        }
    }
}

impl Default for PrincipalValue {
    fn default() -> Self {
        PrincipalValue::String(String::new())
    }
}

impl PrincipalType {
    pub const fn id(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "individual",
            PrincipalType::Group => "group",
            PrincipalType::Resource => "resource",
            PrincipalType::Location => "location",
            PrincipalType::List => "list",
            PrincipalType::Other => "other",
            PrincipalType::Domain => "domain",
            PrincipalType::Tenant => "tenant",
            PrincipalType::Role => "role",
            PrincipalType::ApiKey => "apiKey",
            PrincipalType::OauthClient => "oauthClient",
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "Individual",
            PrincipalType::Group => "Group",
            PrincipalType::Resource => "Resource",
            PrincipalType::Location => "Location",
            PrincipalType::List => "Mailing List",
            PrincipalType::Other => "Other",
            PrincipalType::Domain => "Domain",
            PrincipalType::Tenant => "Tenant",
            PrincipalType::Role => "Role",
            PrincipalType::ApiKey => "API Key",
            PrincipalType::OauthClient => "OAuth Client",
        }
    }

    pub const fn item_name(&self, plural: bool) -> &'static str {
        match (self, plural) {
            (PrincipalType::Individual, false) => "account",
            (PrincipalType::Individual, true) => "accounts",
            (PrincipalType::Group, false) => "group",
            (PrincipalType::Group, true) => "groups",
            (PrincipalType::Resource, false) => "resource",
            (PrincipalType::Resource, true) => "resources",
            (PrincipalType::Location, false) => "location",
            (PrincipalType::Location, true) => "locations",
            (PrincipalType::List, false) => "mailing list",
            (PrincipalType::List, true) => "mailing lists",
            (PrincipalType::Other, false) => "other",
            (PrincipalType::Other, true) => "other",
            (PrincipalType::Domain, false) => "domain",
            (PrincipalType::Domain, true) => "domains",
            (PrincipalType::Tenant, false) => "tenant",
            (PrincipalType::Tenant, true) => "tenants",
            (PrincipalType::Role, false) => "role",
            (PrincipalType::Role, true) => "roles",
            (PrincipalType::ApiKey, false) => "API key",
            (PrincipalType::ApiKey, true) => "API keys",
            (PrincipalType::OauthClient, false) => "OAuth client",
            (PrincipalType::OauthClient, true) => "OAuth clients",
        }
    }

    pub fn resource_name(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "accounts",
            PrincipalType::Group => "groups",
            PrincipalType::List => "lists",
            PrincipalType::Role => "roles",
            PrincipalType::Tenant => "tenants",
            PrincipalType::Domain => "domains",
            PrincipalType::ApiKey => "api-keys",
            PrincipalType::OauthClient => "oauth-clients",
            _ => unimplemented!("resource_name for {:?}", self),
        }
    }
}

impl FromStr for PrincipalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "individual" => Ok(PrincipalType::Individual),
            "group" => Ok(PrincipalType::Group),
            "resource" => Ok(PrincipalType::Resource),
            "location" => Ok(PrincipalType::Location),
            "list" => Ok(PrincipalType::List),
            "other" => Ok(PrincipalType::Other),
            "domain" => Ok(PrincipalType::Domain),
            "tenant" => Ok(PrincipalType::Tenant),
            "role" => Ok(PrincipalType::Role),
            "apiKey" => Ok(PrincipalType::ApiKey),
            "oauthClient" => Ok(PrincipalType::OauthClient),
            _ => Err(format!("Invalid PrincipalType: {}", s)),
        }
    }
}

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
    fn is_otp_auth(&self) -> bool;
    fn is_app_password(&self) -> bool;
    fn is_password(&self) -> bool;
}

impl<T> SpecialSecrets for T
where
    T: AsRef<str>,
{
    fn is_otp_auth(&self) -> bool {
        self.as_ref().starts_with("otpauth://")
    }

    fn is_app_password(&self) -> bool {
        self.as_ref().starts_with("$app$")
    }

    fn is_password(&self) -> bool {
        !self.is_otp_auth() && !self.is_app_password()
    }
}

impl<'de> serde::Deserialize<'de> for PrincipalValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrincipalValueVisitor;

        impl<'de> Visitor<'de> for PrincipalValueVisitor {
            type Value = PrincipalValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an optional values or a sequence of values")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PrincipalValue::default())
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PrincipalValue::Integer(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PrincipalValue::String(value))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PrincipalValue::String(v.to_string()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut vec_u64 = Vec::new();
                let mut vec_string = Vec::new();

                while let Some(value) = seq.next_element::<StringOrU64>()? {
                    match value {
                        StringOrU64::String(s) => vec_string.push(s),
                        StringOrU64::U64(u) => vec_u64.push(u),
                    }
                }

                match (vec_u64.is_empty(), vec_string.is_empty()) {
                    (true, false) => Ok(PrincipalValue::StringList(vec_string)),
                    (false, true) => Ok(PrincipalValue::IntegerList(vec_u64)),
                    (true, true) => Ok(PrincipalValue::StringList(vec_string)),
                    _ => Err(serde::de::Error::custom("invalid principal value")),
                }
            }
        }

        deserializer.deserialize_any(PrincipalValueVisitor)
    }
}

#[derive(Debug)]
enum StringOrU64 {
    String(String),
    U64(u64),
}

impl<'de> serde::Deserialize<'de> for StringOrU64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringOrU64Visitor;

        impl Visitor<'_> for StringOrU64Visitor {
            type Value = StringOrU64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or u64")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StringOrU64::String(value.to_string()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StringOrU64::U64(value))
            }
        }

        deserializer.deserialize_any(StringOrU64Visitor)
    }
}

pub static PERMISSIONS: &[(&str, &str)] = &[
    ("impersonate", "Act on behalf of another user"),
    ("unlimited-requests", "Perform unlimited requests"),
    ("unlimited-uploads", "Upload unlimited data"),
    ("delete-system-folders", "Delete of system folders"),
    ("message-queue-list", "View message queue"),
    (
        "message-queue-get",
        "Retrieve specific messages from the queue",
    ),
    ("message-queue-update", "Modify queued messages"),
    ("message-queue-delete", "Remove messages from the queue"),
    (
        "outgoing-report-list",
        "View outgoing DMARC and TLS reports",
    ),
    (
        "outgoing-report-get",
        "Retrieve specific outgoing DMARC and TLS reports",
    ),
    (
        "outgoing-report-delete",
        "Remove outgoing DMARC and TLS reports",
    ),
    (
        "incoming-report-list",
        "View incoming DMARC, TLS and ARF reports",
    ),
    (
        "incoming-report-get",
        "Retrieve specific incoming DMARC, TLS and ARF reports",
    ),
    (
        "incoming-report-delete",
        "Remove incoming DMARC, TLS and ARF reports",
    ),
    ("settings-list", "View system settings"),
    ("settings-update", "Modify system settings"),
    ("settings-delete", "Remove system settings"),
    ("settings-reload", "Refresh system settings"),
    ("individual-list", "View list of user accounts"),
    ("individual-get", "Retrieve specific account information"),
    ("individual-update", "Modify user account information"),
    ("individual-delete", "Remove user accounts"),
    ("individual-create", "Add new user accounts"),
    ("group-list", "View list of user groups"),
    ("group-get", "Retrieve specific group information"),
    ("group-update", "Modify group information"),
    ("group-delete", "Remove user groups"),
    ("group-create", "Add new user groups"),
    ("domain-list", "View list of email domains"),
    ("domain-get", "Retrieve specific domain information"),
    ("domain-create", "Add new email domains"),
    ("domain-update", "Modify domain information"),
    ("domain-delete", "Remove email domains"),
    ("tenant-list", "View list of tenants"),
    ("tenant-get", "Retrieve specific tenant information"),
    ("tenant-create", "Add new tenants"),
    ("tenant-update", "Modify tenant information"),
    ("tenant-delete", "Remove tenants"),
    ("mailing-list-list", "View list of mailing lists"),
    (
        "mailing-list-get",
        "Retrieve specific mailing list information",
    ),
    ("mailing-list-create", "Create new mailing lists"),
    ("mailing-list-update", "Modify mailing list information"),
    ("mailing-list-delete", "Remove mailing lists"),
    ("role-list", "View list of roles"),
    ("role-get", "Retrieve specific role information"),
    ("role-create", "Create new roles"),
    ("role-update", "Modify role information"),
    ("role-delete", "Remove roles"),
    ("principal-list", "View list of principals"),
    ("principal-get", "Retrieve specific principal information"),
    ("principal-create", "Create new principals"),
    ("principal-update", "Modify principal information"),
    ("principal-delete", "Remove principals"),
    ("blob-fetch", "Retrieve arbitrary blobs"),
    ("purge-blob-store", "Purge the blob storage"),
    ("purge-data-store", "Purge the data storage"),
    ("purge-in-memory-store", "Purge the in-memory storage"),
    ("purge-account", "Purge user accounts"),
    ("fts-reindex", "Rebuild the full-text search index"),
    ("undelete", "Restore deleted items"),
    (
        "dkim-signature-create",
        "Create DKIM signatures for email authentication",
    ),
    ("dkim-signature-get", "Retrieve DKIM signature information"),
    ("spam-filter-update", "Modify spam filter settings"),
    (
        "spam-filter-classify",
        "Classify emails with the spam filter",
    ),
    ("spam-filter-train", "Train the spam filter"),
    ("webadmin-update", "Modify web admin interface settings"),
    ("logs-view", "Access system logs"),
    ("restart", "Restart the email server"),
    ("tracing-list", "View stored traces"),
    ("tracing-get", "Retrieve specific trace information"),
    ("tracing-live", "Perform real-time tracing"),
    ("metrics-list", "View stored metrics"),
    ("metrics-live", "View real-time metrics"),
    ("authenticate", "Authenticate"),
    ("authenticate-oauth", "Authenticate via OAuth"),
    ("email-send", "Send emails"),
    ("email-receive", "Receive emails"),
    ("manage-encryption", "Manage encryption-at-rest settings"),
    ("manage-passwords", "Manage account passwords"),
    ("jmap-email-get", "Retrieve emails via JMAP"),
    ("jmap-mailbox-get", "Retrieve mailboxes via JMAP"),
    ("jmap-thread-get", "Retrieve email threads via JMAP"),
    ("jmap-identity-get", "Retrieve user identities via JMAP"),
    (
        "jmap-email-submission-get",
        "Retrieve email submission info via JMAP",
    ),
    (
        "jmap-push-subscription-get",
        "Retrieve push subscriptions via JMAP",
    ),
    ("jmap-sieve-script-get", "Retrieve Sieve scripts via JMAP"),
    (
        "jmap-vacation-response-get",
        "Retrieve vacation responses via JMAP",
    ),
    (
        "jmap-principal-get",
        "Retrieve principal information via JMAP",
    ),
    ("jmap-quota-get", "Retrieve quota information via JMAP"),
    ("jmap-blob-get", "Retrieve blobs via JMAP"),
    ("jmap-email-set", "Modify emails via JMAP"),
    ("jmap-mailbox-set", "Modify mailboxes via JMAP"),
    ("jmap-identity-set", "Modify user identities via JMAP"),
    (
        "jmap-email-submission-set",
        "Modify email submission settings via JMAP",
    ),
    (
        "jmap-push-subscription-set",
        "Modify push subscriptions via JMAP",
    ),
    ("jmap-sieve-script-set", "Modify Sieve scripts via JMAP"),
    (
        "jmap-vacation-response-set",
        "Modify vacation responses via JMAP",
    ),
    ("jmap-email-changes", "Track email changes via JMAP"),
    ("jmap-mailbox-changes", "Track mailbox changes via JMAP"),
    ("jmap-thread-changes", "Track thread changes via JMAP"),
    ("jmap-identity-changes", "Track identity changes via JMAP"),
    (
        "jmap-email-submission-changes",
        "Track email submission changes via JMAP",
    ),
    ("jmap-quota-changes", "Track quota changes via JMAP"),
    ("jmap-email-copy", "Copy emails via JMAP"),
    ("jmap-blob-copy", "Copy blobs via JMAP"),
    ("jmap-email-import", "Import emails via JMAP"),
    ("jmap-email-parse", "Parse emails via JMAP"),
    (
        "jmap-email-query-changes",
        "Track email query changes via JMAP",
    ),
    (
        "jmap-mailbox-query-changes",
        "Track mailbox query changes via JMAP",
    ),
    (
        "jmap-email-submission-query-changes",
        "Track email submission query changes via JMAP",
    ),
    (
        "jmap-sieve-script-query-changes",
        "Track Sieve script query changes via JMAP",
    ),
    (
        "jmap-principal-query-changes",
        "Track principal query changes via JMAP",
    ),
    (
        "jmap-quota-query-changes",
        "Track quota query changes via JMAP",
    ),
    ("jmap-email-query", "Perform email queries via JMAP"),
    ("jmap-mailbox-query", "Perform mailbox queries via JMAP"),
    (
        "jmap-email-submission-query",
        "Perform email submission queries via JMAP",
    ),
    (
        "jmap-sieve-script-query",
        "Perform Sieve script queries via JMAP",
    ),
    ("jmap-principal-query", "Perform principal queries via JMAP"),
    ("jmap-quota-query", "Perform quota queries via JMAP"),
    ("jmap-search-snippet", "Retrieve search snippets via JMAP"),
    (
        "jmap-sieve-script-validate",
        "Validate Sieve scripts via JMAP",
    ),
    ("jmap-blob-lookup", "Look up blobs via JMAP"),
    ("jmap-blob-upload", "Upload blobs via JMAP"),
    ("jmap-echo", "Perform JMAP echo requests"),
    ("imap-authenticate", "Authenticate via IMAP"),
    ("imap-acl-get", "Retrieve ACLs via IMAP"),
    ("imap-acl-set", "Set ACLs via IMAP"),
    ("imap-my-rights", "Retrieve own rights via IMAP"),
    ("imap-list-rights", "List rights via IMAP"),
    ("imap-append", "Append messages via IMAP"),
    ("imap-capability", "Retrieve server capabilities via IMAP"),
    ("imap-id", "Retrieve server ID via IMAP"),
    ("imap-copy", "Copy messages via IMAP"),
    ("imap-move", "Move messages via IMAP"),
    ("imap-create", "Create mailboxes via IMAP"),
    ("imap-delete", "Delete mailboxes or messages via IMAP"),
    ("imap-enable", "Enable IMAP extensions"),
    ("imap-expunge", "Expunge deleted messages via IMAP"),
    ("imap-fetch", "Fetch messages or metadata via IMAP"),
    ("imap-idle", "Use IMAP IDLE command"),
    ("imap-list", "List mailboxes via IMAP"),
    ("imap-lsub", "List subscribed mailboxes via IMAP"),
    ("imap-namespace", "Retrieve namespaces via IMAP"),
    ("imap-rename", "Rename mailboxes via IMAP"),
    ("imap-search", "Search messages via IMAP"),
    ("imap-sort", "Sort messages via IMAP"),
    ("imap-select", "Select mailboxes via IMAP"),
    ("imap-examine", "Examine mailboxes via IMAP"),
    ("imap-status", "Retrieve mailbox status via IMAP"),
    ("imap-store", "Modify message flags via IMAP"),
    ("imap-subscribe", "Subscribe to mailboxes via IMAP"),
    ("imap-thread", "Thread messages via IMAP"),
    ("pop3-authenticate", "Authenticate via POP3"),
    ("pop3-list", "List messages via POP3"),
    ("pop3-uidl", "Retrieve unique IDs via POP3"),
    ("pop3-stat", "Retrieve mailbox statistics via POP3"),
    ("pop3-retr", "Retrieve messages via POP3"),
    ("pop3-dele", "Mark messages for deletion via POP3"),
    (
        "sieve-authenticate",
        "Authenticate for Sieve script management",
    ),
    ("sieve-list-scripts", "List Sieve scripts"),
    ("sieve-set-active", "Set active Sieve script"),
    ("sieve-get-script", "Retrieve Sieve scripts"),
    ("sieve-put-script", "Upload Sieve scripts"),
    ("sieve-delete-script", "Delete Sieve scripts"),
    ("sieve-rename-script", "Rename Sieve scripts"),
    ("sieve-check-script", "Validate Sieve scripts"),
    (
        "sieve-have-space",
        "Check available space for Sieve scripts",
    ),
    ("api-key-list", "View API keys"),
    ("api-key-get", "Retrieve specific API keys"),
    ("api-key-create", "Create new API keys"),
    ("api-key-update", "Modify API keys"),
    ("api-key-delete", "Remove API keys"),
    ("oauth-client-list", "View OAuth clients"),
    ("oauth-client-get", "Retrieve specific OAuth clients"),
    ("oauth-client-create", "Create new OAuth clients"),
    ("oauth-client-update", "Modify OAuth clients"),
    ("oauth-client-delete", "Remove OAuth clients"),
    ("oauth-client-registration", "Register OAuth clients"),
    ("oauth-client-override", "Override OAuth client settings"),
    ("ai-model-interact", "Interact with AI models"),
    ("troubleshoot", "Perform troubleshooting"),
    (
        "dav-sync-collection",
        "WebDAV - Synchronize collection changes with client",
    ),
    (
        "dav-expand-property",
        "WebDAV - Expand properties that reference other resources",
    ),
    (
        "dav-principal-acl",
        "WebDAV - Set principal properties for access control",
    ),
    (
        "dav-principal-list",
        "WebDAV - List available principals in the system",
    ),
    (
        "dav-principal-match",
        "WebDAV - Match principals based on specified criteria",
    ),
    (
        "dav-principal-search",
        "WebDAV - Search for principals by property values",
    ),
    (
        "dav-principal-search-prop-set",
        "WebDAV - Define property sets for principal searches",
    ),
    (
        "dav-file-prop-find",
        "FileDAV - Retrieve properties of file resources",
    ),
    (
        "dav-file-prop-patch",
        "FileDAV - Modify properties of file resources",
    ),
    ("dav-file-get", "FileDAV - Download file resources"),
    (
        "dav-file-mk-col",
        "FileDAV - Create new file collections or directories",
    ),
    ("dav-file-delete", "FileDAV - Remove file resources"),
    ("dav-file-put", "FileDAV - Upload or modify file resources"),
    (
        "dav-file-copy",
        "FileDAV - Copy file resources to new locations",
    ),
    (
        "dav-file-move",
        "FileDAV - Move file resources to new locations",
    ),
    (
        "dav-file-lock",
        "FileDAV - Lock file resources to prevent concurrent modifications",
    ),
    (
        "dav-file-acl",
        "FileDAV - Manage access control lists for file resources",
    ),
    (
        "dav-card-prop-find",
        "CardDAV - Retrieve properties of address book entries",
    ),
    (
        "dav-card-prop-patch",
        "CardDAV - Modify properties of address book entries",
    ),
    ("dav-card-get", "CardDAV - Download address book entries"),
    (
        "dav-card-mk-col",
        "CardDAV - Create new address book collections",
    ),
    (
        "dav-card-delete",
        "CardDAV - Remove address book entries or collections",
    ),
    (
        "dav-card-put",
        "CardDAV - Upload or modify address book entries",
    ),
    (
        "dav-card-copy",
        "CardDAV - Copy address book entries to new locations",
    ),
    (
        "dav-card-move",
        "CardDAV - Move address book entries to new locations",
    ),
    (
        "dav-card-lock",
        "CardDAV - Lock address book entries to prevent concurrent modifications",
    ),
    (
        "dav-card-acl",
        "CardDAV - Manage access control lists for address book entries",
    ),
    (
        "dav-card-query",
        "CardDAV - Search for address book entries matching criteria",
    ),
    (
        "dav-card-multi-get",
        "CardDAV - Retrieve multiple address book entries in a single request",
    ),
    (
        "dav-cal-prop-find",
        "CalDAV - Retrieve properties of calendar entries",
    ),
    (
        "dav-cal-prop-patch",
        "CalDAV - Modify properties of calendar entries",
    ),
    ("dav-cal-get", "CalDAV - Download calendar entries"),
    ("dav-cal-mk-col", "CalDAV - Create new calendar collections"),
    (
        "dav-cal-delete",
        "CalDAV - Remove calendar entries or collections",
    ),
    ("dav-cal-put", "CalDAV - Upload or modify calendar entries"),
    (
        "dav-cal-copy",
        "CalDAV - Copy calendar entries to new locations",
    ),
    (
        "dav-cal-move",
        "CalDAV - Move calendar entries to new locations",
    ),
    (
        "dav-cal-lock",
        "CalDAV - Lock calendar entries to prevent concurrent modifications",
    ),
    (
        "dav-cal-acl",
        "CalDAV - Manage access control lists for calendar entries",
    ),
    (
        "dav-cal-query",
        "CalDAV - Search for calendar entries matching criteria",
    ),
    (
        "dav-cal-multi-get",
        "CalDAV - Retrieve multiple calendar entries in a single request",
    ),
    (
        "dav-cal-free-busy-query",
        "CalDAV - Query free/busy time information for scheduling",
    ),
];
