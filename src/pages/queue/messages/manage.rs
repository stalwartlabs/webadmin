use std::{collections::HashSet, vec};

use chrono::Utc;
use chrono_humanize::HumanTime;
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};

use crate::{
    components::{
        card::{Card, CardItem},
        icon::{
            IconAlertTriangle, IconBell, IconCancel, IconClock, IconEnvelope, IconId, IconLaunch,
            IconScale,
        },
        list::{
            header::ColumnList, row::SelectItem, toolbar::ToolbarButton, Footer, ListItem,
            ListTable, ListTextItem, Toolbar,
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
        FormatDateTime,
    },
};

#[component]
pub fn QueueManage() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let params = use_params_map();
    let fetch_message = create_resource(
        move || params().get("id").cloned().unwrap_or_default(),
        move |id| {
            let auth = auth.get_untracked();
            let id = id.clone();

            async move {
                HttpRequest::get(format!("/api/queue/messages/{id}"))
                    .with_authorization(&auth)
                    .send::<Message>()
                    .await
            }
        },
    );

    let cancel_action = create_action(move |items: &Vec<String>| {
        let id = params().get("id").cloned().unwrap_or_default();
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items {
                match HttpRequest::delete(format!("/api/queue/messages/{id}"))
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
        let id = params().get("id").cloned().unwrap_or_default();
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items {
                match HttpRequest::patch(format!("/api/queue/messages/{id}"))
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
    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
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
                    let return_path = message.return_path().to_string();
                    let num_recipients = message
                        .domains
                        .iter()
                        .map(|d| d.recipients.len())
                        .sum::<usize>();
                    let num_domains = message.domains.len();
                    let next_retry = message.next_retry();
                    let next_dsn = message.next_dsn();
                    let expires = message.expires();
                    let recipients = message
                        .clone()
                        .domains
                        .into_iter()
                        .flat_map(|d| {
                            d.recipients
                                .into_iter()
                                .map(move |mut r| {
                                    if r.status == Status::Scheduled {
                                        r.status = d.status.clone();
                                    }
                                    (r, d.next_retry)
                                })
                        });
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
                                                let ns = selected.get().len();
                                                if ns > 0 {
                                                    format!("Retry ({ns})")
                                                } else {
                                                    "Retry".to_string()
                                                }
                                            })

                                            color=Color::Gray
                                            on_click=Callback::new(move |_| {
                                                let to_delete = selected.get().len();
                                                if to_delete > 0 {
                                                    let mut domains = Vec::<String>::new();
                                                    for rcpt in selected
                                                        .try_update(std::mem::take)
                                                        .unwrap_or_default()
                                                    {
                                                        if let Some((_, domain)) = rcpt.split_once('@') {
                                                            let domain = domain.to_string();
                                                            if !domains.contains(&domain) {
                                                                domains.push(domain);
                                                            }
                                                        }
                                                    }
                                                    retry_action
                                                        .dispatch(
                                                            if domains.len() != num_domains {
                                                                domains
                                                            } else {
                                                                vec!["".to_string()]
                                                            },
                                                        );
                                                }
                                            })
                                        >

                                            <IconLaunch/>
                                        </ToolbarButton>
                                        <ToolbarButton
                                            text=Signal::derive(move || {
                                                let ns = selected.get().len();
                                                if ns > 0 {
                                                    format!("Cancel ({ns})")
                                                } else {
                                                    "Cancel".to_string()
                                                }
                                            })

                                            color=Color::Red
                                            on_click=Callback::new(move |_| {
                                                let to_delete = selected.get().len();
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
                                                                    let selected = selected
                                                                        .try_update(std::mem::take)
                                                                        .unwrap_or_default();
                                                                    let selected = if selected.len() == num_recipients {
                                                                        vec!["".to_string()]
                                                                    } else {
                                                                        selected.into_iter().collect()
                                                                    };
                                                                    cancel_action.dispatch(selected);
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
                                            "Status".to_string(),
                                            "Server Response".to_string(),
                                            "Next/Last Retry".to_string(),
                                        ]

                                        select_all=Callback::new(move |_| {
                                            message
                                                .domains
                                                .iter()
                                                .flat_map(|d| {
                                                    d.recipients.iter().map(|r| r.address.clone())
                                                })
                                                .collect::<Vec<_>>()
                                        })
                                    >

                                        <For
                                            each=move || { recipients.clone() }

                                            key=|(recipient, _)| recipient.address.clone()
                                            children=move |(recipient, next_retry)| {
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

                                                        <ListItem>{display_status}</ListItem>

                                                        <ListItem class="h-px w-72 whitespace-nowrap">
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
    }
}
