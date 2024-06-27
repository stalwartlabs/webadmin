/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use chrono::{DateTime, Utc};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::pages::queue::messages::deserialize_datetime;
use crate::{
    components::{
        badge::Badge,
        list::{
            header::ColumnList, pagination::Pagination, toolbar::SearchBox, Footer, ListItem,
            ListSection, ListTable, Toolbar, ZeroResults,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
    pages::{FormatDateTime, List},
};

const PAGE_SIZE: u32 = 50;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LogEntry {
    #[serde(deserialize_with = "deserialize_datetime")]
    pub timestamp: DateTime<Utc>,
    level: String,
    message: String,
}

#[component]
pub fn Logs() -> impl IntoView {
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
    let logs = create_resource(
        move || (page.get(), filter.get()),
        move |(page, filter)| {
            let auth = auth.get_untracked();

            async move {
                HttpRequest::get("/api/logs")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_optional_parameter("filter", filter)
                    .send::<List<LogEntry>>()
                    .await
            }
        },
    );

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection>
            <ListTable title="Log files" subtitle="View and search log entries">
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new("/manage/logs")
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match logs.get() {
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
                        Some(Ok(logs)) if !logs.items.is_empty() => {
                            total_results.set(Some(logs.total as u32));
                            Some(
                                view! {
                                    <ColumnList headers=vec![
                                        "Date".to_string(),
                                        "Level".to_string(),
                                        "Message".to_string(),
                                    ]>

                                        <For
                                            each=move || logs.items.clone()
                                            key=|log| log.id()
                                            let:log
                                        >
                                            <LogItem log/>
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
                                        subtitle="No log entries were found with the selected criteria."
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
                                &UrlBuilder::new("/manage/logs")
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
fn LogItem(log: LogEntry) -> impl IntoView {
    let timestamp = log.timestamp.format_date_time();

    view! {
        <tr>
            <ListItem>
                <span class="text-sm text-gray-500">{timestamp}</span>
            </ListItem>

            <ListItem>

                {
                    let color = match log.level.as_str() {
                        "ERROR" => Color::Red,
                        "WARN" => Color::Yellow,
                        "INFO" => Color::Green,
                        "DEBUG" => Color::Blue,
                        _ => Color::Gray,
                    };
                    view! { <Badge color=color>{log.level}</Badge> }
                }

            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500 text-wrap">{log.message}</span>
            </ListItem>

        </tr>
    }
}

impl LogEntry {
    pub fn id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.level.hash(&mut hasher);
        self.message.hash(&mut hasher);
        self.timestamp.hash(&mut hasher);
        hasher.finish().to_string()
    }
}
