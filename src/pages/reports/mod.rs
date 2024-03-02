use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::queue::reports::{ActionDisposition, Feedback, FeedbackType, Report, TlsReport};

pub mod display;
pub mod list;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Dmarc,
    Tls,
    Arf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncomingReportSummary {
    Dmarc {
        id: String,
        received: DateTime<Utc>,
        range_from: DateTime<Utc>,
        range_to: DateTime<Utc>,
        from: String,
        domains: Vec<String>,
        total_passes: u32,
        total_rejects: u32,
        total_quarantined: u32,
    },
    Tls {
        id: String,
        received: DateTime<Utc>,
        range_from: DateTime<Utc>,
        range_to: DateTime<Utc>,
        from: String,
        domains: Vec<String>,
        total_success: u32,
        total_failures: u32,
    },
    Arf {
        id: String,
        received: DateTime<Utc>,
        arrival_date: Option<DateTime<Utc>>,
        typ: FeedbackType,
        from: String,
        domains: Vec<String>,
        total_incidents: u32,
    },
}

impl IncomingReportSummary {
    pub fn id(&self) -> &str {
        match self {
            IncomingReportSummary::Dmarc { id, .. } => id,
            IncomingReportSummary::Tls { id, .. } => id,
            IncomingReportSummary::Arf { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingReport<T> {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub report: T,
}

impl ReportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Dmarc => "dmarc",
            ReportType::Tls => "tls",
            ReportType::Arf => "arf",
        }
    }
}

impl IncomingReportSummary {
    pub fn dmarc(id: String, report: IncomingReport<Report>) -> Self {
        let mut total_passes = 0;
        let mut total_quarantined = 0;
        let mut total_rejects = 0;

        for record in report.report.record {
            match record.row.policy_evaluated.disposition {
                ActionDisposition::Pass
                | ActionDisposition::None
                | ActionDisposition::Unspecified => total_passes += 1,
                ActionDisposition::Quarantine => total_quarantined += 1,
                ActionDisposition::Reject => total_rejects += 1,
            }
        }

        IncomingReportSummary::Dmarc {
            received: parse_report_date(&id),
            range_from: DateTime::from_timestamp(
                report.report.report_metadata.date_range.begin as i64,
                0,
            )
            .unwrap_or_else(Utc::now),
            range_to: DateTime::from_timestamp(
                report.report.report_metadata.date_range.end as i64,
                0,
            )
            .unwrap_or_else(Utc::now),
            from: report.from,
            domains: vec![report.report.policy_published.domain],
            id,
            total_passes,
            total_rejects,
            total_quarantined,
        }
    }

    pub fn tls(id: String, report: IncomingReport<TlsReport>) -> Self {
        let mut total_success = 0;
        let mut total_failures = 0;

        for record in &report.report.policies {
            total_success += record.summary.total_success;
            total_failures += record.summary.total_failure;
        }

        IncomingReportSummary::Tls {
            received: parse_report_date(&id),
            id,
            range_from: report.report.date_range.start_datetime,
            range_to: report.report.date_range.end_datetime,
            from: report.from,
            domains: report
                .report
                .policies
                .into_iter()
                .map(|p| p.policy.policy_domain)
                .collect(),
            total_success,
            total_failures,
        }
    }

    pub fn arf(id: String, report: IncomingReport<Feedback>) -> Self {
        IncomingReportSummary::Arf {
            received: parse_report_date(&id),
            from: report.from,
            domains: report.report.reported_domain,
            id,
            arrival_date: report
                .report
                .arrival_date
                .and_then(|date| DateTime::from_timestamp(date, 0)),
            typ: report.report.feedback_type,
            total_incidents: std::cmp::max(report.report.incidents, 1),
        }
    }
}

pub(super) fn parse_report_date(id: &str) -> DateTime<Utc> {
    DateTime::from_timestamp(
        id.split_once('_')
            .and_then(|(_, id)| id.parse::<i64>().ok())
            .unwrap_or(0),
        0,
    )
    .unwrap_or_else(Utc::now)
}
