/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
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
    pub recipients: Vec<Recipient>,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub created: DateTime<Utc>,
    pub size: usize,
    #[serde(default)]
    pub priority: i16,
    #[serde(default)]
    pub env_id: Option<String>,
    pub blob_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recipient {
    pub address: String,
    pub status: Status,
    pub queue: String,
    pub retry_num: u32,
    #[serde(deserialize_with = "deserialize_maybe_datetime", default)]
    pub next_retry: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_maybe_datetime", default)]
    pub next_notify: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_maybe_datetime", default)]
    pub expires: Option<DateTime<Utc>>,
    #[serde(default)]
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

        for (pos, recipient) in self
            .recipients
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
            .enumerate()
        {
            if pos == 0
                || recipient
                    .next_retry
                    .as_ref()
                    .is_some_and(|next_retry| next_retry < &next_event.unwrap())
            {
                next_event = recipient.next_retry;
            }
        }

        next_event
    }

    pub fn next_dsn(&self) -> Option<DateTime<Utc>> {
        let mut next_event = None;

        for (pos, recipient) in self
            .recipients
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
            .enumerate()
        {
            if pos == 0
                || recipient
                    .next_notify
                    .as_ref()
                    .is_some_and(|next_notify| next_notify < &next_event.unwrap())
            {
                next_event = recipient.next_notify;
            }
        }

        next_event
    }

    pub fn expires(&self) -> Option<DateTime<Utc>> {
        let mut next_event = None;

        for recipient in self
            .recipients
            .iter()
            .filter(|d| matches!(d.status, Status::Scheduled | Status::TemporaryFailure(_)))
        {
            if let Some(expires) = recipient.expires {
                if let Some(next_event) = &mut next_event {
                    if expires < *next_event {
                        *next_event = expires;
                    }
                } else {
                    next_event = Some(expires);
                }
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
