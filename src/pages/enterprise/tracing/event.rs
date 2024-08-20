/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: LicenseRef-SEL
 *
 * This file is subject to the Stalwart Enterprise License Agreement (SEL) and
 * is not open source software. It must not be modified or distributed without
 * explicit permission from Stalwart Labs Ltd.
 * Unauthorized use, modification, or distribution is strictly prohibited.
 */

use crate::pages::queue::messages::deserialize_datetime;
use chrono::{DateTime, Utc};
use core::fmt;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Event {
    #[serde(rename = "type")]
    pub typ: String,
    pub text: Option<String>,
    pub details: Option<String>,
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "vec_map_deserialize")]
    pub data: Vec<(Key, Value)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SubEvent {
    #[serde(rename = "type")]
    pub typ: String,
    pub text: Option<String>,
    pub details: Option<String>,
    #[serde(deserialize_with = "vec_map_deserialize")]
    pub data: Vec<(Key, Value)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Key {
    AccountName,
    AccountId,
    BlobId,
    CausedBy,
    ChangeId,
    Code,
    Collection,
    Contents,
    Details,
    DkimFail,
    DkimNone,
    DkimPass,
    DmarcNone,
    DmarcPass,
    DmarcQuarantine,
    DmarcReject,
    DocumentId,
    Domain,
    Due,
    Elapsed,
    Expires,
    From,
    Hostname,
    Id,
    Key,
    Limit,
    ListenerId,
    LocalIp,
    LocalPort,
    MailboxName,
    MailboxId,
    MessageId,
    NextDsn,
    NextRetry,
    Path,
    Policy,
    QueueId,
    RangeFrom,
    RangeTo,
    Reason,
    RemoteIp,
    RemotePort,
    ReportId,
    Result,
    Size,
    Source,
    SpanId,
    SpfFail,
    SpfNone,
    SpfPass,
    Strict,
    Tls,
    To,
    Total,
    TotalFailures,
    TotalSuccesses,
    Type,
    Uid,
    UidNext,
    UidValidity,
    Url,
    ValidFrom,
    ValidTo,
    Value,
    Version,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Int(u64),
    Float(f64),
    Bool(bool),
    Event(SubEvent),
    Array(Vec<Value>),
}

impl Event {
    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.created_at.hash(&mut hasher);
        self.typ.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, key: Key) -> Option<&Value> {
        self.data
            .iter()
            .find_map(|(k, v)| if *k == key { Some(v) } else { None })
    }

    pub fn get_as_str(&self, key: Key) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    pub fn get_as_str_list(&self, key: Key) -> Box<dyn Iterator<Item = &str> + '_> {
        match self.get(key) {
            Some(v) => v.as_str_list(),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn get_as_int(&self, key: Key) -> Option<u64> {
        self.get(key).and_then(|v| v.as_int())
    }
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str_list(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            Value::Array(arr) => Box::new(arr.iter().filter_map(|v| v.as_str())),
            Value::String(s) => Box::new(std::iter::once(s.as_str())),
            _ => Box::new(std::iter::empty()),
        }
    }

    pub fn as_int(&self) -> Option<u64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
}

impl Eq for Value {}

fn vec_map_deserialize<'de, D>(deserializer: D) -> Result<Vec<(Key, Value)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecMapVisitor<K, V> {
        marker: PhantomData<Vec<(K, V)>>,
    }

    impl<'de, K, V> Visitor<'de> for VecMapVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((key, value)) = access.next_entry()? {
                vec.push((key, value));
            }

            Ok(vec)
        }
    }

    let visitor = VecMapVisitor {
        marker: PhantomData,
    };
    deserializer.deserialize_map(visitor)
}
