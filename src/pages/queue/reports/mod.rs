use crate::pages::queue::messages::deserialize_datetime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Report {
    Tls {
        id: String,
        domain: String,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_from: DateTime<Utc>,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_to: DateTime<Utc>,
        report: TlsFormat,
    },
    Dmarc {
        id: String,
        domain: String,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_from: DateTime<Utc>,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_to: DateTime<Utc>,
        report: DmarcFormat,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmarcFormat {
    pub rua: Vec<URI>,
    pub policy: PolicyPublished,
    pub records: Vec<Record>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsFormat {
    pub rua: Vec<ReportUri>,
    pub policy: PolicyDetails,
    pub records: Vec<Option<FailureDetails>>,
}

impl Report {
    pub fn domain(&self) -> &str {
        match self {
            Report::Tls { domain, .. } => domain,
            Report::Dmarc { domain, .. } => domain,
        }
    }

    pub fn type_(&self) -> &str {
        match self {
            Report::Tls { .. } => "TLS",
            Report::Dmarc { .. } => "DMARC",
        }
    }

    pub fn range_from(&self) -> &DateTime<Utc> {
        match self {
            Report::Tls { range_from, .. } => range_from,
            Report::Dmarc { range_from, .. } => range_from,
        }
    }

    pub fn range_to(&self) -> &DateTime<Utc> {
        match self {
            Report::Tls { range_to, .. } => range_to,
            Report::Dmarc { range_to, .. } => range_to,
        }
    }

    pub fn num_records(&self) -> usize {
        match self {
            Report::Tls { report, .. } => report.records.len(),
            Report::Dmarc { report, .. } => report.records.len(),
        }
    }
}
