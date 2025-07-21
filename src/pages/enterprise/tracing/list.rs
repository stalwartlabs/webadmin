/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: LicenseRef-SEL
 *
 * This file is subject to the Stalwart Enterprise License Agreement (SEL) and
 * is not open source software. It must not be modified or distributed without
 * explicit permission from Stalwart Labs LLC.
 * Unauthorized use, modification, or distribution is strictly prohibited.
 */

use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use super::event::{Event, Key};
use crate::components::list::ListTextItem;
use crate::{
    components::{
        list::{
            header::ColumnList, pagination::Pagination, toolbar::SearchBox, Footer, ListItem,
            ListSection, ListTable, Toolbar, ZeroResults,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
    pages::{FormatDateTime, List},
};

const PAGE_SIZE: u32 = 10;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct PageDetails {
    event_type: &'static str,
    title: &'static str,
    subtitle: &'static str,
    url: &'static str,
}

#[component]
pub fn SpanList() -> impl IntoView {
    let params = use_params_map();
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
    let selected_type = create_memo(move |_| {
        match params
            .get()
            .get("object")
            .map(|id| id.as_str())
            .unwrap_or_default()
        {
            "delivery" => PageDetails {
                event_type: "delivery.attempt-start",
                title: "Delivery Attempt History",
                subtitle: "View and search the message delivery history",
                url: "/manage/tracing/delivery",
            },
            _ => PageDetails {
                event_type: "smtp.connection-start",
                title: "Received Messages",
                subtitle: "View and search the received messages history",
                url: "/manage/tracing/received",
            },
        }
    });

    let auth = use_authorization();
    let alert = use_alerts();
    let spans = create_resource(
        move || (page.get(), filter.get()),
        move |(page, filter)| {
            let auth = auth.get_untracked();
            let queue_id = query.get().get("queue_id").cloned();
            let params = selected_type.get();

            async move {
                HttpRequest::get("/api/telemetry/traces")
                    .with_authorization(&auth)
                    .with_parameter("type", params.event_type)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_parameter("values", "1")
                    .with_optional_parameter("filter", filter)
                    .with_optional_parameter("queue_id", queue_id)
                    .send::<List<Event>>()
                    .await
            }
        },
    );

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection>
            <ListTable
                title=Signal::derive(move || selected_type.get().title.to_string())
                subtitle=Signal::derive(move || selected_type.get().subtitle.to_string())
            >
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new(selected_type.get().url)
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match spans.get() {
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
                        Some(Ok(spans)) if !spans.items.is_empty() => {
                            total_results.set(Some(spans.total as u32));
                            Some(
                                view! {
                                    <ColumnList headers=vec![
                                        "Date".to_string(),
                                        "From".to_string(),
                                        "To".to_string(),
                                        "Size".to_string(),
                                        "".to_string(),
                                    ]>

                                        <For
                                            each=move || spans.items.clone()
                                            key=|span| span.id()
                                            let:span
                                        >
                                            <HistoryItem span/>
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
                                        subtitle="No history entries were found with the selected criteria."
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
                                &UrlBuilder::new(selected_type.get().url)
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

#[component]
fn HistoryItem(span: Event) -> impl IntoView {
    let timestamp = span.created_at.format_date_time();
    let from = span.get_as_str(Key::From).unwrap_or_default().to_string();
    let size = format_size(span.get_as_int(Key::Size).unwrap_or_default(), DECIMAL);
    let mut to = String::new();
    let mut to_count = 0;
    for (pos, rcpt) in span.get_as_str_list(Key::To).enumerate() {
        if pos == 0 {
            to.push_str(rcpt);
        } else {
            to_count += 1;
        }
    }
    if to_count > 0 {
        to.push_str(&format!(" +{to_count} more"));
    }
    let view_url = format!(
        "/manage/tracing/span/{}",
        span.get_as_int(Key::SpanId).unwrap_or_default()
    );

    view! {
        <tr>
            <ListTextItem>{timestamp}</ListTextItem>
            <ListTextItem>{from}</ListTextItem>
            <ListTextItem>{to}</ListTextItem>
            <ListTextItem>{size}</ListTextItem>
            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=view_url
                >
                    View
                </a>
            </ListItem>
        </tr>
    }
}
