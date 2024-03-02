pub mod arf;
pub mod display;
pub mod dmarc;
pub mod list;
pub mod tls;

use std::{fmt::Display, net::IpAddr};

use crate::pages::queue::messages::deserialize_datetime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AggregateReport {
    Tls {
        id: String,
        domain: String,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_from: DateTime<Utc>,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_to: DateTime<Utc>,
        report: TlsReport,
        rua: Vec<ReportUri>,
    },
    Dmarc {
        id: String,
        domain: String,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_from: DateTime<Utc>,
        #[serde(deserialize_with = "deserialize_datetime")]
        range_to: DateTime<Utc>,
        report: Report,
        rua: Vec<URI>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AggregateReportId {
    pub id: String,
    pub domain: String,
    pub policy: u64,
    pub created: DateTime<Utc>,
    pub due: DateTime<Utc>,
    pub typ: AggregateReportType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
pub enum AggregateReportType {
    Tls,
    Dmarc,
}

impl AggregateReportId {
    pub fn parse(id: String) -> Option<Self> {
        let mut parts = id.split('!');
        let typ = parts.next()?;

        AggregateReportId {
            domain: parts
                .next()
                .and_then(|v| if !v.is_empty() { Some(v) } else { None })?
                .to_string(),
            policy: parts.next().and_then(|p| p.parse::<u64>().ok())?,
            created: DateTime::from_timestamp(
                parts.next().and_then(|p| p.parse::<u64>().ok())? as i64,
                0,
            )?,
            due: DateTime::from_timestamp(
                parts.next().and_then(|p| p.parse::<u64>().ok())? as i64,
                0,
            )?,
            typ: match typ {
                "d" => AggregateReportType::Dmarc,
                "t" => AggregateReportType::Tls,
                _ => return None,
            },
            id,
        }
        .into()
    }
}

// TODO: use definitions from mail-auth crate (which needs to be able to compile for WASM first)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DmarcDateRange {
    pub begin: u64,
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub org_name: String,
    pub email: String,
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_range: DmarcDateRange,
    pub error: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Alignment {
    Relaxed,
    Strict,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Disposition {
    None,
    Quarantine,
    Reject,
    #[default]
    Unspecified,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ActionDisposition {
    None,
    Pass,
    Quarantine,
    Reject,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PolicyPublished {
    pub domain: String,
    pub version_published: Option<f32>,
    pub adkim: Alignment,
    pub aspf: Alignment,
    pub p: Disposition,
    pub sp: Disposition,
    pub testing: bool,
    pub fo: Option<String>,
}

impl Eq for PolicyPublished {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DmarcResult {
    Pass,
    Fail,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PolicyOverride {
    Forwarded,
    SampledOut,
    TrustedForwarder,
    MailingList,
    LocalPolicy,
    #[default]
    Other,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PolicyOverrideReason {
    type_: PolicyOverride,
    comment: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PolicyEvaluated {
    pub disposition: ActionDisposition,
    pub dkim: DmarcResult,
    pub spf: DmarcResult,
    pub reason: Vec<PolicyOverrideReason>,
}

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub source_ip: Option<IpAddr>,
    pub count: u32,
    pub policy_evaluated: PolicyEvaluated,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Extension {
    name: String,
    definition: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Identifier {
    envelope_to: Option<String>,
    envelope_from: String,
    header_from: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DkimResult {
    #[default]
    None,
    Pass,
    Fail,
    Policy,
    Neutral,
    TempError,
    PermError,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DKIMAuthResult {
    domain: String,
    selector: String,
    result: DkimResult,
    human_result: Option<String>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SPFDomainScope {
    Helo,
    MailFrom,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpfResult {
    #[default]
    None,
    Neutral,
    Pass,
    Fail,
    SoftFail,
    TempError,
    PermError,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SPFAuthResult {
    domain: String,
    scope: SPFDomainScope,
    result: SpfResult,
    human_result: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuthResult {
    dkim: Vec<DKIMAuthResult>,
    spf: Vec<SPFAuthResult>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Record {
    pub row: Row,
    pub identifiers: Identifier,
    pub auth_results: AuthResult,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Report {
    pub version: f32,
    pub report_metadata: ReportMetadata,
    pub policy_published: PolicyPublished,
    pub record: Vec<Record>,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct URI {
    pub uri: String,
    pub max_size: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TlsReport {
    #[serde(rename = "organization-name")]
    #[serde(default)]
    pub organization_name: Option<String>,

    #[serde(rename = "date-range")]
    pub date_range: TlsDateRange,

    #[serde(rename = "contact-info")]
    #[serde(default)]
    pub contact_info: Option<String>,

    #[serde(rename = "report-id")]
    #[serde(default)]
    pub report_id: String,

    #[serde(rename = "policies")]
    #[serde(default)]
    pub policies: Vec<Policy>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct Policy {
    #[serde(rename = "policy")]
    pub policy: PolicyDetails,

    #[serde(rename = "summary")]
    pub summary: Summary,

    #[serde(rename = "failure-details")]
    #[serde(default)]
    pub failure_details: Vec<FailureDetails>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct PolicyDetails {
    #[serde(rename = "policy-type")]
    pub policy_type: PolicyType,

    #[serde(rename = "policy-string")]
    #[serde(default)]
    pub policy_string: Vec<String>,

    #[serde(rename = "policy-domain")]
    #[serde(default)]
    pub policy_domain: String,

    #[serde(rename = "mx-host")]
    #[serde(default)]
    pub mx_host: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct Summary {
    #[serde(rename = "total-successful-session-count")]
    #[serde(default)]
    pub total_success: u32,

    #[serde(rename = "total-failure-session-count")]
    #[serde(default)]
    pub total_failure: u32,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct FailureDetails {
    #[serde(rename = "result-type")]
    pub result_type: ResultType,

    #[serde(rename = "sending-mta-ip")]
    pub sending_mta_ip: Option<IpAddr>,

    #[serde(rename = "receiving-mx-hostname")]
    pub receiving_mx_hostname: Option<String>,

    #[serde(rename = "receiving-mx-helo")]
    pub receiving_mx_helo: Option<String>,

    #[serde(rename = "receiving-ip")]
    pub receiving_ip: Option<IpAddr>,

    #[serde(rename = "failed-session-count")]
    #[serde(default)]
    pub failed_session_count: u32,

    #[serde(rename = "additional-information")]
    pub additional_information: Option<String>,

    #[serde(rename = "failure-reason-code")]
    pub failure_reason_code: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TlsDateRange {
    #[serde(rename = "start-datetime")]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub start_datetime: DateTime<Utc>,
    #[serde(rename = "end-datetime")]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub end_datetime: DateTime<Utc>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum PolicyType {
    #[serde(rename = "tlsa")]
    Tlsa,
    #[serde(rename = "sts")]
    Sts,
    #[serde(rename = "no-policy-found")]
    NoPolicyFound,
    #[serde(other)]
    #[default]
    Other,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultType {
    #[serde(rename = "starttls-not-supported")]
    StartTlsNotSupported,
    #[serde(rename = "certificate-host-mismatch")]
    CertificateHostMismatch,
    #[serde(rename = "certificate-expired")]
    CertificateExpired,
    #[serde(rename = "certificate-not-trusted")]
    CertificateNotTrusted,
    #[serde(rename = "validation-failure")]
    ValidationFailure,
    #[serde(rename = "tlsa-invalid")]
    TlsaInvalid,
    #[serde(rename = "dnssec-invalid")]
    DnssecInvalid,
    #[serde(rename = "dane-required")]
    DaneRequired,
    #[serde(rename = "sts-policy-fetch-error")]
    StsPolicyFetchError,
    #[serde(rename = "sts-policy-invalid")]
    StsPolicyInvalid,
    #[serde(rename = "sts-webpki-invalid")]
    StsWebpkiInvalid,
    #[serde(other)]
    #[default]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Feedback {
    pub feedback_type: FeedbackType,
    pub arrival_date: Option<i64>,
    pub authentication_results: Vec<String>,
    pub incidents: u32,
    pub original_envelope_id: Option<String>,
    pub original_mail_from: Option<String>,
    pub original_rcpt_to: Option<String>,
    pub reported_domain: Vec<String>,
    pub reported_uri: Vec<String>,
    pub reporting_mta: Option<String>,
    pub source_ip: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub version: u32,
    pub source_port: u32,

    // Auth-Failure keys
    pub auth_failure: AuthFailureType,
    pub delivery_result: DeliveryResult,
    pub dkim_adsp_dns: Option<String>,
    pub dkim_canonicalized_body: Option<String>,
    pub dkim_canonicalized_header: Option<String>,
    pub dkim_domain: Option<String>,
    pub dkim_identity: Option<String>,
    pub dkim_selector: Option<String>,
    pub dkim_selector_dns: Option<String>,
    pub spf_dns: Option<String>,
    pub identity_alignment: IdentityAlignment,

    pub message: Option<String>,
    pub headers: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum AuthFailureType {
    Adsp,
    BodyHash,
    Revoked,
    Signature,
    Spf,
    Dmarc,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum IdentityAlignment {
    None,
    Spf,
    Dkim,
    DkimSpf,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum DeliveryResult {
    Delivered,
    Spam,
    Policy,
    Reject,
    Other,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum FeedbackType {
    Abuse,
    AuthFailure,
    Fraud,
    NotSpam,
    #[default]
    Other,
    Virus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportUri {
    Mail(String),
    Http(String),
}

impl From<Disposition> for ActionDisposition {
    fn from(value: Disposition) -> Self {
        match value {
            Disposition::None => ActionDisposition::None,
            Disposition::Quarantine => ActionDisposition::Quarantine,
            Disposition::Reject => ActionDisposition::Reject,
            Disposition::Unspecified => ActionDisposition::Unspecified,
        }
    }
}

impl Display for Alignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Alignment::Relaxed => "Relaxed",
            Alignment::Strict => "Strict",
            Alignment::Unspecified => "Unspecified",
        })
    }
}

impl Display for Disposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Disposition::None => "None",
            Disposition::Quarantine => "Quarantine",
            Disposition::Reject => "Reject",
            Disposition::Unspecified => "Unspecified",
        })
    }
}

impl Display for ActionDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ActionDisposition::Pass => "Pass",
            ActionDisposition::Quarantine => "Quarantine",
            ActionDisposition::Reject => "Reject",
            ActionDisposition::Unspecified => "Unspecified",
            ActionDisposition::None => "None",
        })
    }
}

impl Display for PolicyOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PolicyOverride::Forwarded => "Forwarded",
            PolicyOverride::SampledOut => "Sampled Out",
            PolicyOverride::TrustedForwarder => "Trusted Forwarder",
            PolicyOverride::MailingList => "Mailing List",
            PolicyOverride::LocalPolicy => "Local Policy",
            PolicyOverride::Other => "Other",
        })
    }
}

impl Display for PolicyOverrideReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.type_.fmt(f)?;
        if let Some(comment) = &self.comment {
            write!(f, " ({})", comment)?;
        }
        Ok(())
    }
}

impl Display for DkimResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DkimResult::None => "None",
            DkimResult::Pass => "Pass",
            DkimResult::Fail => "Fail",
            DkimResult::Policy => "Policy",
            DkimResult::Neutral => "Neutral",
            DkimResult::TempError => "TempError",
            DkimResult::PermError => "PermError",
        })
    }
}

impl Display for SpfResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SpfResult::None => "None",
            SpfResult::Neutral => "Neutral",
            SpfResult::Pass => "Pass",
            SpfResult::Fail => "Fail",
            SpfResult::SoftFail => "SoftFail",
            SpfResult::TempError => "TempError",
            SpfResult::PermError => "PermError",
        })
    }
}

impl Display for SPFDomainScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SPFDomainScope::Helo => "HELO",
            SPFDomainScope::MailFrom => "MAIL FROM",
            SPFDomainScope::Unspecified => "Unspecified",
        })
    }
}

impl Display for ReportUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportUri::Mail(uri) => write!(f, "mailto:{}", uri),
            ReportUri::Http(uri) => f.write_str(uri),
        }
    }
}

impl Display for PolicyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyType::Tlsa => f.write_str("TLSA"),
            PolicyType::Sts => f.write_str("MTA-STS"),
            PolicyType::NoPolicyFound => f.write_str("No Policy Found"),
            PolicyType::Other => f.write_str("Other"),
        }
    }
}

impl Display for ResultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultType::StartTlsNotSupported => f.write_str("STARTTLS not supported"),
            ResultType::CertificateHostMismatch => f.write_str("Certificate host mismatch"),
            ResultType::CertificateExpired => f.write_str("Certificate expired"),
            ResultType::CertificateNotTrusted => f.write_str("Certificate not trusted"),
            ResultType::ValidationFailure => f.write_str("Validation failure"),
            ResultType::TlsaInvalid => f.write_str("TLSA invalid"),
            ResultType::DnssecInvalid => f.write_str("DNSSEC invalid"),
            ResultType::DaneRequired => f.write_str("DANE required"),
            ResultType::StsPolicyFetchError => f.write_str("STS policy fetch error"),
            ResultType::StsPolicyInvalid => f.write_str("STS policy invalid"),
            ResultType::StsWebpkiInvalid => f.write_str("STS webpki invalid"),
            ResultType::Other => f.write_str("Other"),
        }
    }
}

impl Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Abuse => f.write_str("Abuse"),
            FeedbackType::AuthFailure => f.write_str("Authentication Failure"),
            FeedbackType::Fraud => f.write_str("Fraud"),
            FeedbackType::NotSpam => f.write_str("Not Spam"),
            FeedbackType::Other => f.write_str("Other"),
            FeedbackType::Virus => f.write_str("Virus"),
        }
    }
}

impl Display for AuthFailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthFailureType::Adsp => f.write_str("ADSP"),
            AuthFailureType::BodyHash => f.write_str("Body Hash"),
            AuthFailureType::Revoked => f.write_str("Revoked"),
            AuthFailureType::Signature => f.write_str("Signature"),
            AuthFailureType::Spf => f.write_str("SPF"),
            AuthFailureType::Dmarc => f.write_str("DMARC"),
            AuthFailureType::Unspecified => f.write_str("Unspecified"),
        }
    }
}

impl Display for IdentityAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityAlignment::None => f.write_str("None"),
            IdentityAlignment::Spf => f.write_str("SPF"),
            IdentityAlignment::Dkim => f.write_str("DKIM"),
            IdentityAlignment::DkimSpf => f.write_str("DKIM+SPF"),
            IdentityAlignment::Unspecified => f.write_str("Unspecified"),
        }
    }
}

impl Display for DeliveryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryResult::Delivered => f.write_str("Delivered"),
            DeliveryResult::Spam => f.write_str("Spam"),
            DeliveryResult::Policy => f.write_str("Policy"),
            DeliveryResult::Reject => f.write_str("Reject"),
            DeliveryResult::Other => f.write_str("Other"),
            DeliveryResult::Unspecified => f.write_str("Unspecified"),
        }
    }
}

#[cfg(feature = "demo")]
pub fn test_dmarc_report() -> Report {
    Report {
        version: 0.0,
        report_metadata: ReportMetadata {
            org_name: "ACME Inc.".to_string(),
            email: "hello@world.org".to_string(),
            extra_contact_info: Some("John Doe".to_string()),
            report_id: "1234567890".to_string(),
            date_range: DmarcDateRange {
                begin: Utc::now().timestamp() as u64,
                end: (Utc::now().timestamp() + 86400) as u64,
            },
            error: vec!["Invalid record".to_string()],
        },
        policy_published: PolicyPublished {
            domain: "example.org".into(),
            version_published: None,
            adkim: Alignment::Strict,
            aspf: Alignment::Relaxed,
            p: Disposition::Quarantine,
            sp: Disposition::Reject,
            testing: true,
            fo: "0".to_string().into(),
        },
        record: vec![
            test_dmarc_record(
                "john",
                ActionDisposition::Pass,
                DmarcResult::Pass,
                DmarcResult::Pass,
                1,
            ),
            test_dmarc_record(
                "mike",
                ActionDisposition::Reject,
                DmarcResult::Fail,
                DmarcResult::Pass,
                2,
            ),
            test_dmarc_record(
                "tom",
                ActionDisposition::Quarantine,
                DmarcResult::Pass,
                DmarcResult::Pass,
                3,
            ),
            test_dmarc_record(
                "bill",
                ActionDisposition::None,
                DmarcResult::Pass,
                DmarcResult::Fail,
                4,
            ),
            test_dmarc_record(
                "jane",
                ActionDisposition::Unspecified,
                DmarcResult::Fail,
                DmarcResult::Pass,
                5,
            ),
            test_dmarc_record(
                "toto",
                ActionDisposition::Pass,
                DmarcResult::Fail,
                DmarcResult::Fail,
                6,
            ),
            test_dmarc_record(
                "pepe",
                ActionDisposition::None,
                DmarcResult::Pass,
                DmarcResult::Pass,
                7,
            ),
            test_dmarc_record(
                "mark",
                ActionDisposition::Pass,
                DmarcResult::Pass,
                DmarcResult::Pass,
                8,
            ),
            test_dmarc_record(
                "alfred",
                ActionDisposition::Quarantine,
                DmarcResult::Pass,
                DmarcResult::Fail,
                9,
            ),
            test_dmarc_record(
                "wilhelm",
                ActionDisposition::Reject,
                DmarcResult::Pass,
                DmarcResult::Pass,
                10,
            ),
            test_dmarc_record(
                "timothy",
                ActionDisposition::Quarantine,
                DmarcResult::Fail,
                DmarcResult::Pass,
                11,
            ),
            test_dmarc_record(
                "rupert",
                ActionDisposition::Reject,
                DmarcResult::Pass,
                DmarcResult::Pass,
                12,
            ),
            test_dmarc_record(
                "robert",
                ActionDisposition::Pass,
                DmarcResult::Fail,
                DmarcResult::Fail,
                13,
            ),
            test_dmarc_record(
                "jane",
                ActionDisposition::Quarantine,
                DmarcResult::Pass,
                DmarcResult::Pass,
                14,
            ),
            test_dmarc_record(
                "mary",
                ActionDisposition::Pass,
                DmarcResult::Fail,
                DmarcResult::Fail,
                15,
            ),
            test_dmarc_record(
                "melanie",
                ActionDisposition::None,
                DmarcResult::Pass,
                DmarcResult::Fail,
                16,
            ),
        ],
        extensions: vec![],
    }
}

#[cfg(feature = "demo")]
fn test_dmarc_record(
    addr: &str,
    disposition: ActionDisposition,
    dkim: DmarcResult,
    spf: DmarcResult,
    count: u32,
) -> Record {
    Record {
        row: Row {
            source_ip: format!("127.0.0.{count}").parse::<IpAddr>().unwrap().into(),
            count,
            policy_evaluated: PolicyEvaluated {
                disposition,
                dkim,
                spf,
                reason: vec![PolicyOverrideReason {
                    type_: PolicyOverride::Forwarded,
                    comment: "automated".to_string().into(),
                }],
            },
        },
        identifiers: Identifier {
            envelope_to: format!("recipient@{addr}.org").into(),
            envelope_from: format!("{addr}@example.org").to_string(),
            header_from: format!("{addr}.doe@example.org").to_string(),
        },
        auth_results: AuthResult {
            dkim: vec![
                DKIMAuthResult {
                    domain: "foobar.com".to_string(),
                    selector: "default".to_string(),
                    result: DkimResult::Fail,
                    human_result: Some("Signature verification failed".to_string()),
                },
                DKIMAuthResult {
                    domain: "foobar.org".to_string(),
                    selector: "test".to_string(),
                    result: DkimResult::Pass,
                    human_result: None,
                },
                DKIMAuthResult {
                    domain: "foobar.net".to_string(),
                    selector: "default".to_string(),
                    result: DkimResult::TempError,
                    human_result: Some("Something went wrong".to_string()),
                },
            ],
            spf: vec![
                SPFAuthResult {
                    domain: "example.org".to_string(),
                    scope: SPFDomainScope::Helo,
                    result: SpfResult::Pass,
                    human_result: None,
                },
                SPFAuthResult {
                    domain: "test.org".to_string(),
                    scope: SPFDomainScope::MailFrom,
                    result: SpfResult::Fail,
                    human_result: Some("Some human readable result".to_string()),
                },
                SPFAuthResult {
                    domain: "foobar.org".to_string(),
                    scope: SPFDomainScope::Helo,
                    result: SpfResult::SoftFail,
                    human_result: Some("Something went wrong".to_string()),
                },
            ],
        },
        extensions: vec![],
    }
}

#[cfg(feature = "demo")]
pub fn test_tls_report() -> TlsReport {
    TlsReport {
        organization_name: Some("ACME Inc.".to_string()),
        date_range: TlsDateRange {
            start_datetime: Utc::now(),
            end_datetime: Utc::now() + chrono::Duration::days(1),
        },
        contact_info: Some("John Doe".to_string()),
        report_id: "1234567890".to_string(),
        policies: vec![
            test_tls_policy(
                PolicyType::Sts,
                "example.org",
                10,
                vec![
                    ResultType::StartTlsNotSupported,
                    ResultType::CertificateHostMismatch,
                    ResultType::CertificateExpired,
                    ResultType::CertificateNotTrusted,
                    ResultType::ValidationFailure,
                ],
            ),
            test_tls_policy(
                PolicyType::Tlsa,
                "foobar.org",
                5,
                vec![
                    ResultType::TlsaInvalid,
                    ResultType::DnssecInvalid,
                    ResultType::DaneRequired,
                ],
            ),
            test_tls_policy(PolicyType::NoPolicyFound, "test.org", 10, vec![]),
            test_tls_policy(
                PolicyType::Sts,
                "other.org",
                10,
                vec![
                    ResultType::StsPolicyFetchError,
                    ResultType::StsPolicyInvalid,
                    ResultType::StsWebpkiInvalid,
                ],
            ),
            test_tls_policy(
                PolicyType::Sts,
                "example.com",
                10,
                vec![
                    ResultType::StartTlsNotSupported,
                    ResultType::CertificateHostMismatch,
                    ResultType::CertificateExpired,
                    ResultType::CertificateNotTrusted,
                    ResultType::ValidationFailure,
                ],
            ),
            test_tls_policy(
                PolicyType::Tlsa,
                "foobar.com",
                5,
                vec![
                    ResultType::TlsaInvalid,
                    ResultType::DnssecInvalid,
                    ResultType::DaneRequired,
                ],
            ),
            test_tls_policy(PolicyType::NoPolicyFound, "test.com", 10, vec![]),
            test_tls_policy(
                PolicyType::Sts,
                "other.com",
                10,
                vec![
                    ResultType::StsPolicyFetchError,
                    ResultType::StsPolicyInvalid,
                    ResultType::StsWebpkiInvalid,
                ],
            ),
        ],
    }
}

#[cfg(feature = "demo")]
fn test_tls_policy(
    policy_type: PolicyType,
    host: &str,
    total_success: u32,
    failures: Vec<ResultType>,
) -> Policy {
    Policy {
        policy: PolicyDetails {
            policy_type,
            policy_string: vec![],
            policy_domain: host.to_string(),
            mx_host: vec![format!("mx1.{}", host), format!("mx2.{}", host)],
        },
        summary: Summary {
            total_success,
            total_failure: failures.len() as u32,
        },
        failure_details: failures
            .into_iter()
            .enumerate()
            .map(|(i, f)| test_tls_failure(f, host, i))
            .collect(),
    }
}

#[cfg(feature = "demo")]
fn test_tls_failure(result_type: ResultType, domain: &str, id: usize) -> FailureDetails {
    FailureDetails {
        result_type,
        sending_mta_ip: Some(format!("127.0.0.{id}").parse::<IpAddr>().unwrap()),
        receiving_mx_hostname: Some(format!("host{id}.{domain}")),
        receiving_mx_helo: Some(format!("hello{id}.{domain}")),
        receiving_ip: Some(format!("192.168.1.{id}").parse::<IpAddr>().unwrap()),
        failed_session_count: (id * 2) as u32,
        additional_information: Some(format!("Some additional information for {id}")),
        failure_reason_code: Some(format!("Some reason code for {id}")),
    }
}
