use leptos::*;
use leptos_router::*;
use std::collections::HashSet;

use crate::{
    components::{
        icon::{IconCancel, IconLaunch, IconRefresh},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ListItem, ListSection, Toolbar, ZeroResults,
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
        directory::List,
        maybe_plural,
        queue::messages::{Message, Status},
    },
};

use chrono_humanize::HumanTime;

const PAGE_SIZE: u32 = 10;

#[component]
pub fn QueueList() -> impl IntoView {
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

    let messages = create_resource(
        move || (page(), filter()),
        move |(page, filter)| {
            let auth = auth.get();

            async move {
                HttpRequest::get("https://127.0.0.1:9980/api/queue/list")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_parameter("values", "")
                    .with_optional_parameter("text", filter)
                    .send::<List<Message>>()
                    .await
            }
        },
    );

    let cancel_action = create_action(move |items: &HashSet<String>| {
        let mut query = String::new();
        for (pos, item) in items.iter().enumerate() {
            if pos > 0 {
                query.push(',');
            }
            query.push_str(item);
        }

        let auth = auth.get();

        async move {
            match HttpRequest::get("https://127.0.0.1:9980/api/queue/cancel")
                .with_authorization(&auth)
                .with_parameter("ids", query)
                .send::<Vec<bool>>()
                .await
            {
                Ok(results) => {
                    messages.refetch();
                    alert.set(Alert::success(format!(
                        "Removed {} from queue.",
                        maybe_plural(
                            results.iter().filter(|&&b| b).count(),
                            "message",
                            "messages"
                        )
                    )));
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });
    let retry_action = create_action(move |items: &HashSet<String>| {
        let mut query = String::new();
        for (pos, item) in items.iter().enumerate() {
            if pos > 0 {
                query.push(',');
            }
            query.push_str(item);
        }

        let auth = auth.get();

        async move {
            match HttpRequest::get("https://127.0.0.1:9980/api/queue/retry")
                .with_authorization(&auth)
                .with_parameter("ids", query)
                .send::<Vec<bool>>()
                .await
            {
                Ok(results) => {
                    messages.refetch();
                    alert.set(Alert::success(format!(
                        "Successfully requested immediate delivery of {}.",
                        maybe_plural(
                            results.iter().filter(|&&b| b).count(),
                            "message",
                            "messages"
                        )
                    )));
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection title="Message Queue" subtitle="View, cancel or reschedule queued messages">
            <Toolbar slot>
                <SearchBox
                    value=filter
                    on_search=move |value| {
                        use_navigate()(
                            &UrlBuilder::new("/manage/queue/messages")
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
                        messages.refetch();
                    })
                >

                    <IconRefresh/>
                </ToolbarButton>

                <ToolbarButton
                    text=Signal::derive(move || {
                        let ns = selected.get().len();
                        if ns > 0 { format!("Retry ({ns})") } else { "Retry".to_string() }
                    })

                    color=Color::Gray
                    on_click=Callback::new(move |_| {
                        let to_delete = selected.get().len();
                        if to_delete > 0 {
                            retry_action
                                .dispatch(selected.try_update(std::mem::take).unwrap_or_default());
                        }
                    })
                >

                    <IconLaunch/>
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
                            let text = maybe_plural(to_delete, "message", "messages");
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
                {move || match messages.get() {
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
                    Some(Ok(messages)) if !messages.items.is_empty() => {
                        total_results.set(Some(messages.total as u32));
                        let messages_ = messages.clone();
                        Some(
                            view! {
                                <ColumnList
                                    headers=vec![
                                        "Envelope".to_string(),
                                        "Status".to_string(),
                                        "Next Retry".to_string(),
                                        "Next DSN".to_string(),
                                        "".to_string(),
                                    ]

                                    select_all=Callback::new(move |_| {
                                        messages_
                                            .items
                                            .iter()
                                            .map(|p| p.id.to_string())
                                            .collect::<Vec<_>>()
                                    })
                                >

                                    <For
                                        each=move || messages.items.clone()
                                        key=|message| message.id
                                        let:message
                                    >
                                        <QueueItem message/>
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
                                    subtitle="No queued messages were found with the selected criteria."
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
                            &UrlBuilder::new("/manage/queue/messages")
                                .with_parameter("page", page.to_string())
                                .with_optional_parameter("filter", filter())
                                .finish(),
                            Default::default(),
                        );
                    }
                />

            </Footer>
        </ListSection>
    }
}

#[component]
fn QueueItem(message: Message) -> impl IntoView {
    let mut total_success = 0;
    let mut total_pending = 0;
    let mut total_failed = 0;
    let mut total_recipients = 0;

    for domain in &message.domains {
        for rcpt in &domain.recipients {
            match &rcpt.status {
                Status::Completed(_) => total_success += 1,
                Status::TemporaryFailure(_) => total_pending += 1,
                Status::PermanentFailure(_) => total_failed += 1,
                Status::Scheduled => match domain.status {
                    Status::Scheduled | Status::TemporaryFailure(_) => total_pending += 1,
                    Status::PermanentFailure(_) => total_failed += 1,
                    _ => {}
                },
            }

            total_recipients += 1;
        }
    }

    let next_retry = message
        .next_retry()
        .map(|dt| HumanTime::from(dt).to_string());
    let next_dsn = message.next_dsn().map(|dt| HumanTime::from(dt).to_string());
    let return_path = message.return_path().to_string();
    let recipients = format!(
        "{} in {}",
        maybe_plural(total_recipients, "recipient", "recipients"),
        maybe_plural(message.domains.len(), "domain", "domains")
    );

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=message.id.to_string()/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                    <div class="flex items-center gap-x-3">
                        <div class="grow">
                            <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                                {return_path}
                            </span>
                            <span class="block text-sm text-gray-500">{recipients}</span>
                        </div>
                    </div>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-3">
                    <div class="inline-flex gap-2 p-1">
                        <Show when=move || { total_success > 0 }>
                            <StatusBadge status=Status::Completed(format!("{total_success} Done"))/>
                        </Show>
                        <Show when=move || { total_pending > 0 }>
                            <StatusBadge status=Status::TemporaryFailure(
                                format!("{total_pending} Pending"),
                            )/>

                        </Show>
                        <Show when=move || { total_failed > 0 }>
                            <StatusBadge status=Status::PermanentFailure(
                                format!("{total_failed} Failed"),
                            )/>

                        </Show>
                    </div>

                </div>
            </td>

            <ListItem>
                <span class="text-sm text-gray-500">{next_retry}</span>
            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500">{next_dsn}</span>
            </ListItem>

            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=format!("/manage/queue/message/{}", message.id)
                >
                    Manage
                </a>
            </ListItem>
        </tr>
    }
}

#[component]
pub fn StatusBadge(status: Status) -> impl IntoView {
    match status {
    Status::Completed(text) => view! {
        <span class="py-1 px-2 inline-flex items-center gap-x-1 text-xs font-medium bg-teal-100 text-teal-800 rounded-full dark:bg-teal-500/10 dark:text-teal-500">
            <svg
                class="flex-shrink-0 size-3"
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"></path>
                <path d="m9 12 2 2 4-4"></path>
            </svg>
            {text}
        </span>
    }.into_view(),
    Status::TemporaryFailure(text) => view! {
        <span class="py-1 px-1.5 inline-flex items-center gap-x-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full dark:bg-blue-500/10 dark:text-blue-500">
            <svg
                class="flex-shrink-0 size-3"
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M5 22h14"></path>
                <path d="M5 2h14"></path>
                <path d="M17 22v-4.172a2 2 0 0 0-.586-1.414L12 12l-4.414 4.414A2 2 0 0 0 7 17.828V22"></path>
                <path d="M7 2v4.172a2 2 0 0 0 .586 1.414L12 12l4.414-4.414A2 2 0 0 0 17 6.172V2"></path>
            </svg>

            {text}
        </span>
    }.into_view(),
    Status::PermanentFailure(text) => view! {
        <span class="py-1 px-1.5 inline-flex items-center gap-x-1 text-xs font-medium bg-red-100 text-red-800 rounded-full dark:bg-red-500/10 dark:text-red-500">
            <svg
                class="flex-shrink-0 size-3"
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path>
                <path d="M12 9v4"></path>
                <path d="M12 17h.01"></path>
            </svg>
            {text}

        </span>
    }.into_view(),
    _ => unreachable!(),
}
}
