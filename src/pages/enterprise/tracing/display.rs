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

use std::{collections::HashSet, vec};

use chrono::{DateTime, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};

use crate::{
    components::{
        card::{Card, CardItem},
        form::button::Button,
        icon::{
            IconAlertTriangle, IconChatBubbleBottom, IconClock, IconDocumentChartBar,
            IconExclamationCircle,
        },
        messages::alert::{use_alerts, Alert, Alerts},
        report::ReportView,
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::FormatDateTime,
};

use super::event::{Event, Key, Value};

#[component]
pub fn SpanDisplay() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let fetch_report = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();

            async move {
                HttpRequest::get(("/api/tracing/span", &id))
                    .with_authorization(&auth)
                    .send::<Vec<Event>>()
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
                    use_navigate()("/manage/tracing/delivery", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(events)) => {
                    if events.len() >= 2 {
                        Some(view! { <Span events=events/> }.into_view())
                    } else {
                        log::warn!("Invalid span: {events:?}");
                        use_navigate()("/manage/tracing/delivery", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                }
            }}

        </Transition>
    }
}

#[component]
#[allow(unused_parens)]
fn Span(events: Vec<Event>) -> impl IntoView {
    let span_start = events.first().unwrap().created_at;
    let span_duration = HumanTime::from(events.last().unwrap().created_at - span_start)
        .to_text_en(Accuracy::Precise, Tense::Present);
    let start_date = span_start.format_date();
    let start_time = span_start.format_time();
    let (span_type, back_url) = if events.first().unwrap().typ.starts_with("smtp.") {
        ("Received Message", "/manage/tracing/received")
    } else {
        ("Delivery Attempt", "/manage/tracing/delivery")
    };
    let num_events = (events.len() - 2).to_string();

    view! {
        <Card>
            <CardItem title="Span Type" contents=span_type.to_string()>

                <IconDocumentChartBar attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Received" contents=start_date subcontents=start_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Duration" contents=span_duration>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

            <CardItem title="Events" contents=num_events>

                <IconAlertTriangle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

        </Card>

        <ReportView>
            <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                <div class="sm:col-span-12 pb-10">
                    <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                        Event Timeline
                    </h2>
                </div>

                <div>
                    {events
                        .into_iter()
                        .map(|event| {
                            view! { <EventView event=event span_start=span_start.into()/> }
                        })
                        .collect_view()}
                </div>

            </div>

            <div class="flex justify-end">

                <Button
                    text="Close"
                    color=Color::Blue
                    on_click=move |_| {
                        use_navigate()(back_url, Default::default());
                    }
                />

            </div>
        </ReportView>
    }
}

#[component]
pub fn EventView(event: Event, span_start: Option<DateTime<Utc>>) -> impl IntoView {
    view! {
        <div class="group relative flex gap-x-5">

            <div class="relative group-last:after:hidden after:absolute after:top-8 after:bottom-2 after:start-3 after:w-px after:-translate-x-[0.5px] after:bg-gray-200 dark:after:bg-neutral-700">
                <div class="relative z-10 size-6 flex justify-center items-center">
                    {event.icon()}
                </div>
            </div>

            <div class="grow pb-8 group-last:pb-0">
                <h3 class="mb-1 text-xs text-gray-600 dark:text-neutral-400">
                    {span_start
                        .map(|start| {
                            format!(
                                "{} ({} later)",
                                event.created_at.format_time(),
                                HumanTime::from(event.created_at - start)
                                    .to_text_en(Accuracy::Precise, Tense::Present),
                            )
                        })
                        .unwrap_or_else(|| event.created_at.format_date_time())}

                </h3>

                <p class="font-semibold text-sm text-gray-800 dark:text-neutral-200">
                    {event.text.unwrap_or_default()}
                </p>

                <p class="mt-1 text-sm text-gray-600 dark:text-neutral-400">
                    {event.details.unwrap_or_default()}
                </p>

                <ul class="list-disc ms-6 mt-3 space-y-1.5">

                    {event
                        .data
                        .into_iter()
                        .map(|(k, v)| {
                            view! {
                                <li class="ps-1 text-sm text-gray-600 dark:text-neutral-400">
                                    {format_kv(k, v)}
                                </li>
                            }
                        })
                        .collect_view()}

                </ul>
            </div>

        </div>
    }
}

impl Event {
    pub fn icon(&self) -> View {
        const CLASS: &str = "shrink-0 size-6 text-gray-600 dark:text-neutral-400";
        const SIZE: usize = 24;
        let id = self.typ.as_str();

        if id.ends_with("-start") || id.ends_with("-end") {
            view! { <IconClock size=SIZE attr:class=CLASS/> }
        } else if id.contains("failed")
            || id.contains("error")
            || id.contains("invalid")
            || id.contains("reject")
        {
            view! { <IconExclamationCircle size=SIZE attr:class=CLASS/> }
        } else {
            view! { <IconChatBubbleBottom size=SIZE attr:class=CLASS/> }
        }
    }
}

fn format_kv(key: Key, value: Value) -> View {
    let description = match key {
        Key::AccountName => "Account Name",
        Key::AccountId => "Account ID",
        Key::BlobId => "Blob ID",
        Key::CausedBy => "Caused by",
        Key::ChangeId => "Change ID",
        Key::Code => "Code",
        Key::Collection => "Collection",
        Key::Contents => "Contents",
        Key::Details => "Details",
        Key::DkimFail => "DKIM Failure Count",
        Key::DkimNone => "DKIM None Count",
        Key::DkimPass => "DKIM Pass Count",
        Key::DmarcNone => "DMARC None Count",
        Key::DmarcPass => "DMARC Pass Count",
        Key::DmarcQuarantine => "DMARC Quarantine Count",
        Key::DmarcReject => "DMARC Reject Count",
        Key::DocumentId => "Document ID",
        Key::Domain => "Domain name",
        Key::Due => "Due in",
        Key::Elapsed => "Completed in",
        Key::Expires => "Expires in",
        Key::From => "From",
        Key::Hostname => "Hostname",
        Key::Id => "Id",
        Key::Key => "Key",
        Key::Limit => "Limit",
        Key::ListenerId => "Listener ID",
        Key::LocalIp => "Local IP",
        Key::LocalPort => "Local Port",
        Key::MailboxName => "Mailbox Name",
        Key::MailboxId => "Mailbox ID",
        Key::MessageId => "Message ID",
        Key::NextDsn => "Next DSN at",
        Key::NextRetry => "Next Retry at",
        Key::Path => "Path",
        Key::Policy => "Policy",
        Key::QueueId => "Queue ID",
        Key::RangeFrom => "Range from",
        Key::RangeTo => "Range to",
        Key::Reason => "Reason",
        Key::RemoteIp => "Remote IP",
        Key::RemotePort => "Remote Port",
        Key::ReportId => "Report ID",
        Key::Result => "Result",
        Key::Size => "Size",
        Key::Source => "Source",
        Key::SpanId => "Span ID",
        Key::SpfFail => "SPF Failure Count",
        Key::SpfNone => "SPF None Count",
        Key::SpfPass => "SPF Pass Count",
        Key::Strict => "Strict",
        Key::Tls => "TLS",
        Key::To => "To",
        Key::Total => "Total",
        Key::TotalFailures => "Total Failures",
        Key::TotalSuccesses => "Total Successes",
        Key::Type => "Type",
        Key::Uid => "UID",
        Key::UidNext => "UID Next",
        Key::UidValidity => "UID Validity",
        Key::Url => "URL",
        Key::ValidFrom => "Valid from",
        Key::ValidTo => "Valid to",
        Key::Value => "Value",
        Key::Version => "Version",
    };

    view! {
        {format!("{description} ")}
        {format_value(key, value)}
    }
    .into_view()
}

fn format_value(key: Key, value: Value) -> View {
    match (key, value) {
        (
            Key::ValidFrom
            | Key::ValidTo
            | Key::RangeFrom
            | Key::RangeTo
            | Key::Due
            | Key::NextDsn
            | Key::NextRetry,
            Value::String(v),
        ) => view! {
            <b>
                {DateTime::parse_from_rfc3339(&v)
                    .ok()
                    .map(|d| d.to_utc().format_date_time())
                    .unwrap_or(v)}
            </b>
        }
        .into_view(),
        (_, Value::String(v)) => view! { <b>{v}</b> }.into_view(),
        (Key::Elapsed, Value::Int(v)) => {
            view! { <b>{milliseconds_to_human_readable(v)}</b> }.into_view()
        }
        (Key::QueueId, Value::Int(v)) => {
            view! {
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=format!("/manage/tracing/delivery?queue_id={v}")
                >
                    {v}
                </a>
            }.into_view()
        }
        (Key::SpanId, Value::Int(v)) => {
            view! {
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=format!("/manage/tracing/span/{v}")
                >
                    {v}
                </a>
            }.into_view()
        }
        (Key::Size, Value::Int(v)) => {
            view! { <b>{format_size(v, DECIMAL)}</b> }.into_view()
        }
        (_, Value::Int(v)) => view! { <b>{v}</b> }.into_view(),
        (_, Value::Float(v)) => view! { <b>{v}</b> }.into_view(),
        (Key::Tls, Value::Bool(v)) => {
            view! { <b>{if v { "enabled" } else { "disabled" }}</b> }.into_view()
        }
        (_, Value::Bool(v)) => view! { <b>{v}</b> }.into_view(),
        (_, Value::Event(v)) => {
            view! {
                <b>{v.text.unwrap_or_default()}</b>
                <ul class="list-disc ms-6 mt-3 space-y-1.5">

                    {v
                        .data
                        .into_iter()
                        .map(|(k, v)| {
                            view! {
                                <li class="ps-1 text-sm text-gray-600 dark:text-neutral-400">
                                    {format_kv(k, v)}
                                </li>
                            }
                        })
                        .collect_view()}

                </ul>
            }.into_view()
        }
        (_, Value::Array(v)) => view! {
            <ul class="list-disc ms-6 mt-3 space-y-1.5">

                {v
                    .into_iter()
                    .map(|v| {
                        view! {
                            <li class="ps-1 text-sm text-gray-600 dark:text-neutral-400">
                                {format_value(key, v)}
                            </li>
                        }
                    })
                    .collect_view()}

            </ul>
        }
        .into_view(),
    }
}

fn milliseconds_to_human_readable(mut ms: u64) -> String {
    let days = ms / (24 * 60 * 60 * 1000);
    ms %= 24 * 60 * 60 * 1000;

    let hours = ms / (60 * 60 * 1000);
    ms %= 60 * 60 * 1000;

    let minutes = ms / (60 * 1000);
    ms %= 60 * 1000;

    let seconds = ms / 1000;
    ms %= 1000;

    let mut parts = String::new();

    if days > 0 {
        parts.push_str(&format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        if !parts.is_empty() {
            parts.push_str(", ");
        }
        parts.push_str(&format!(
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        ));
    }
    if minutes > 0 {
        if !parts.is_empty() {
            parts.push_str(if ms + seconds == 0 { " and " } else { ", " });
        }
        parts.push_str(&format!(
            "{} minute{}",
            minutes,
            if minutes > 1 { "s" } else { "" }
        ));
    }
    if seconds > 0 {
        if !parts.is_empty() {
            parts.push_str(if ms == 0 { " and " } else { ", " });
        }
        parts.push_str(&format!(
            "{} second{}",
            seconds,
            if seconds > 1 { "s" } else { "" }
        ));
    }
    if ms > 0 {
        if !parts.is_empty() {
            parts.push_str(" and ");
        }
        parts.push_str(&format!("{} ms", ms));
    }
    if parts.is_empty() {
        parts.push_str("0 ms");
    }

    parts
}
