use leptos::*;
use leptos_router::*;
use std::collections::HashSet;

use crate::{
    components::{
        badge::Badge,
        icon::{IconCancel, IconEnvelope, IconRefresh, IconShieldCheck},
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
        queue::reports::{AggregateReportId, AggregateReportType},
        FormatDateTime, List,
    },
};

use chrono_humanize::{Accuracy, HumanTime, Tense};

const PAGE_SIZE: u32 = 10;

#[component]
pub fn ReportList() -> impl IntoView {
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

    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
    provide_context(selected);

    let reports = create_resource(
        move || (page(), filter()),
        move |(page, filter)| {
            let auth = auth.get_untracked();

            async move {
                HttpRequest::get("/api/queue/reports")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_optional_parameter("domain", filter)
                    .send::<List<String>>()
                    .await
                    .map(|list| {
                        let mut response = List {
                            items: Vec::with_capacity(list.items.len()),
                            total: list.total,
                        };
                        for item in list.items {
                            if let Some(item) = AggregateReportId::parse(item.clone()) {
                                response.items.push(item);
                            } else {
                                log::warn!("Invalid report id: {item}");
                            }
                        }
                        response
                    })
            }
        },
    );

    let cancel_action = create_action(move |items: &HashSet<String>| {
        let items = items.clone();
        let auth = auth.get();

        async move {
            let mut total_deleted = 0;
            for id in items {
                match HttpRequest::delete(format!("/api/queue/reports/{id}"))
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
                    "Removed {} from queue.",
                    maybe_plural(total_deleted, "report", "reports")
                )));
            }
        }
    });

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection>
            <ListTable
                title="Outgoing Reports"
                subtitle="View or cancel scheduled DMARC and TLS aggregate reports"
            >
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new("/manage/queue/reports")
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                    <ToolbarButton
                        text="Refresh"

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
                            if ns > 0 { format!("Cancel ({ns})") } else { "Cancel".to_string() }
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
                                                    "Are you sure you want to cancel delivery of {text}? This action cannot be undone.",
                                                ),
                                            )
                                            .with_button(format!("Delete {text}"))
                                            .with_dangerous_callback(move || {
                                                cancel_action
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
                            Some(
                                view! {
                                    <ColumnList
                                        headers=vec![
                                            "Domain".to_string(),
                                            "Type".to_string(),
                                            "Scheduled Delivery".to_string(),
                                            "Period".to_string(),
                                            "".to_string(),
                                        ]

                                        select_all=Callback::new(move |_| {
                                            reports_
                                                .items
                                                .iter()
                                                .map(|p| p.id.to_string())
                                                .collect::<Vec<_>>()
                                        })
                                    >

                                        <For
                                            each=move || reports.items.clone()
                                            key=|report| report.id.clone()
                                            let:report
                                        >
                                            <ReportItem report/>
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
                                &UrlBuilder::new("/manage/queue/reports")
                                    .with_parameter("page", page.to_string())
                                    .with_optional_parameter("filter", filter())
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

#[component]
fn ReportItem(report: AggregateReportId) -> impl IntoView {
    let show_url = format!("/manage/queue/report/{}", report.id);

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=report.id.to_string()/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                    <div class="flex items-center gap-x-3">
                        <div class="grow">
                            <span class="block text-sm text-gray-500">{report.domain}</span>
                        </div>
                    </div>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-3">
                    <div class="inline-flex gap-2 p-1">

                        {match report.typ {
                            AggregateReportType::Dmarc => {
                                view! {
                                    <Badge color=Color::Blue>
                                        <IconEnvelope attr:class="flex-shrink-0 size-3"/>
                                        DMARC
                                    </Badge>
                                }
                                    .into_view()
                            }
                            AggregateReportType::Tls => {
                                view! {
                                    <Badge color=Color::Green>
                                        <IconShieldCheck attr:class="flex-shrink-0 size-3"/>
                                        TLS
                                    </Badge>
                                }
                                    .into_view()
                            }
                        }}

                    </div>

                </div>
            </td>

            <ListTextItem>
                {format!("{} ({})", HumanTime::from(report.due), report.due.format_date_time())}

            </ListTextItem>

            <ListTextItem>
                {HumanTime::from(report.due - report.created)
                    .to_text_en(Accuracy::Precise, Tense::Present)}
            </ListTextItem>

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
