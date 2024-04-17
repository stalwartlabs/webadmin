/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::{collections::HashSet, vec};

use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        messages::alert::{use_alerts, Alert, Alerts},
        skeleton::Skeleton,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::{
        queue::reports::{
            arf::ArfReportDisplay, dmarc::DmarcReportDisplay, tls::TlsReportDisplay, Feedback,
            Report, TlsReport,
        },
        reports::IncomingReport,
    },
};

use super::{parse_report_date, ReportType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
enum ReportWrapper {
    Dmarc(IncomingReport<Report>),
    Tls(IncomingReport<TlsReport>),
    Arf(IncomingReport<Feedback>),
}

#[component]
pub fn IncomingReportDisplay() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let report_type = create_memo(move |_| {
        match params.get()
            .get("object")
            .map(|id| id.as_str())
            .unwrap_or_default()
        {
            "dmarc" => ReportType::Dmarc,
            "tls" => ReportType::Tls,
            "arf" => ReportType::Arf,
            _ => ReportType::Dmarc,
        }
    });
    let fetch_report = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();
            let report_type = report_type.get();

            async move {
                match report_type {
                    ReportType::Dmarc => HttpRequest::get(format!("/api/reports/dmarc/{id}"))
                        .with_authorization(&auth)
                        .send::<IncomingReport<Report>>()
                        .await
                        .map(ReportWrapper::Dmarc),
                    ReportType::Tls => HttpRequest::get(format!("/api/reports/tls/{id}"))
                        .with_authorization(&auth)
                        .send::<IncomingReport<TlsReport>>()
                        .await
                        .map(ReportWrapper::Tls),
                    ReportType::Arf => HttpRequest::get(format!("/api/reports/arf/{id}"))
                        .with_authorization(&auth)
                        .send::<IncomingReport<Feedback>>()
                        .await
                        .map(ReportWrapper::Arf),
                }
            }
        },
    );

    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
    provide_context(selected);

    view! {
        <Alerts/>
        <Transition fallback=Skeleton>

            {move || match fetch_report.get() {
                None => None,
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(http::Error::NotFound)) => {
                    use_navigate()(
                        &format!("/manage/reports/{}", report_type.get().as_str()),
                        Default::default(),
                    );
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(report)) => {
                    match report {
                        ReportWrapper::Tls(report) => {
                            let (report, extra) = report.unwrap_report();
                            Some(
                                view! {
                                    <TlsReportDisplay
                                        report=report
                                        extra=extra
                                        back_url="/manage/reports/tls".to_string()
                                    />
                                }
                                    .into_view(),
                            )
                        }
                        ReportWrapper::Dmarc(report) => {
                            let (report, extra) = report.unwrap_report();
                            Some(
                                view! {
                                    <DmarcReportDisplay
                                        report=report
                                        extra=extra
                                        back_url="/manage/reports/dmarc".to_string()
                                    />
                                }
                                    .into_view(),
                            )
                        }
                        ReportWrapper::Arf(report) => {
                            let (report, extra) = report.unwrap_report();
                            let received = parse_report_date(
                                &params.get().get("id").cloned().unwrap_or_default(),
                            );
                            Some(
                                view! {
                                    <ArfReportDisplay
                                        report=report
                                        received=received
                                        extra=extra
                                        back_url="/manage/reports/arf".to_string()
                                    />
                                }
                                    .into_view(),
                            )
                        }
                    }
                }
            }}

        </Transition>
    }
}

impl<T> IncomingReport<T> {
    pub fn unwrap_report(self) -> (T, Vec<(String, String)>) {
        (
            self.report,
            vec![
                ("Received From".to_string(), self.from),
                ("Recipients".to_string(), self.to.join(", ")),
                ("Subject".to_string(), self.subject),
            ],
        )
    }
}
