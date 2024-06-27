/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod list;
pub mod manage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: u64,
    pub return_path: String,
    pub domains: Vec<Domain>,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub created: DateTime<Utc>,
    pub size: usize,
    #[serde(default)]
    pub priority: i16,
    pub env_id: Option<String>,
    pub blob_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Domain {
    pub name: String,
    pub status: Status,
    pub recipients: Vec<Recipient>,

    pub retry_num: u32,
    #[serde(deserialize_with = "deserialize_maybe_datetime")]
    pub next_retry: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_maybe_datetime")]
    pub next_notify: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recipient {
    pub address: String,
    pub status: Status,
    pub orcpt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "completed")]
    Completed(String),
    #[serde(rename = "temp_fail")]
    TemporaryFailure(String),
    #[serde(rename = "perm_fail")]
    PermanentFailure(String),
}

impl Message {
    pub fn return_path(&self) -> &str {
        if !self.return_path.is_empty() {
            &self.return_path
        } else {
            "Mailer Daemon"
        }
    }

    pub fn next_retry(&self) -> Option<DateTime<Utc>> {
        let mut next_event = None;

        for (pos, domain) in self
            .domains
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
            .enumerate()
        {
            if pos == 0
                || domain
                    .next_retry
                    .as_ref()
                    .map_or(false, |next_retry| next_retry < &next_event.unwrap())
            {
                next_event = domain.next_retry.unwrap().into();
            }
        }

        next_event
    }

    pub fn next_dsn(&self) -> Option<DateTime<Utc>> {
        let mut next_event = None;

        for (pos, domain) in self
            .domains
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
            .enumerate()
        {
            if pos == 0
                || domain
                    .next_notify
                    .as_ref()
                    .map_or(false, |next_notify| next_notify < &next_event.unwrap())
            {
                next_event = domain.next_notify.unwrap().into();
            }
        }

        next_event
    }

    pub fn expires(&self) -> Option<DateTime<Utc>> {
        let mut next_event = None;

        for (pos, domain) in self
            .domains
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
            .enumerate()
        {
            if pos == 0 || domain.expires > next_event.unwrap() {
                next_event = domain.expires.into();
            }
        }

        next_event
    }
}

impl Status {
    pub fn unwrap_message(self) -> String {
        match self {
            Status::Completed(message)
            | Status::TemporaryFailure(message)
            | Status::PermanentFailure(message) => message,
            Status::Scheduled => "N/A".to_string(),
        }
    }
}

fn deserialize_maybe_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(value) = Option::<&str>::deserialize(deserializer)? {
        if let Ok(value) = DateTime::parse_from_rfc3339(value) {
            Ok(Some(value.to_utc()))
        } else {
            Err(serde::de::Error::custom(
                "Failed to parse RFC3339 timestamp",
            ))
        }
    } else {
        Ok(None)
    }
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Ok(value) = DateTime::parse_from_rfc3339(<&str>::deserialize(deserializer)?) {
        Ok(value.to_utc())
    } else {
        Err(serde::de::Error::custom(
            "Failed to parse RFC3339 timestamp",
        ))
    }
}
