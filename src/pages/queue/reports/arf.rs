/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::vec;

use chrono::{DateTime, Utc};
use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::{
        card::{Card, CardItem},
        form::button::Button,
        icon::{IconAlertTriangle, IconClock, IconDocumentChartBar},
        report::{ReportItem, ReportSection, ReportTextValue, ReportView},
        Color,
    },
    pages::{
        queue::reports::{DeliveryResult, FeedbackType, IdentityAlignment},
        FormatDateTime,
    },
};

use super::Feedback;

#[component]
#[allow(unused_parens)]
pub fn ArfReportDisplay(
    report: Feedback,
    received: DateTime<Utc>,
    extra: Vec<(String, String)>,
    back_url: String,
) -> impl IntoView {
    let received_date = received.format_date();
    let received_time = received.format_time();
    let arrival_date = report
        .arrival_date
        .and_then(|date| DateTime::from_timestamp(date, 0))
        .unwrap_or(received);
    let arrival_time = arrival_date.format_time();
    let arrival_date = arrival_date.format_date();
    let has_port = report.source_port > 0;
    let extra = extra
        .into_iter()
        .filter_map(|(k, v)| {
            if !v.is_empty() {
                Some(view! {
                    <ReportItem label=k>
                        <ReportTextValue value=v/>
                    </ReportItem>
                })
            } else {
                None
            }
        })
        .collect_view();

    let auth_failure = if report.feedback_type == FeedbackType::AuthFailure {
        let items = [
            ("Failure Type", Some(report.auth_failure.to_string())),
            (
                "Delivery Result",
                if report.delivery_result != DeliveryResult::Unspecified {
                    Some(report.delivery_result.to_string())
                } else {
                    None
                },
            ),
            ("DKIM ADSP DNS", report.dkim_adsp_dns),
            ("DKIM Canonicalized Body", report.dkim_canonicalized_body),
            (
                "DKIM Canonicalized Header",
                report.dkim_canonicalized_header,
            ),
            ("DKIM Domain", report.dkim_domain),
            ("DKIM Identity", report.dkim_identity),
            ("DKIM Selector", report.dkim_selector),
            ("DKIM Selector DNS", report.dkim_selector_dns),
            ("SPF DNS", report.spf_dns),
            (
                "Identity Alignment",
                if report.identity_alignment != IdentityAlignment::Unspecified {
                    Some(report.identity_alignment.to_string())
                } else {
                    None
                },
            ),
        ];
        let items = items
            .into_iter()
            .filter_map(|(k, v)| {
                let v = v?;
                Some(view! {
                    <ReportItem label=k.to_string()>
                        <ReportTextValue value=v/>
                    </ReportItem>
                })
            })
            .collect_view();

        Some(
            view! { <ReportSection title="Authentication Failure Details">{items}</ReportSection> }
                .into_view(),
        )
    } else {
        None
    };

    view! {
        <Card>
            <CardItem title="Report Type" contents=report.feedback_type.to_string()>

                <IconDocumentChartBar attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Incidents" contents=std::cmp::max(report.incidents, 1).to_string()>

                <IconAlertTriangle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Received" contents=received_date subcontents=received_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Arrival" contents=arrival_date subcontents=arrival_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

        </Card>

        <ReportView>
            <ReportSection title="Report Details">
                <ReportItem label="Reported Domain" hide=report.reported_domain.is_empty()>
                    <ReportTextValue value=report.reported_domain.join(",")/>
                </ReportItem>
                <ReportItem label="Reported URI" hide=report.reported_uri.is_empty()>
                    <ReportTextValue value=report.reported_uri.join(",")/>
                </ReportItem>

                <ReportItem
                    label="Authentication Results"
                    hide=report.authentication_results.is_empty()
                >
                    <ReportTextValue value=report.authentication_results.join(", ")/>
                </ReportItem>
                <ReportItem label="Original Mail From" hide=report.original_mail_from.is_none()>
                    <ReportTextValue value=report.original_mail_from.unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Original Rcpt To" hide=report.original_rcpt_to.is_none()>
                    <ReportTextValue value=report.original_rcpt_to.unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Original Envelope Id" hide=report.original_envelope_id.is_none()>
                    <ReportTextValue value=report.original_envelope_id.unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Reporting MTA" hide=report.reporting_mta.is_none()>
                    <ReportTextValue value=report.reporting_mta.unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Source IP" hide=report.source_ip.is_none()>
                    <ReportTextValue value=report
                        .source_ip
                        .map(|ip| ip.to_string())
                        .unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Source Port" hide=has_port>
                    <ReportTextValue value=report.source_port.to_string()/>
                </ReportItem>
                <ReportItem label="User Agent" hide=report.user_agent.is_none()>
                    <ReportTextValue value=report.user_agent.unwrap_or_default()/>
                </ReportItem>

                {extra}
            </ReportSection>
            {auth_failure}

            <div class="flex justify-end">

                <Button
                    text="Close"
                    color=Color::Blue
                    on_click=move |_| {
                        use_navigate()(&back_url, Default::default());
                    }
                />

            </div>
        </ReportView>
    }
}
