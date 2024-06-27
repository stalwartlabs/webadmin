/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use leptos::*;
use leptos_router::*;
use std::collections::HashSet;

use super::ReportType;
use crate::{
    components::{
        icon::{IconCancel, IconRefresh},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ListItem, ListSection, ListTable, ListTextItem, Toolbar, ZeroResults,
        },
        messages::{
            alert::{use_alerts, Alert},
            modal::{use_modals, Modal},
        },
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
    pages::{
        maybe_plural,
        queue::reports::{Feedback, Report, TlsReport},
        reports::{IncomingReport, IncomingReportSummary},
        FormatDateTime, List,
    },
};
use chrono_humanize::{Accuracy, HumanTime, Tense};

const PAGE_SIZE: u32 = 10;

#[component]
pub fn IncomingReportList() -> impl IntoView {
    let query = use_query_map();
    let page = create_memo(move |_| {
        query
            .with(|q| q.get("page").and_then(|page| page.parse::<u32>().ok()))
            .filter(|&page| page > 0)
            .unwrap_or(1)
    });
    let filter = create_memo(move |_| {
        query.with(|q| {
            q.get("filter").and_then(|s| {
                let s = s.trim();
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            })
        })
    });
    let params = use_params_map();
    let report_type = create_memo(move |_| {
        match params
            .get()
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

    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
    provide_context(selected);

    let reports = create_resource(
        move || (page.get(), filter.get()),
        move |(page, filter)| {
            let auth = auth.get_untracked();
            let report_type = report_type.get();

            async move {
                let ids = HttpRequest::get(format!("/api/reports/{}", report_type.as_str()))
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_parameter("max-total", "100")
                    .with_optional_parameter("filter", filter)
                    .send::<List<String>>()
                    .await?;
                let mut result = List {
                    items: Vec::with_capacity(ids.items.len()),
                    total: ids.total,
                };

                for id in ids.items {
                    let report = match report_type {
                        ReportType::Dmarc => HttpRequest::get(format!("/api/reports/dmarc/{id}"))
                            .with_authorization(&auth)
                            .try_send::<IncomingReport<Report>>()
                            .await?
                            .map(|report| IncomingReportSummary::dmarc(id, report)),
                        ReportType::Tls => HttpRequest::get(format!("/api/reports/tls/{id}"))
                            .with_authorization(&auth)
                            .try_send::<IncomingReport<TlsReport>>()
                            .await?
                            .map(|report| IncomingReportSummary::tls(id, report)),
                        ReportType::Arf => HttpRequest::get(format!("/api/reports/arf/{id}"))
                            .with_authorization(&auth)
                            .try_send::<IncomingReport<Feedback>>()
                            .await?
                            .map(|report| IncomingReportSummary::arf(id, report)),
                    };
                    if let Some(report) = report {
                        result.items.push(report);
                    }
                }

                Ok(result)
            }
        },
    );

    let delete_action = create_action(move |items: &HashSet<String>| {
        let items = items.clone();
        let auth = auth.get();
        let report_class = report_type.get().as_str();

        async move {
            let mut total_deleted = 0;
            for id in items {
                match HttpRequest::delete(format!("/api/reports/{report_class}/{id}"))
                    .with_authorization(&auth)
                    .send::<bool>()
                    .await
                {
                    Ok(true) => {
                        total_deleted += 1;
                    }
                    Ok(false) | Err(http::Error::NotFound) => {}
                    Err(err) => {
                        alert.set(Alert::from(err));
                        return;
                    }
                }
            }

            if total_deleted > 0 {
                reports.refetch();
                alert.set(Alert::success(format!(
                    "Removed {}.",
                    maybe_plural(total_deleted, "report", "reports")
                )));
            }
        }
    });

    let total_results = create_rw_signal(None::<u32>);

    let title = create_memo(move |_| {
        match report_type.get() {
            ReportType::Dmarc => "DMARC Aggregate Reports",
            ReportType::Tls => "TLS Reports",
            ReportType::Arf => "Failure Reports",
        }
        .to_string()
    });
    let subtitle = create_memo(move |_| {
        match report_type.get() {
            ReportType::Dmarc => "View received DMARC aggregate reports",
            ReportType::Tls => "View received TLS aggregate reports",
            ReportType::Arf => "View received abuse and authentication failure reports",
        }
        .to_string()
    });

    view! {
        <ListSection>
            <ListTable title=title subtitle=subtitle>
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new(
                                        format!("/manage/reports/{}", report_type.get().as_str()),
                                    )
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                    <ToolbarButton
                        text="Reload"

                        color=Color::Gray
                        on_click=Callback::new(move |_| {
                            reports.refetch();
                        })
                    >

                        <IconRefresh/>
                    </ToolbarButton>

                    <ToolbarButton
                        text=Signal::derive(move || {
                            let ns = selected.get().len();
                            if ns > 0 { format!("Delete ({ns})") } else { "Delete".to_string() }
                        })

                        color=Color::Red
                        on_click=Callback::new(move |_| {
                            let to_delete = selected.get().len();
                            if to_delete > 0 {
                                let text = maybe_plural(to_delete, "report", "reports");
                                modal
                                    .set(
                                        Modal::with_title("Confirm deletion")
                                            .with_message(
                                                format!(
                                                    "Are you sure you want to delete {text}? This action cannot be undone.",
                                                ),
                                            )
                                            .with_button(format!("Delete {text}"))
                                            .with_dangerous_callback(move || {
                                                delete_action
                                                    .dispatch(
                                                        selected.try_update(std::mem::take).unwrap_or_default(),
                                                    );
                                            }),
                                    )
                            }
                        })
                    >

                        <IconCancel/>
                    </ToolbarButton>

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match reports.get() {
                        None => None,
                        Some(Err(http::Error::Unauthorized)) => {
                            use_navigate()("/login", Default::default());
                            Some(view! { <div></div> }.into_view())
                        }
                        Some(Err(err)) => {
                            total_results.set(Some(0));
                            alert.set(Alert::from(err));
                            Some(view! { <Skeleton/> }.into_view())
                        }
                        Some(Ok(reports)) if !reports.items.is_empty() => {
                            total_results.set(Some(reports.total as u32));
                            let reports_ = reports.clone();
                            let headers = match report_type.get() {
                                ReportType::Dmarc => {
                                    vec![
                                        "From".to_string(),
                                        "Report Range".to_string(),
                                        "Domain".to_string(),
                                        "Pass".to_string(),
                                        "Reject".to_string(),
                                        "Quarantine".to_string(),
                                        "".to_string(),
                                    ]
                                }
                                ReportType::Tls => {
                                    vec![
                                        "From".to_string(),
                                        "Report Range".to_string(),
                                        "Domains".to_string(),
                                        "Successes".to_string(),
                                        "Failures".to_string(),
                                        "".to_string(),
                                    ]
                                }
                                ReportType::Arf => {
                                    vec![
                                        "From".to_string(),
                                        "Type".to_string(),
                                        "Date".to_string(),
                                        "Domains".to_string(),
                                        "Incidents".to_string(),
                                        "".to_string(),
                                    ]
                                }
                            };
                            Some(
                                view! {
                                    <ColumnList
                                        headers=headers

                                        select_all=Callback::new(move |_| {
                                            reports_
                                                .items
                                                .iter()
                                                .map(|p| p.id().to_string())
                                                .collect::<Vec<_>>()
                                        })
                                    >

                                        <For
                                            each=move || reports.items.clone()
                                            key=|report| report.id().to_string()
                                            let:report
                                        >
                                            <ReportItem report=report/>
                                        </For>

                                    </ColumnList>
                                }
                                    .into_view(),
                            )
                        }
                        Some(Ok(_)) => {
                            total_results.set(Some(0));
                            Some(
                                view! {
                                    <ZeroResults
                                        title="No results"
                                        subtitle="No reports were found with the selected criteria."
                                    />
                                }
                                    .into_view(),
                            )
                        }
                    }}

                </Transition>

                <Footer slot>

                    <Pagination
                        current_page=page
                        total_results=total_results.read_only()
                        page_size=PAGE_SIZE
                        on_page_change=move |page: u32| {
                            use_navigate()(
                                &UrlBuilder::new(
                                        format!("/manage/reports/{}", report_type.get().as_str()),
                                    )
                                    .with_parameter("page", page.to_string())
                                    .with_optional_parameter("filter", filter.get())
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                </Footer>
            </ListTable>
        </ListSection>
    }
}

enum Item {
    Single(String),
    Double((String, String)),
}

#[component]
fn ReportItem(report: IncomingReportSummary) -> impl IntoView {
    let (show_url, item_id, columns) = match report {
        IncomingReportSummary::Dmarc {
            id,
            range_from,
            range_to,
            from,
            domains,
            total_passes,
            total_rejects,
            total_quarantined,
            ..
        } => (
            format!("/manage/reports/dmarc/{id}"),
            id,
            vec![
                Item::Single(from),
                Item::Double((
                    range_from.format_date_time(),
                    format!(
                        "Covering {}",
                        HumanTime::from(range_to - range_from)
                            .to_text_en(Accuracy::Rough, Tense::Present)
                    ),
                )),
                Item::Single(domains.join(", ")),
                Item::Single(total_passes.to_string()),
                Item::Single(total_rejects.to_string()),
                Item::Single(total_quarantined.to_string()),
            ],
        ),
        IncomingReportSummary::Tls {
            id,
            range_from,
            range_to,
            from,
            domains,
            total_success,
            total_failures,
            ..
        } => (
            format!("/manage/reports/tls/{id}"),
            id,
            vec![
                Item::Single(from),
                Item::Double((
                    range_from.format_date_time(),
                    format!(
                        "Covering {}",
                        HumanTime::from(range_to - range_from)
                            .to_text_en(Accuracy::Rough, Tense::Present)
                    ),
                )),
                {
                    if domains.len() > 1 {
                        Item::Double((
                            format!("{} and others", domains.first().unwrap(),),
                            domains.len().to_string(),
                        ))
                    } else {
                        Item::Single(
                            domains
                                .first()
                                .map(|s| s.as_str())
                                .unwrap_or_default()
                                .to_string(),
                        )
                    }
                },
                Item::Single(total_success.to_string()),
                Item::Single(total_failures.to_string()),
            ],
        ),
        IncomingReportSummary::Arf {
            id,
            arrival_date,
            typ,
            from,
            domains,
            total_incidents,
            received,
        } => (
            format!("/manage/reports/arf/{id}"),
            id,
            vec![
                Item::Single(from),
                Item::Single(typ.to_string()),
                Item::Single(arrival_date.unwrap_or(received).format_date_time()),
                {
                    if domains.len() > 1 {
                        Item::Double((
                            format!("{} and others", domains.first().unwrap(),),
                            domains.len().to_string(),
                        ))
                    } else {
                        Item::Single(
                            domains
                                .first()
                                .map(|s| s.as_str())
                                .unwrap_or_default()
                                .to_string(),
                        )
                    }
                },
                Item::Single(total_incidents.to_string()),
            ],
        ),
    };

    let columns = columns
        .into_iter()
        .map(|column| match column {
            Item::Single(value) => view! { <ListTextItem>{value}</ListTextItem> }.into_view(),
            Item::Double((value1, value2)) => {
                view! {
                    <ListItem class="h-px w-72 whitespace-nowrap">
                        <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                            {value1}
                        </span>
                        <span class="block text-sm text-gray-500">{value2}</span>
                    </ListItem>
                }
            }
        })
        .collect_view();

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=item_id/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>

            {columns}

            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=show_url
                >
                    Show
                </a>
            </ListItem>
        </tr>
    }
}
