use serde::{Deserialize, Serialize};

use crate::components::form::select::SelectOption;

pub mod domains;
pub mod principals;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct List<T> {
    pub items: Vec<T>,
    pub total: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(rename = "type")]
    pub typ: Option<Type>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u32>,

    #[serde(rename = "usedQuota")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_quota: Option<u32>,

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
pub enum Type {
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
    Integer(u32),
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
                    value: PrincipalValue::String(change.value()),
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

impl SelectOption for Type {
    fn label(&self) -> String {
        self.name().to_string()
    }

    fn value(&self) -> String {
        self.id().to_string()
    }
}

impl Type {
    pub fn id(&self) -> &'static str {
        match self {
            Type::Individual => "individual",
            Type::Group => "group",
            Type::Resource => "resource",
            Type::Location => "location",
            Type::Superuser => "superuser",
            Type::List => "list",
            Type::Other => "other",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Type::Individual => "Individual",
            Type::Group => "Group",
            Type::Resource => "Resource",
            Type::Location => "Location",
            Type::Superuser => "Superuser",
            Type::List => "Mailing List",
            Type::Other => "Other",
        }
    }

    pub fn item_name(&self, plural: bool) -> &'static str {
        match (self, plural) {
            (Type::Individual, false) => "account",
            (Type::Individual, true) => "accounts",
            (Type::Group, false) => "group",
            (Type::Group, true) => "groups",
            (Type::Resource, false) => "resource",
            (Type::Resource, true) => "resources",
            (Type::Location, false) => "location",
            (Type::Location, true) => "locations",
            (Type::Superuser, false) => "superuser",
            (Type::Superuser, true) => "superusers",
            (Type::List, false) => "mailing list",
            (Type::List, true) => "mailing lists",
            (Type::Other, false) => "other",
            (Type::Other, true) => "other",
        }
    }

    pub fn resource_name(&self, list: bool) -> &'static str {
        match (self, list) {
            (Type::Individual, false) => "account",
            (Type::Individual, true) => "accounts",
            (Type::Group, false) => "group",
            (Type::Group, true) => "groups",
            (Type::List, false) => "list",
            (Type::List, true) => "lists",
            _ => unimplemented!("resource_name for {:?}", self),
        }
    }
}
