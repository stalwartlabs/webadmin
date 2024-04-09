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

use crate::{
    components::{
        messages::alert::{use_alerts, Alert, Alerts},
        skeleton::Skeleton,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::queue::reports::{dmarc::DmarcReportDisplay, tls::TlsReportDisplay, AggregateReport},
};

pub(super) const PAGE_SIZE: u32 = 10;

#[component]
pub fn ReportDisplay() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let fetch_report = create_resource(
        move || params().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();

            async move {
                #[cfg(feature = "demo")]
                {
                    if id == "dmarc_demo" {
                        return Ok(AggregateReport::Dmarc {
                            rua: vec![crate::pages::queue::reports::URI {
                                uri: "rcpt@sender.org".to_string(),
                                max_size: 0,
                            }],
                            id,
                            domain: "foobar.net".to_string(),
                            range_from: chrono::Utc::now(),
                            range_to: chrono::Utc::now(),
                            report: crate::pages::queue::reports::test_dmarc_report(),
                        });
                    } else if id == "tls_demo" {
                        return Ok(AggregateReport::Tls {
                            rua: vec![crate::pages::queue::reports::ReportUri::Mail(
                                "rcpt@sender.org".to_string(),
                            )],
                            id,
                            domain: "foobar.net".to_string(),
                            range_from: chrono::Utc::now(),
                            range_to: chrono::Utc::now(),
                            report: crate::pages::queue::reports::test_tls_report(),
                        });
                    }
                }

                HttpRequest::get(format!("/api/queue/reports/{id}"))
                    .with_authorization(&auth)
                    .send::<AggregateReport>()
                    .await
            }
        },
    );

    provide_context(create_rw_signal::<HashSet<String>>(HashSet::new()));

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
                    use_navigate()("/manage/queue/reports", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(report)) => {
                    match report {
                        AggregateReport::Tls { report, rua, .. } => {
                            Some(
                                view! {
                                    <TlsReportDisplay
                                        report=report
                                        extra=vec![
                                            (
                                                "Report URI".to_string(),
                                                rua
                                                    .into_iter()
                                                    .map(|uri| uri.to_string())
                                                    .collect::<Vec<String>>()
                                                    .join(", "),
                                            ),
                                        ]

                                        back_url="/manage/queue/reports".to_string()
                                    />
                                }
                                    .into_view(),
                            )
                        }
                        AggregateReport::Dmarc { report, rua, .. } => {
                            Some(
                                view! {
                                    <DmarcReportDisplay
                                        report=report
                                        extra=vec![
                                            (
                                                "Report URI".to_string(),
                                                rua
                                                    .into_iter()
                                                    .map(|uri| uri.uri)
                                                    .collect::<Vec<String>>()
                                                    .join(", "),
                                            ),
                                        ]

                                        back_url="/manage/queue/reports".to_string()
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
