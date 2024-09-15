/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub mod dns;
pub mod edit;
pub mod list;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(rename = "type")]
    pub typ: Option<PrincipalType>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,

    #[serde(rename = "usedQuota")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_quota: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "memberOf")]
    pub member_of: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "roles")]
    pub roles: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "lists")]
    pub lists: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "members")]
    pub members: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "enabledPermissions")]
    pub enabled_permissions: Vec<Permission>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "disabledPermissions")]
    pub disabled_permissions: Vec<Permission>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PrincipalValue {
    String(String),
    StringList(Vec<String>),
    Integer(u64),
}

impl Principal {
    pub fn is_blank(&self) -> bool {
        self.id.is_none()
            && self.typ.is_none()
            && self.name.is_none()
            && self.secrets.is_empty()
            && self.emails.is_empty()
            && self.member_of.is_empty()
            && self.members.is_empty()
            && self.description.is_none()
    }

    pub fn into_updates(self, changes: Principal) -> Vec<PrincipalUpdate> {
        let current = self;
        let mut updates = vec![];
        match (current.name, changes.name) {
            (Some(current), Some(change)) if current != change => {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field: PrincipalField::Name,
                    value: PrincipalValue::String(change),
                });
            }
            _ => {}
        }
        match (current.typ, changes.typ) {
            (Some(current), Some(change)) if current != change => {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field: PrincipalField::Type,
                    value: PrincipalValue::String(change.id().to_string()),
                });
            }
            _ => {}
        }
        match (current.quota, changes.quota) {
            (Some(current), Some(change)) if current != change => {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field: PrincipalField::Quota,
                    value: PrincipalValue::Integer(change),
                });
            }
            _ => {}
        }
        match (
            current.description.unwrap_or_default(),
            changes.description.unwrap_or_default(),
        ) {
            (current, change) if current != change => {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field: PrincipalField::Description,
                    value: PrincipalValue::String(change),
                });
            }
            _ => {}
        }

        let mut changed_password = false;

        for new_secret in &changes.secrets {
            if !new_secret.is_app_password() && !current.secrets.contains(new_secret) {
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

        for prev_secret in current.secrets {
            if !prev_secret.is_password() {
                let mut found = false;

                for new_secret in &changes.secrets {
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
                PrincipalField::MemberOf,
                current.member_of,
                changes.member_of,
            ),
            (PrincipalField::Members, current.members, changes.members),
        ] {
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
        }
    }

    pub fn resource_name(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "accounts",
            PrincipalType::Group => "groups",
            PrincipalType::List => "lists",
            PrincipalType::Role => "roles",
            PrincipalType::Tenant => "tenants",
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
            _ => Err(format!("Invalid PrincipalType: {}", s)),
        }
    }
}

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::core::Permission;

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
