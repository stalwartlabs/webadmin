/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: LicenseRef-SEL
 *
 * This file is subject to the Stalwart Enterprise License Agreement (SEL) and
 * is not open source software. It must not be modified or distributed without
 * explicit permission from Stalwart Labs Ltd.
 * Unauthorized use, modification, or distribution is strictly prohibited.
 */

use ahash::AHashMap;
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use codee::string::JsonSerdeCodec;
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_chartistry::*;
use leptos_meta::Style;
use leptos_router::{use_navigate, use_params_map};
use leptos_use::{
    use_event_source_with_options, ReconnectLimit, UseEventSourceOptions, UseEventSourceReturn,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        card::{CardSimple, CardSimpleItem},
        icon::{
            IconBuildingOffice, IconClock, IconDocumentChartBar, IconDocumentText,
            IconExclamationCircle, IconExclamationTriangle, IconInboxArrowDown, IconNoSymbol,
            IconPaperAirplane, IconPhone, IconPhoneArrowDown, IconQueueList, IconServer,
            IconShieldExclamation, IconSignal, IconThreeDots, IconTrash, IconUserGroup,
        },
        messages::alert::{use_alerts, Alert, Alerts},
        report::ReportView,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DataPoint {
    x: u64,
    y: [u64; 4],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum Section {
    Overview,
    Network,
    Security,
    Delivery,
    Performance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum Period {
    LastDay,
    Last7Days,
    Last30Days,
    Last90Days,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    Counter {
        id: String,
        timestamp: DateTime<Utc>,
        value: u64,
    },
    Gauge {
        id: String,
        timestamp: DateTime<Utc>,
        value: u64,
    },
    Histogram {
        id: String,
        timestamp: DateTime<Utc>,
        count: u64,
        sum: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum LiveMetric {
    Counter { id: String, value: u64 },
    Gauge { id: String, value: u64 },
    Histogram { id: String, count: u64, sum: u64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
struct MetricSummary {
    sum: u64,
    count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
struct MetricSummaries(AHashMap<String, MetricSummary>);

#[component]
pub fn Dashboard() -> impl IntoView {
    let params = use_params_map();
    let section = create_memo(move |_| {
        match params
            .get()
            .get("object")
            .map(|id| id.as_str())
            .unwrap_or_default()
        {
            "network" => Section::Network,
            "security" => Section::Security,
            "delivery" => Section::Delivery,
            "performance" => Section::Performance,
            _ => Section::Overview,
        }
    });

    let auth = use_authorization();
    let alert = use_alerts();
    let show_period = RwSignal::new(false);
    let period = RwSignal::new(Period::LastDay);
    let history = create_resource(
        move || period.get(),
        move |period| {
            let auth = auth.get_untracked();

            async move {
                HttpRequest::get("/api/telemetry/metrics")
                    .with_authorization(&auth)
                    .with_parameter("after", period.as_timestamp())
                    .send::<Vec<Metric>>()
                    .await
                    .map(|r| (r, period))
            }
        },
    );
    let start_live_telemetry = create_resource(
        || (),
        move |_| {
            let auth = auth.get();

            async move {
                HttpRequest::get("/api/telemetry/live/token")
                    .with_authorization(&auth)
                    .send::<String>()
                    .await
            }
        },
    );

    let summary: RwSignal<MetricSummaries> = RwSignal::new(Default::default());
    let live_summary: RwSignal<MetricSummaries> = RwSignal::new(Default::default());

    let messages_sent_received = RwSignal::new(Vec::<DataPoint>::new());
    let memory_usage = RwSignal::new(Vec::<DataPoint>::new());
    let smtp_connections_in_out = RwSignal::new(Vec::<DataPoint>::new());
    let imap_pop3_connections = RwSignal::new(Vec::<DataPoint>::new());
    let http_connections = RwSignal::new(Vec::<DataPoint>::new());
    let fail_to_ban = RwSignal::new(Vec::<DataPoint>::new());
    let warnings_dmarc_tls = RwSignal::new(Vec::<DataPoint>::new());
    let messages_received = RwSignal::new(Vec::<DataPoint>::new());
    let messages_sent = RwSignal::new(Vec::<DataPoint>::new());
    let delivery_time = RwSignal::new(Vec::<DataPoint>::new());
    let queue_size = RwSignal::new(Vec::<DataPoint>::new());
    let database_performance = RwSignal::new(Vec::<DataPoint>::new());

    create_effect(move |_| match start_live_telemetry.get() {
        Some(Ok(auth_token)) => {
            let url_builer = UrlBuilder::new(format!(
                "{}/api/telemetry/metrics/live/{}",
                auth.get_untracked().base_url,
                auth_token
            ))
            .with_parameter("interval", "30")
            .with_parameter(
                "metrics",
                concat!(
                    "server.memory,",
                    "queue.count,",
                    "user.count,",
                    "domain.count,",
                    "smtp.active-connections,",
                    "imap.active-connections,",
                    "pop3.active-connections,",
                    "http.active-connections,",
                    "delivery.active-connections"
                ),
            );

            let UseEventSourceReturn { data, error, .. } =
                use_event_source_with_options::<Vec<LiveMetric>, JsonSerdeCodec>(
                    &url_builer.finish(),
                    UseEventSourceOptions::default()
                        .reconnect_limit(ReconnectLimit::Limited(5))
                        .reconnect_interval(2000)
                        .named_events(vec!["metrics".to_string()]),
                );

            create_effect(move |_| {
                let mut summary_: AHashMap<String, MetricSummary> = AHashMap::new();
                for metric in data.get().unwrap_or_default() {
                    let (id, sum, count) = match metric {
                        LiveMetric::Counter { id, value } => (id, value, 1),
                        LiveMetric::Gauge { id, value } => (id, value, 1),
                        LiveMetric::Histogram { id, count, sum } => (id, sum, count),
                    };
                    summary_.insert(id, MetricSummary { sum, count });
                }
                live_summary.set(MetricSummaries(summary_));
            });

            create_effect(move |_| {
                error.with(|error| {
                    if let Some(err) = error {
                        alert.set(Alert::error(format!("Live tracing error: {}", err)));
                    }
                });
            });
        }
        Some(Err(http::Error::Unauthorized)) => {
            use_navigate()("/login", Default::default());
        }
        Some(Err(err)) => {
            alert.set(Alert::from(err));
        }
        _ => {}
    });

    create_effect(move |_| match history.get() {
        Some(Ok((metrics, period))) => {
            // Calculate metric summaries
            let mut summary_: AHashMap<String, MetricSummary> = AHashMap::new();
            for metric in metrics.iter() {
                let entry = summary_.entry(metric.id().to_string()).or_default();
                match metric {
                    Metric::Counter { value, .. } => {
                        entry.sum += *value;
                        entry.count += 1;
                    }
                    Metric::Histogram { sum, count, .. } => {
                        entry.sum += *sum;
                        entry.count += *count;
                    }
                    Metric::Gauge { value, .. } => {
                        entry.sum += *value;
                        entry.count += 1;
                    }
                }
            }
            summary.set(MetricSummaries(summary_));

            // Memory usage
            memory_usage.set(
                Bucket::create(period)
                    .add_readings(&metrics, &[&["server.memory"]])
                    .finish_avg(),
            );

            // Messages sent and received
            messages_sent_received.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[
                            &["queue.queue-message"],
                            &[
                                "queue.queue-message-authenticated",
                                "queue.queue-dsn",
                                "queue.queue-report",
                            ],
                        ],
                    )
                    .finish_sum(),
            );

            // SMTP connections
            smtp_connections_in_out.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[&["smtp.connection-start"], &["delivery.attempt-start"]],
                    )
                    .finish_sum(),
            );

            // IMAP & POP3 connections
            imap_pop3_connections.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[&["imap.connection-start"], &["pop3.connection-start"]],
                    )
                    .finish_sum(),
            );

            // HTTP connections
            http_connections.set(
                Bucket::create(period)
                    .add_readings(&metrics, &[&["http.connection-start"]])
                    .finish_sum(),
            );

            // Fail-to-ban
            fail_to_ban.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[
                            &["security.authentication-ban"],
                            &["security.brute-force-ban"],
                            &["security.loiter-ban"],
                            &["security.ip-blocked"],
                        ],
                    )
                    .finish_sum(),
            );

            // DMARC & TLS warnings
            warnings_dmarc_tls.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[
                            &["incoming-report.dmarc-report-with-warnings"],
                            &["incoming-report.tls-report-with-warnings"],
                        ],
                    )
                    .finish_sum(),
            );

            // Messages received
            messages_received.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[&["message-ingest.ham"], &["message-ingest.spam"]],
                    )
                    .finish_sum(),
            );

            // Messages sent
            messages_sent.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[
                            &["queue.queue-message-authenticated"],
                            &["queue.queue-dsn"],
                            &["queue.queue-report"],
                        ],
                    )
                    .finish_sum(),
            );

            // Delivery time
            delivery_time.set(
                Bucket::create(period)
                    .add_readings(&metrics, &[&["delivery.total-time"]])
                    .finish_avg(),
            );

            // Queue size
            queue_size.set(
                Bucket::create(period)
                    .add_readings(&metrics, &[&["queue.count"]])
                    .finish_avg(),
            );

            // Database performance
            database_performance.set(
                Bucket::create(period)
                    .add_readings(
                        &metrics,
                        &[&["message-ingest.time"], &["message-ingest.index-time"]],
                    )
                    .finish_avg(),
            );
        }
        Some(Err(http::Error::Unauthorized)) => {
            use_navigate()("/login", Default::default());
        }
        Some(Err(err)) => {
            alert.set(Alert::from(err));
        }
        _ => {}
    });

    view! {
        <Style>
            "
            .dash-theme ._chartistry_rotated_label {
                font-family: Inter, sans-serif;
            }
            .dash-theme ._chartistry_snippet {
                font-family: Inter, sans-serif;
            }
            .dash-theme ._chartistry_tick_label {
                font-family: Inter, sans-serif;
            }
            "
        </Style>

        <Alerts/>

        <div class="max-w-[85rem] px-4 sm:px-6 lg:px-8 mx-auto flex justify-end">
            <div class="m-1 hs-dropdown [--trigger:hover] relative inline-flex">
                <button
                    id="hs-dropdown-custom-icon-trigger"
                    type="button"
                    class="hs-dropdown-toggle flex justify-center items-center size-9 text-sm font-semibold rounded-lg border border-gray-200 bg-white text-gray-800 shadow-sm hover:bg-gray-50 focus:outline-none focus:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-neutral-900 dark:border-neutral-700 dark:text-white dark:hover:bg-neutral-800 dark:focus:bg-neutral-800"
                    aria-haspopup="menu"
                    aria-expanded="false"
                    aria-label="Dropdown"
                    on:click=move |_| {
                        show_period
                            .update(|v| {
                                *v = !*v;
                            });
                    }
                >

                    <IconThreeDots attr:class="flex-none size-4 text-gray-600 dark:text-neutral-500"/>
                </button>

                <div
                    class=move || {
                        if show_period.get() {
                            "hs-dropdown-menu transition-[opacity,margin] absolute top-full right-0 duration opacity-100 open block min-w-60 bg-white shadow-md rounded-lg p-1 space-y-0.5 mt-2 z-50 dark:bg-neutral-800 dark:border dark:border-neutral-700 dark:divide-neutral-700 after:h-4 after:absolute after:-bottom-4 after:start-0 after:w-full before:h-4 before:absolute before:-top-4 before:start-0 before:w-full"
                        } else {
                            "hs-dropdown-menu transition-[opacity,margin] duration hs-dropdown-open:opacity-100 opacity-0 hidden min-w-60 bg-white shadow-md rounded-lg p-1 space-y-0.5 mt-2 dark:bg-neutral-800 dark:border dark:border-neutral-700 dark:divide-neutral-700 after:h-4 after:absolute after:-bottom-4 after:start-0 after:w-full before:h-4 before:absolute before:-top-4 before:start-0 before:w-full"
                        }
                    }

                    role="menu"
                    aria-orientation="vertical"
                    aria-labelledby="hs-dropdown-hover-event"
                >
                    <a
                        class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700"
                        href="#"
                        on:click=move |_| {
                            period.set(Period::LastDay);
                            show_period.set(false);
                        }
                    >

                        Last 24 hours
                    </a>
                    <a
                        class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700"
                        href="#"
                        on:click=move |_| {
                            period.set(Period::Last7Days);
                            show_period.set(false);
                        }
                    >

                        Last 7 days
                    </a>
                    <a
                        class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700"
                        href="#"
                        on:click=move |_| {
                            period.set(Period::Last30Days);
                            show_period.set(false);
                        }
                    >

                        Last 30 days
                    </a>
                    <a
                        class="flex items-center gap-x-3.5 py-2 px-3 rounded-lg text-sm text-gray-800 hover:bg-gray-100 focus:outline-none focus:bg-gray-100 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-300 dark:focus:bg-neutral-700"
                        href="#"
                        on:click=move |_| {
                            period.set(Period::Last90Days);
                            show_period.set(false);
                        }
                    >

                        Last 90 days
                    </a>
                </div>
            </div>
        </div>
        <Show when=move || { section.get() == Section::Overview }>
            <CardSimple>
                <CardSimpleItem
                    title="Total Users"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["user.count"]).to_string()
                    })
                >

                    <IconUserGroup attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Total Domains"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["domain.count"]).to_string()
                    })
                >

                    <IconBuildingOffice attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Server Memory"
                    contents=Signal::derive(move || {
                        format_size(live_summary.get().sum(&["server.memory"]), DECIMAL)
                    })
                >

                    <IconServer attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>
            <CardSimple>
                <CardSimpleItem
                    title="Received Messages"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["queue.queue-message"]).to_string()
                    })
                >

                    <IconInboxArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Sent Messages"
                    contents=Signal::derive(move || {
                        summary
                            .get()
                            .sum(
                                &[
                                    "queue.queue-message-authenticated",
                                    "queue.queue-dsn",
                                    "queue.queue-report",
                                ],
                            )
                            .to_string()
                    })
                >

                    <IconPaperAirplane attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Pending Messages"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["queue.count"]).to_string()
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <DashboardChart
                title="Messages"
                labels=&["sent", "received"]
                data=messages_sent_received
            />
            <DashboardChart title="Memory usage" labels=&["bytes"] data=memory_usage/>
        </Show>
        <Show when=move || { section.get() == Section::Network }>
            <CardSimple>
                <CardSimpleItem
                    title="SMTP active"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["smtp.active-connections"]).to_string()
                    })
                >

                    <IconPhoneArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="IMAP active"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["imap.active-connections"]).to_string()
                    })
                >

                    <IconPhoneArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="POP3 active"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["pop3.active-connections"]).to_string()
                    })
                >

                    <IconPhoneArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="HTTP active"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["http.active-connections"]).to_string()
                    })
                >

                    <IconPhoneArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>
            <CardSimple>
                <CardSimpleItem
                    title="SMTP total"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["smtp.connection-start"]).to_string()
                    })
                >

                    <IconPhone attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="IMAP total"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["imap.connection-start"]).to_string()
                    })
                >

                    <IconPhone attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="POP3 total"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["pop3.connection-start"]).to_string()
                    })
                >

                    <IconPhone attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="HTTP total"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["http.connection-start"]).to_string()
                    })
                >

                    <IconPhone attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <DashboardChart
                title="SMTP Connections"
                labels=&["inbound", "outbound"]
                data=smtp_connections_in_out
            />
            <DashboardChart
                title="IMAP & POP3 Connections"
                labels=&["IMAP", "POP3"]
                data=imap_pop3_connections
            />
            <DashboardChart title="HTTP Connections" labels=&["total"] data=http_connections/>
        </Show>
        <Show when=move || { section.get() == Section::Security }>
            <CardSimple>
                <CardSimpleItem
                    title="Threats blocked"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["security.ip-blocked"]).to_string()
                    })
                >

                    <IconNoSymbol attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="IPs banned"
                    contents=Signal::derive(move || {
                        summary
                            .get()
                            .sum(
                                &[
                                    "security.authentication-ban",
                                    "security.brute-force-ban",
                                    "security.loiter-ban",
                                ],
                            )
                            .to_string()
                    })
                >

                    <IconShieldExclamation attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Auth Failures"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["auth.failed"]).to_string()
                    })
                >

                    <IconExclamationCircle attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>
            <CardSimple>
                <CardSimpleItem
                    title="Spam blocked"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["message-ingest.spam"]).to_string()
                    })
                >

                    <IconTrash attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="DMARC Warnings"
                    contents=Signal::derive(move || {
                        summary
                            .get()
                            .sum(&["incoming-report.dmarc-report-with-warnings"])
                            .to_string()
                    })
                >

                    <IconExclamationTriangle attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="TLS Warnings"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["incoming-report.tls-report-with-warnings"]).to_string()
                    })
                >

                    <IconExclamationTriangle attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <DashboardChart
                title="Fail-to-ban"
                labels=&["auth bans", "brute bans", "loiter bans", "blocked"]
                data=fail_to_ban
            />
            <DashboardChart title="Warnings" labels=&["DMARC", "TLS"] data=warnings_dmarc_tls/>
        </Show>

        <Show when=move || { section.get() == Section::Delivery }>
            <CardSimple>
                <CardSimpleItem
                    title="Queued Messages"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["queue.count"]).to_string()
                    })
                >

                    <IconQueueList attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Active Sessions"
                    contents=Signal::derive(move || {
                        live_summary.get().sum(&["delivery.active-connections"]).to_string()
                    })
                >

                    <IconSignal attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Session Time"
                    contents=Signal::derive(move || {
                        duration(summary.get().average(&["delivery.attempt-time"]))
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Delivery Time"
                    contents=Signal::derive(move || {
                        duration(summary.get().average(&["delivery.total-time"]))
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <CardSimple>
                <CardSimpleItem
                    title="Messages Received"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["queue.queue-message"]).to_string()
                    })
                >

                    <IconInboxArrowDown attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Messages Sent"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["queue.queue-message-authenticated"]).to_string()
                    })
                >

                    <IconPaperAirplane attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="DSN Sent"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["queue.queue-dsn"]).to_string()
                    })
                >

                    <IconDocumentText attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Reports Sent"
                    contents=Signal::derive(move || {
                        summary.get().sum(&["queue.queue-report"]).to_string()
                    })
                >

                    <IconDocumentChartBar attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <DashboardChart
                title="Messages Received"
                labels=&["ham", "spam"]
                data=messages_received
            />
            <DashboardChart
                title="Messages Sent"
                labels=&["messages", "DSNs", "reports"]
                data=messages_sent
            />
            <DashboardChart title="Delivery time" labels=&["seconds"] data=delivery_time/>
            <DashboardChart title="Queue size" labels=&["messages"] data=queue_size/>

        </Show>

        <Show when=move || { section.get() == Section::Performance }>
            <CardSimple>
                <CardSimpleItem
                    title="Ingestion Time"
                    contents=Signal::derive(move || {
                        duration(summary.get().average(&["message-ingest.time"]))
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Indexing Time"
                    contents=Signal::derive(move || {
                        duration(summary.get().average(&["message-ingest.index-time"]))
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="DNS Lookup Time"
                    contents=Signal::derive(move || {
                        duration(summary.get().average(&["dns.lookup-time"]))
                    })
                >

                    <IconClock attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
                <CardSimpleItem
                    title="Server Memory"
                    contents=Signal::derive(move || {
                        format_size(live_summary.get().sum(&["server.memory"]), DECIMAL)
                    })
                >

                    <IconServer attr:class="shrink-0 size-5 text-gray-600 dark:text-neutral-400"/>
                </CardSimpleItem>
            </CardSimple>

            <DashboardChart
                title="Database performance"
                labels=&["ingest", "index"]
                data=database_performance
            />
            <DashboardChart title="Memory usage" labels=&["bytes"] data=memory_usage/>
        </Show>
    }
}

#[component]
fn DashboardChart(
    title: impl Into<String> + 'static,
    labels: &'static [&'static str],
    data: RwSignal<Vec<DataPoint>>,
) -> impl IntoView {
    let mut series = Series::new(|data: &DataPoint| data.x as f64);
    for (num, label) in labels.iter().enumerate() {
        series = series.line(
            Line::new(move |data: &DataPoint| data.y[num] as f64).with_name(label.to_string()),
        );
    }

    view! {
        <ReportView>

            <div class="dash-theme gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                <Chart
                    aspect_ratio=AspectRatio::from_env_width(300.0)
                    debug=false
                    series=series
                    data=data
                    top=RotatedLabel::middle(title)
                    left=TickLabels::aligned_floats()
                    bottom=Legend::end()
                    inner=[
                        AxisMarker::left_edge().into_inner(),
                        AxisMarker::bottom_edge().into_inner(),
                        XGridLine::default().into_inner(),
                        YGridLine::default().into_inner(),
                        YGuideLine::over_mouse().into_inner(),
                        XGuideLine::over_data().into_inner(),
                    ]

                    tooltip=Tooltip::left_cursor().show_x_ticks(false)
                />
            </div>
        </ReportView>
    }
}

struct Bucket {
    period: Period,
    value: Vec<DataPoint>,
    count: Vec<DataPoint>,
}

impl Bucket {
    fn create(period: Period) -> Self {
        let mut bucket = match period {
            Period::LastDay => vec![DataPoint::default(); 24],
            Period::Last7Days => vec![DataPoint::default(); 7],
            Period::Last30Days => vec![DataPoint::default(); 30],
            Period::Last90Days => vec![DataPoint::default(); 90],
        };

        for (x, item) in bucket.iter_mut().enumerate() {
            item.x = x as u64;
        }

        Bucket {
            period,
            value: bucket.clone(),
            count: bucket,
        }
    }

    fn add_readings(mut self, metrics: &[Metric], ids: &[&[&str]]) -> Self {
        for metric in metrics {
            'outer: for (idx, ids) in ids.iter().enumerate() {
                for id in ids.iter() {
                    if metric.id() == *id {
                        self.add_reading(metric, idx);
                        break 'outer;
                    }
                }
            }
        }
        self
    }

    fn add_reading(&mut self, metric: &Metric, y_num: usize) {
        let (value, count, timestamp) = match metric {
            Metric::Counter {
                value, timestamp, ..
            }
            | Metric::Gauge {
                value, timestamp, ..
            } => (*value, 1u64, timestamp),
            Metric::Histogram {
                sum,
                count,
                timestamp,
                ..
            } => (*sum, *count, timestamp),
        };

        // Calculate bucket index relative to current time
        let diff = Utc::now() - *timestamp;
        let num_hours = diff.num_hours() as usize;
        let num_days = diff.num_days() as usize;
        let index = match self.period {
            Period::LastDay if num_hours < 24 => 23 - num_hours,
            Period::Last7Days if num_days < 7 => 6 - num_days,
            Period::Last30Days if num_days < 30 => 29 - num_days,
            Period::Last90Days if num_days < 90 => 89 - num_days,
            _ => return,
        };

        self.value[index].y[y_num] += value;
        self.count[index].y[y_num] += count;
    }

    fn finish_sum(self) -> Vec<DataPoint> {
        self.value
    }

    fn finish_avg(mut self) -> Vec<DataPoint> {
        for (value, count) in self.value.iter_mut().zip(self.count.iter()) {
            if count.y[0] > 0 {
                value.y[0] /= count.y[0];
            }
            if count.y[1] > 0 {
                value.y[1] /= count.y[1];
            }
        }
        self.value
    }
}

impl Period {
    fn as_timestamp(&self) -> String {
        let diff = match self {
            Period::LastDay => Duration::days(1),
            Period::Last7Days => Duration::days(7),
            Period::Last30Days => Duration::days(30),
            Period::Last90Days => Duration::days(90),
        };

        (Utc::now() - diff - Duration::hours(1) - Duration::seconds(15))
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

impl Metric {
    fn id(&self) -> &str {
        match self {
            Metric::Counter { id, .. } => id,
            Metric::Histogram { id, .. } => id,
            Metric::Gauge { id, .. } => id,
        }
    }
}

impl MetricSummaries {
    fn average(&self, ids: &[&str]) -> u64 {
        let mut sum = 0;
        let mut count = 0;

        for id in ids {
            if let Some(metric) = self.0.get(*id) {
                sum += metric.sum;
                count += metric.count;
            }
        }

        if count > 0 {
            sum / count
        } else {
            0
        }
    }

    fn sum(&self, ids: &[&str]) -> u64 {
        let mut sum = 0;

        for id in ids {
            if let Some(metric) = self.0.get(*id) {
                sum += metric.sum;
            }
        }

        sum
    }
}

fn duration(time: u64) -> String {
    HumanTime::from(Duration::from_std(std::time::Duration::from_millis(time)).unwrap_or_default())
        .to_text_en(Accuracy::Precise, Tense::Present)
}
