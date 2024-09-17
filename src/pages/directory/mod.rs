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
    pub quota: Option<IntOrMany>,

    #[serde(rename = "usedQuota")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_quota: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    pub secrets: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    pub emails: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "memberOf")]
    pub member_of: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "roles")]
    pub roles: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "lists")]
    pub lists: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "members")]
    pub members: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "enabledPermissions")]
    pub enabled_permissions: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "string_or_vec"
    )]
    #[serde(rename = "disabledPermissions")]
    pub disabled_permissions: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntOrMany {
    Int(u64),
    Many(Vec<u64>),
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
    IntegerList(Vec<u64>),
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
            let current = current.unwrap_or_default();
            let change = change.unwrap_or_default();

            if current != change {
                updates.push(PrincipalUpdate {
                    action: PrincipalAction::Set,
                    field,
                    value: PrincipalValue::String(change),
                });
            }
        }

        let current_quota = current.quota.unwrap_or(IntOrMany::Int(0));
        let changes_quota = changes.quota.unwrap_or(IntOrMany::Int(0));

        if current_quota != changes_quota {
            updates.push(PrincipalUpdate {
                action: PrincipalAction::Set,
                field: PrincipalField::Quota,
                value: match changes_quota {
                    IntOrMany::Int(v) => PrincipalValue::Integer(v),
                    IntOrMany::Many(v) => PrincipalValue::IntegerList(v),
                },
            });
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
            PrincipalType::Domain => "domains",
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

impl IntOrMany {
    pub fn first(&self) -> u64 {
        match self {
            IntOrMany::Int(i) => *i,
            IntOrMany::Many(v) => v.first().copied().unwrap_or_default(),
        }
    }
}

fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value])
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

pub static PERMISSIONS: &[(&str, &str)] = &[
    ("impersonate", "Allows acting on behalf of another user"),
    ("unlimited-requests", "Removes request limits or quotas"),
    (
        "unlimited-uploads",
        "Removes upload size or frequency limits",
    ),
    (
        "delete-system-folders",
        "Allows deletion of critical system folders",
    ),
    ("message-queue-list", "View message queue"),
    (
        "message-queue-get",
        "Retrieve specific messages from the queue",
    ),
    ("message-queue-update", "Modify queued messages"),
    ("message-queue-delete", "Remove messages from the queue"),
    ("outgoing-report-list", "View reports for outgoing emails"),
    (
        "outgoing-report-get",
        "Retrieve specific outgoing email reports",
    ),
    ("outgoing-report-delete", "Remove outgoing email reports"),
    ("incoming-report-list", "View reports for incoming emails"),
    (
        "incoming-report-get",
        "Retrieve specific incoming email reports",
    ),
    ("incoming-report-delete", "Remove incoming email reports"),
    ("settings-list", "View system settings"),
    ("settings-update", "Modify system settings"),
    ("settings-delete", "Remove system settings"),
    ("settings-reload", "Refresh system settings"),
    ("individual-list", "View list of individual users"),
    ("individual-get", "Retrieve specific user information"),
    ("individual-update", "Modify user information"),
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
    (
        "principal-list",
        "View list of principals (users or system entities)",
    ),
    ("principal-get", "Retrieve specific principal information"),
    ("principal-create", "Create new principals"),
    ("principal-update", "Modify principal information"),
    ("principal-delete", "Remove principals"),
    ("blob-fetch", "Retrieve binary large objects"),
    ("purge-blob-store", "Clear the blob storage"),
    ("purge-data-store", "Clear the data storage"),
    ("purge-lookup-store", "Clear the lookup storage"),
    (
        "purge-account",
        "Completely remove an account and all associated data",
    ),
    ("undelete", "Restore deleted items"),
    (
        "dkim-signature-create",
        "Create DKIM signatures for email authentication",
    ),
    ("dkim-signature-get", "Retrieve DKIM signature information"),
    ("update-spam-filter", "Modify spam filter settings"),
    ("update-webadmin", "Modify web admin interface settings"),
    ("logs-view", "Access system logs"),
    ("sieve-run", "Execute Sieve scripts for email filtering"),
    ("restart", "Restart the email server"),
    ("tracing-list", "View list of system traces"),
    ("tracing-get", "Retrieve specific trace information"),
    ("tracing-live", "View real-time system traces"),
    ("metrics-list", "View list of system metrics"),
    ("metrics-live", "View real-time system metrics"),
    ("authenticate", "Perform authentication"),
    ("authenticate-oauth", "Perform OAuth authentication"),
    ("email-send", "Send emails"),
    ("email-receive", "Receive emails"),
    (
        "manage-encryption",
        "Handle encryption settings and operations",
    ),
    ("manage-passwords", "Manage user passwords"),
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
];
