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
            let auth = auth.get();
            let id = id.clone();

            async move {
                #[cfg(feature = "demo")]
                {
                    if id == "dmarc_demo" {
                        return Ok(Some(AggregateReport::Dmarc {
                            rua: vec![crate::pages::queue::reports::URI {
                                uri: "rcpt@sender.org".to_string(),
                                max_size: 0,
                            }],
                            id,
                            domain: "foobar.net".to_string(),
                            range_from: chrono::Utc::now(),
                            range_to: chrono::Utc::now(),
                            report: crate::pages::queue::reports::test_dmarc_report(),
                        }));
                    } else if id == "tls_demo" {
                        return Ok(Some(AggregateReport::Tls {
                            rua: vec![crate::pages::queue::reports::ReportUri::Mail(
                                "rcpt@sender.org".to_string(),
                            )],
                            id,
                            domain: "foobar.net".to_string(),
                            range_from: chrono::Utc::now(),
                            range_to: chrono::Utc::now(),
                            report: crate::pages::queue::reports::test_tls_report(),
                        }));
                    }
                }

                HttpRequest::get("https://127.0.0.1:9980/api/report/status")
                    .with_authorization(&auth)
                    .with_parameter("id", id)
                    .send::<Vec<Option<AggregateReport>>>()
                    .await
                    .map(|result| result.into_iter().next().flatten())
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
                Some(Err(http::Error::NotFound)) | Some(Ok(None)) => {
                    use_navigate()("/manage/queue/reports", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(Some(report))) => {
                    match report {
                        AggregateReport::Tls { report, rua, .. } => {
                            Some(view! { <TlsReportDisplay report=report rua=rua/> }.into_view())
                        }
                        AggregateReport::Dmarc { report, rua, .. } => {
                            Some(view! { <DmarcReportDisplay report=report rua=rua/> }.into_view())
                        }
                    }
                }
            }}

        </Transition>
    }
}
