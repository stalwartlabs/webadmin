use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub mod domains;
pub mod principals;

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

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "memberOf")]
    pub member_of: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "members")]
    pub members: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipalType {
    #[serde(rename = "individual")]
    #[default]
    Individual = 0,
    #[serde(rename = "group")]
    Group = 1,
    #[serde(rename = "resource")]
    Resource = 2,
    #[serde(rename = "location")]
    Location = 3,
    #[serde(rename = "superuser")]
    Superuser = 4,
    #[serde(rename = "list")]
    List = 5,
    #[serde(rename = "other")]
    Other = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrincipalField {
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "type")]
    Type,
    #[serde(rename = "quota")]
    Quota,
    #[serde(rename = "description")]
    Description,
    #[serde(rename = "secrets")]
    Secrets,
    #[serde(rename = "emails")]
    Emails,
    #[serde(rename = "memberOf")]
    MemberOf,
    #[serde(rename = "members")]
    Members,
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
        if !changes.secrets.is_empty() {
            updates.push(PrincipalUpdate {
                action: PrincipalAction::Set,
                field: PrincipalField::Secrets,
                value: PrincipalValue::StringList(changes.secrets),
            });
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
            PrincipalType::Superuser => "superuser",
            PrincipalType::List => "list",
            PrincipalType::Other => "other",
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "Individual",
            PrincipalType::Group => "Group",
            PrincipalType::Resource => "Resource",
            PrincipalType::Location => "Location",
            PrincipalType::Superuser => "Superuser",
            PrincipalType::List => "Mailing List",
            PrincipalType::Other => "Other",
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
            (PrincipalType::Superuser, false) => "superuser",
            (PrincipalType::Superuser, true) => "superusers",
            (PrincipalType::List, false) => "mailing list",
            (PrincipalType::List, true) => "mailing lists",
            (PrincipalType::Other, false) => "other",
            (PrincipalType::Other, true) => "other",
        }
    }

    pub fn resource_name(&self, list: bool) -> &'static str {
        match (self, list) {
            (PrincipalType::Individual, false) => "account",
            (PrincipalType::Individual, true) => "accounts",
            (PrincipalType::Group, false) => "group",
            (PrincipalType::Group, true) => "groups",
            (PrincipalType::List, false) => "list",
            (PrincipalType::List, true) => "lists",
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
            "superuser" => Ok(PrincipalType::Superuser),
            "list" => Ok(PrincipalType::List),
            "other" => Ok(PrincipalType::Other),
            _ => Err(format!("Invalid PrincipalType: {}", s)),
        }
    }
}
