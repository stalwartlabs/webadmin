/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::vec;

use chrono::Utc;
use chrono_humanize::HumanTime;
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};

use crate::{
    components::{
        card::{Card, CardItem},
        form::button::Button,
        icon::{
            IconAlertTriangle, IconBell, IconCancel, IconClock, IconEnvelope, IconId, IconLaunch,
            IconScale,
        },
        list::{
            header::ColumnList, row::SelectItem, toolbar::ToolbarButton, Footer, ItemSelection,
            ListItem, ListTable, ListTextItem, Toolbar,
        },
        messages::{
            alert::{use_alerts, Alert, Alerts},
            modal::{use_modals, Modal},
        },
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::{
        maybe_plural,
        queue::messages::{Message, Status},
        FormatDateTime, List,
    },
};

// SPDX-SnippetBegin
// SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
// SPDX-License-Identifier: LicenseRef-SEL
#[cfg(feature = "enterprise")]
use crate::pages::enterprise::tracing::event::{Event, Key};
// SPDX-SnippetEnd

#[component]
pub fn QueueManage() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let params = use_params_map();
    let blob_hash = RwSignal::new(String::new());
    let fetch_headers = RwSignal::new(true);
    let fetch_message = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();

            async move {
                HttpRequest::get(("/api/queue/messages", &id))
                    .with_authorization(&auth)
                    .send::<Message>()
                    .await
            }
        },
    );
    // SPDX-SnippetBegin
    // SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
    // SPDX-License-Identifier: LicenseRef-SEL
    #[cfg(feature = "enterprise")]
    let fetch_attempts = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();

            async move {
                HttpRequest::get("/api/telemetry/traces")
                    .with_authorization(&auth)
                    .with_parameter("queue_id", id)
                    .with_parameter("values", "1")
                    .send::<List<Event>>()
                    .await
            }
        },
    );
    // SPDX-SnippetEnd
    let fetch_contents = create_resource(
        move || (blob_hash.get(), fetch_headers.get()),
        move |(blob_hash, fetch_headers)| {
            let auth = auth.get_untracked();
            let blob_hash = blob_hash.clone();

            async move {
                if !blob_hash.is_empty() {
                    HttpRequest::get(("/api/store/blobs", &blob_hash))
                        .with_optional_parameter("limit", fetch_headers.then_some("10240"))
                        .with_authorization(&auth)
                        .send_raw()
                        .await
                        .map(|bytes| {
                            let contents = if fetch_headers {
                                let mut contents = Vec::with_capacity(bytes.len());
                                for byte in bytes {
                                    match byte {
                                        b'\n'
                                            if contents.last().copied().unwrap_or_default()
                                                == b'\n' =>
                                        {
                                            break;
                                        }
                                        b'\r' => {
                                            continue;
                                        }
                                        _ => {}
                                    }
                                    contents.push(byte);
                                }
                                contents
                            } else {
                                bytes
                            };

                            Some(String::from_utf8(contents).unwrap_or_else(|e| {
                                String::from_utf8_lossy(e.as_bytes()).into_owned()
                            }))
                        })
                } else {
                    Ok(None)
                }
            }
        },
    );

    let cancel_action = create_action(move |items: &Vec<String>| {
        let id = params.get().get("id").cloned().unwrap_or_default();
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items {
                match HttpRequest::delete(("/api/queue/messages", &id))
                    .with_authorization(&auth)
                    .with_parameter("filter", item)
                    .send::<bool>()
                    .await
                {
                    Ok(_) | Err(http::Error::NotFound) => {}
                    Err(err) => {
                        alert.set(Alert::from(err));
                        return;
                    }
                }
            }

            fetch_message.refetch();
            alert.set(Alert::success("Successfully requested cancellation."));
        }
    });
    let retry_action = create_action(move |items: &Vec<String>| {
        let id = params.get().get("id").cloned().unwrap_or_default();
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items {
                match HttpRequest::patch(("/api/queue/messages", &id))
                    .with_authorization(&auth)
                    .with_parameter("filter", item)
                    .send::<bool>()
                    .await
                {
                    Ok(_) | Err(http::Error::NotFound) => {}
                    Err(err) => {
                        alert.set(Alert::from(err));
                        return;
                    }
                }
            }

            fetch_message.refetch();
            alert.set(Alert::success("Successfully requested immediate delivery."));
        }
    });
    let selected = create_rw_signal::<ItemSelection>(ItemSelection::None);
    provide_context(selected);

    view! {
        <Alerts/>
        <Transition fallback=Skeleton>

            {move || match fetch_message.get() {
                None => None,
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(http::Error::NotFound)) => {
                    use_navigate()("/manage/queue/messages", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(message)) => {
                    blob_hash.set(message.blob_hash.clone());
                    let return_path = message.return_path().to_string();
                    let num_recipients = message.recipients.len();
                    let next_retry = message.next_retry();
                    let next_dsn = message.next_dsn();
                    let expires = message.expires();
                    let recipients = message.recipients;
                    Some(
                        view! {
                            <Card>
                                <CardItem
                                    title="Envelope"
                                    contents=return_path
                                    subcontents=maybe_plural(
                                        num_recipients,
                                        "recipient",
                                        "recipients",
                                    )
                                >

                                    <IconEnvelope attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem
                                    title="Sent"
                                    contents=HumanTime::from(message.created).to_string()
                                    subcontents=message.created.format_date_time()
                                >

                                    <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem title="Size" contents=format_size(message.size, DECIMAL)>

                                    <IconScale attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem
                                    title="Queue Id"
                                    contents=message.id.to_string()
                                    subcontents=message
                                        .env_id
                                        .as_ref()
                                        .map(|_| "Tracking Id ".to_string())
                                        .unwrap_or_default()
                                    subcontents_bold=message.env_id.unwrap_or_default()
                                >

                                    <IconId attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>

                            </Card>
                            <Card>
                                <CardItem
                                    title="Next Retry"
                                    contents=next_retry
                                        .map(|dt| HumanTime::from(dt).to_string())
                                        .unwrap_or("N/A".to_string())
                                    subcontents=next_retry
                                        .map(|dt| { dt.format_date_time() })
                                        .unwrap_or("N/A".to_string())
                                >

                                    <IconLaunch attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem
                                    title="Next notification"
                                    contents=next_dsn
                                        .map(|dt| HumanTime::from(dt).to_string())
                                        .unwrap_or("N/A".to_string())
                                    subcontents=next_dsn
                                        .map(|dt| { dt.format_date_time() })
                                        .unwrap_or("N/A".to_string())
                                >

                                    <IconBell attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem
                                    title="Last attempt"
                                    contents=expires
                                        .map(|dt| HumanTime::from(dt).to_string())
                                        .unwrap_or("N/A".to_string())
                                    subcontents=expires
                                        .map(|dt| { dt.format_date_time() })
                                        .unwrap_or("N/A".to_string())
                                >

                                    <IconCancel attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>
                                <CardItem
                                    title="Priority"
                                    contents=match message.priority.cmp(&0) {
                                        std::cmp::Ordering::Less => "low",
                                        std::cmp::Ordering::Greater => "high",
                                        std::cmp::Ordering::Equal => "normal",
                                    }
                                >

                                    <IconAlertTriangle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

                                </CardItem>

                            </Card>

                            <div class="max-w-[85rem] px-4 py-8 sm:px-6 lg:px-8 lg:py-10 mx-auto">
                                <ListTable title="Recipients" subtitle="Retry or cancel delivery">
                                    <Toolbar slot>
                                        <ToolbarButton
                                            text=Signal::derive(move || {
                                                let ns = selected
                                                    .get()
                                                    .total_selected((num_recipients as u32).into());
                                                if ns > 0 {
                                                    format!("Retry ({ns})")
                                                } else {
                                                    "Retry".to_string()
                                                }
                                            })

                                            color=Color::Gray
                                            on_click=Callback::new(move |_| {
                                                match selected
                                                    .try_update(std::mem::take)
                                                    .unwrap_or_default()
                                                {
                                                    ItemSelection::All => {
                                                        retry_action.dispatch(vec!["".to_string()]);
                                                    }
                                                    ItemSelection::Some(rcpts) => {
                                                        retry_action.dispatch(rcpts.into_iter().collect());
                                                    }
                                                    ItemSelection::None => {}
                                                }
                                            })
                                        >

                                            <IconLaunch/>
                                        </ToolbarButton>
                                        <ToolbarButton
                                            text=Signal::derive(move || {
                                                let ns = selected
                                                    .get()
                                                    .total_selected((num_recipients as u32).into());
                                                if ns > 0 {
                                                    format!("Cancel ({ns})")
                                                } else {
                                                    "Cancel".to_string()
                                                }
                                            })

                                            color=Color::Red
                                            on_click=Callback::new(move |_| {
                                                let to_delete = selected
                                                    .get()
                                                    .total_selected((num_recipients as u32).into());
                                                if to_delete > 0 {
                                                    let text = maybe_plural(
                                                        to_delete,
                                                        "recipient",
                                                        "recipients",
                                                    );
                                                    modal
                                                        .set(
                                                            Modal::with_title("Confirm cancel")
                                                                .with_message(
                                                                    format!(
                                                                        "Are you sure you want to cancel delivery for {text}? This action cannot be undone.",
                                                                    ),
                                                                )
                                                                .with_button(format!("Cancel for {text}"))
                                                                .with_dangerous_callback(move || {
                                                                    match selected
                                                                        .try_update(std::mem::take)
                                                                        .unwrap_or_default()
                                                                    {
                                                                        ItemSelection::All => {
                                                                            cancel_action.dispatch(vec!["".to_string()]);
                                                                        }
                                                                        ItemSelection::Some(selected) => {
                                                                            let selected = if selected.len() == num_recipients {
                                                                                vec!["".to_string()]
                                                                            } else {
                                                                                selected.into_iter().collect()
                                                                            };
                                                                            cancel_action.dispatch(selected);
                                                                        }
                                                                        ItemSelection::None => {}
                                                                    }
                                                                }),
                                                        )
                                                }
                                            })
                                        >

                                            <IconCancel/>
                                        </ToolbarButton>

                                    </Toolbar>
                                    <ColumnList
                                        headers=vec![
                                            "Recipient".to_string(),
                                            "Queue".to_string(),
                                            "Status".to_string(),
                                            "Server Response".to_string(),
                                            "Next/Last Retry".to_string(),
                                        ]

                                        has_select_all=true
                                    >

                                        <For
                                            each=move || { recipients.clone() }

                                            key=|recipient| recipient.address.clone()
                                            children=move |recipient| {
                                                let item_id = recipient.address.clone();
                                                let mut status_details = recipient
                                                    .status
                                                    .clone()
                                                    .unwrap_message();
                                                let mut status_response = String::new();
                                                if let Some((code, message)) = status_details
                                                    .split_once(", Message: ")
                                                {
                                                    status_response = code.to_string();
                                                    status_details = message.to_string();
                                                }
                                                let next_retry = next_retry
                                                    .map(|dt| {
                                                        if !matches!(recipient.status, Status::Completed(_))
                                                            || dt < Utc::now()
                                                        {
                                                            format!(
                                                                "{} ({})",
                                                                HumanTime::from(dt),
                                                                dt.format_date_time(),
                                                            )
                                                        } else {
                                                            "".to_string()
                                                        }
                                                    })
                                                    .unwrap_or_default();
                                                let display_status = match &recipient.status {
                                                    Status::Completed(_) => {
                                                        Status::Completed("Delivered".into())
                                                    }
                                                    Status::TemporaryFailure(_) => {
                                                        Status::TemporaryFailure("Pending".into())
                                                    }
                                                    Status::PermanentFailure(_) => {
                                                        Status::PermanentFailure("Failed".into())
                                                    }
                                                    Status::Scheduled => {
                                                        Status::TemporaryFailure("Scheduled".into())
                                                    }
                                                };
                                                view! {
                                                    <tr>
                                                        <ListItem>
                                                            <label class="flex">
                                                                <SelectItem item_id=item_id/>

                                                                <span class="sr-only">Checkbox</span>
                                                            </label>
                                                        </ListItem>
                                                        <ListItem subclass="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                                                            <div class="flex items-center gap-x-3">
                                                                <div class="grow">
                                                                    <span class="block text-sm font-normal text-gray-800 dark:text-gray-200">
                                                                        {recipient.address}
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        </ListItem>

                                                        <ListTextItem>{recipient.queue}</ListTextItem>

                                                        <ListItem>{display_status}</ListItem>

                                                        <ListItem class="h-px w-72 text-wrap">
                                                            <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                                                                {status_response}
                                                            </span>
                                                            <span class="block text-sm text-gray-500">
                                                                {status_details}
                                                            </span>

                                                        </ListItem>

                                                        <ListTextItem>{next_retry}</ListTextItem>

                                                    </tr>
                                                }
                                            }
                                        />

                                    </ColumnList>
                                    <Footer slot>
                                        <div></div>
                                    </Footer>

                                </ListTable>
                            </div>
                        }
                            .into_view(),
                    )
                }
            }}

        </Transition>

        // SPDX-SnippetBegin
        // SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
        // SPDX-License-Identifier: LicenseRef-SEL

        <Transition>
            {#[cfg(feature = "enterprise")]
            move || match fetch_attempts.get() {
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(spans)) if !spans.items.is_empty() => {
                    Some(
                        view! {
                            <div class="max-w-[85rem] px-4 sm:px-6 pb-5 lg:px-8 mx-auto">
                                <ListTable
                                    title="History"
                                    subtitle="View previous delivery attempts and message history"
                                >
                                    <Toolbar slot>
                                        <div></div>
                                    </Toolbar>
                                    <ColumnList headers=vec![
                                        "Date".to_string(),
                                        "Event".to_string(),
                                        "".to_string(),
                                    ]>

                                        {spans
                                            .items
                                            .into_iter()
                                            .map(|span| {
                                                let title = if span.typ.starts_with("delivery.") {
                                                    "Delivery Attempt"
                                                } else {
                                                    "Message Received"
                                                }
                                                    .to_string();
                                                view! {
                                                    <tr>
                                                        <ListTextItem>
                                                            {span.created_at.format_date_time()}
                                                        </ListTextItem>
                                                        <ListTextItem>{title}</ListTextItem>
                                                        <ListTextItem>
                                                            <a
                                                                class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                                                href=format!(
                                                                    "/manage/tracing/span/{}",
                                                                    span.get_as_int(Key::SpanId).unwrap_or_default(),
                                                                )
                                                            >

                                                                View
                                                            </a>
                                                        </ListTextItem>

                                                    </tr>
                                                }
                                            })
                                            .collect_view()}

                                    </ColumnList>
                                    <Footer slot>
                                        <div></div>
                                    </Footer>
                                </ListTable>
                            </div>
                        }
                            .into_view(),
                    )
                }
                _ => None,
            }}

        </Transition>
        // SPDX-SnippetEnd

        <Transition>

            {move || match fetch_contents.get() {
                None | Some(Err(http::Error::NotFound)) => None,
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(message)) => {
                    Some(
                        view! {
                            <div class="max-w-[85rem] px-4 sm:px-6 pb-5 lg:px-8 mx-auto">
                                <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">
                                    <div class="grid gap-3 md:flex md:justify-between md:items-center">
                                        <div>
                                            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200">
                                                {move || {
                                                    if fetch_headers.get() { "Headers" } else { "Contents" }
                                                }}

                                            </h2>

                                        </div>
                                        <Show when=move || fetch_headers.get()>
                                            <div class="inline-flex gap-x-2">

                                                <Button
                                                    text="View Contents"
                                                    color=Color::Gray
                                                    on_click=move |_| {
                                                        fetch_headers.set(false);
                                                    }
                                                >

                                                    <IconEnvelope/>
                                                </Button>

                                            </div>
                                        </Show>
                                    </div>

                                    <div
                                        class="pt-5 text-sm text-gray-600 dark:text-gray-400"
                                        style="white-space: pre-wrap;"
                                    >
                                        {message}
                                    </div>

                                </div>
                            </div>
                        }
                            .into_view(),
                    )
                }
            }}

        </Transition>
    }
}
