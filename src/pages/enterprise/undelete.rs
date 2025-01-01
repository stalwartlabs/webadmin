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

use std::sync::Arc;

use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::button::Button,
        icon::{IconArrowUTurnLeft, IconEnvelope},
        list::{
            header::ColumnList, pagination::Pagination, row::SelectItem, toolbar::ToolbarButton,
            Footer, ItemSelection, ListItem, ListSection, ListTable, Toolbar, ZeroResults,
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
    pages::{maybe_plural, FormatDateTime, List},
};

const PAGE_SIZE: u32 = 20;

#[derive(Clone, Serialize, Deserialize, Default)]
struct DeletedBlob {
    pub hash: String,
    pub size: usize,
    #[serde(rename = "deletedAt")]
    pub deleted_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
    pub collection: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct UndeleteRequest {
    hash: String,
    collection: String,
    #[serde(rename = "restoreTime")]
    time: DateTime<Utc>,
    #[serde(rename = "cancelDeletion")]
    #[serde(default)]
    cancel_deletion: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum UndeleteResponse {
    Success,
    NotFound,
    Error { reason: String },
}

#[component]
pub fn UndeleteList() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let page = create_memo(move |_| {
        query
            .with(|q| q.get("page").and_then(|page| page.parse::<u32>().ok()))
            .filter(|&page| page > 0)
            .unwrap_or(1)
    });

    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let selected = create_rw_signal::<ItemSelection>(ItemSelection::None);
    provide_context(selected);

    let blobs = create_resource(
        move || {
            (
                page.get(),
                params.get().get("id").cloned().unwrap_or_default(),
            )
        },
        move |(page, account)| {
            let auth = auth.get_untracked();

            async move {
                HttpRequest::get(("/api/store/undelete", account))
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .send::<List<DeletedBlob>>()
                    .await
                    .map(Arc::new)
            }
        },
    );
    let results = create_rw_signal(Arc::new(List::default()));
    let blob_hash = RwSignal::new(String::new());
    let fetch_headers = RwSignal::new(true);
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

    let total_results = create_rw_signal(None::<u32>);

    let restore_action = create_action(move |items: &Arc<ItemSelection>| {
        let items = items.clone();
        let account = params.get().get("id").cloned().unwrap_or_default();
        let auth = auth.get();
        let results = results.get_untracked();

        async move {
            let response = match items.as_ref() {
                ItemSelection::All => {
                    HttpRequest::post(("/api/store/undelete", account))
                        .with_authorization(&auth)
                        .with_body(())
                        .unwrap()
                        .send::<Vec<UndeleteResponse>>()
                        .await
                }
                ItemSelection::Some(items) => {
                    let mut request = Vec::with_capacity(items.len());
                    for item in items.iter() {
                        if let Some(blob) = results
                            .items
                            .iter()
                            .find(|b: &&DeletedBlob| &b.hash == item)
                        {
                            request.push(UndeleteRequest {
                                hash: blob.hash.clone(),
                                collection: blob.collection.clone(),
                                time: blob.deleted_at,
                                cancel_deletion: blob.expires_at,
                            });
                        }
                    }
                    HttpRequest::post(("/api/store/undelete", account))
                        .with_authorization(&auth)
                        .with_body(request)
                        .unwrap()
                        .send::<Vec<UndeleteResponse>>()
                        .await
                }
                ItemSelection::None => unimplemented!(),
            };

            match response {
                Ok(responses) => {
                    blobs.refetch();
                    let mut success = 0;
                    let mut not_found = 0;
                    let mut errors = Vec::new();

                    for response in responses {
                        match response {
                            UndeleteResponse::Success => {
                                success += 1;
                            }
                            UndeleteResponse::NotFound => {
                                not_found += 1;
                            }
                            UndeleteResponse::Error { reason } => {
                                errors.push(reason);
                            }
                        }
                    }

                    match (success, not_found, errors.len()) {
                        (_, 0, 0) => {
                            alert.set(Alert::success(format!(
                                "Restored {}.",
                                maybe_plural(success, "blob", "blobs")
                            )));
                        }
                        (_, _, 0) => {
                            alert.set(Alert::warning(format!(
                                "Restored {} and {} could not be found.",
                                maybe_plural(success, "blob", "blobs"),
                                maybe_plural(not_found, "blob", "blobs")
                            )));
                        }
                        _ => {
                            alert.set(
                                Alert::warning(format!(
                                    "Restored {}, {} could not be found. Errors were found:",
                                    maybe_plural(success, "blob", "blobs"),
                                    maybe_plural(not_found, "blob", "blobs")
                                ))
                                .with_details_list(errors),
                            );
                        }
                    }
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Show when=move || blob_hash.get().is_empty()>
            <ListSection>
                <ListTable title="Restore deleted blobs" subtitle="View and restore deleted blobs">
                    <Toolbar slot>
                        <ToolbarButton
                            text=Signal::derive(move || {
                                let ns = selected.get().total_selected(total_results.get());
                                if ns > 0 {
                                    format!("Restore ({ns})")
                                } else {
                                    "Restore".to_string()
                                }
                            })

                            color=Color::Red
                            on_click=Callback::new(move |_| {
                                let to_delete = selected.get().total_selected(total_results.get());
                                if to_delete > 0 {
                                    let text = maybe_plural(to_delete, "blob", "blobs");
                                    modal
                                        .set(
                                            Modal::with_title("Confirm restore")
                                                .with_message(
                                                    format!(
                                                        "Are you sure you want to restore {text}? This action cannot be undone.",
                                                    ),
                                                )
                                                .with_button(format!("Restore {text}"))
                                                .with_dangerous_callback(move || {
                                                    restore_action
                                                        .dispatch(
                                                            Arc::new(
                                                                selected.try_update(std::mem::take).unwrap_or_default(),
                                                            ),
                                                        );
                                                }),
                                        )
                                }
                            })
                        >

                            <IconArrowUTurnLeft/>
                        </ToolbarButton>

                    </Toolbar>

                    <Transition fallback=Skeleton>
                        {move || match blobs.get() {
                            None => None,
                            Some(Err(http::Error::Unauthorized)) => {
                                use_navigate()("/login", Default::default());
                                Some(view! { <div></div> }.into_view())
                            }
                            Some(Err(err)) => {
                                results.set(Arc::new(List::default()));
                                alert.set(Alert::from(err));
                                Some(view! { <Skeleton/> }.into_view())
                            }
                            Some(Ok(blobs)) if !blobs.items.is_empty() => {
                                total_results.set(Some(blobs.total as u32));
                                results.set(blobs.clone());
                                Some(
                                    view! {
                                        <ColumnList
                                            headers=vec![
                                                "Type".to_string(),
                                                "Size".to_string(),
                                                "Deleted".to_string(),
                                                "Expires".to_string(),
                                                "".to_string(),
                                            ]

                                            has_select_all=true
                                        >

                                            <For
                                                each=move || blobs.items.clone()
                                                key=|blob| blob.hash.clone()
                                                let:blob
                                            >
                                                <UndeleteItem blob blob_hash/>
                                            </For>

                                        </ColumnList>
                                    }
                                        .into_view(),
                                )
                            }
                            Some(Ok(_)) => {
                                results.set(Arc::new(List::default()));
                                Some(
                                    view! {
                                        <ZeroResults
                                            title="No results found."
                                            subtitle="No deleted blobs were found on this account."
                                            button_text="".to_string()
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
                            total_results=Signal::derive(move || {
                                Some(results.get().total as u32)
                            })

                            page_size=PAGE_SIZE
                            on_page_change=move |page: u32| {
                                use_navigate()(
                                    &UrlBuilder::new("/manage/undelete")
                                        .with_subpath(
                                            params
                                                .get()
                                                .get("id")
                                                .map(|s| s.as_str())
                                                .unwrap_or_default(),
                                        )
                                        .with_parameter("page", page.to_string())
                                        .finish(),
                                    Default::default(),
                                );
                            }
                        />

                    </Footer>
                </ListTable>
            </ListSection>
        </Show>

        <Transition>

            {move || match fetch_contents.get() {
                None | Some(Ok(None)) | Some(Err(http::Error::NotFound)) => None,
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok(Some(message))) => {
                    Some(
                        view! {
                            <div class="max-w-[85rem] px-4 sm:px-6 pb-5 lg:px-8 mx-auto">
                                <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">
                                    <div class="grid gap-3 md:flex md:justify-between md:items-center">
                                        <div>
                                            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200">
                                                {move || {
                                                    if fetch_headers.get() {
                                                        "View Headers"
                                                    } else {
                                                        "View Contents"
                                                    }
                                                }}

                                            </h2>

                                        </div>
                                        <div class="inline-flex gap-x-2">
                                            <Show when=move || fetch_headers.get()>

                                                <Button
                                                    text="View Contents"
                                                    color=Color::Gray
                                                    on_click=move |_| {
                                                        fetch_headers.set(false);
                                                    }
                                                >

                                                    <IconEnvelope/>
                                                </Button>

                                            </Show>
                                            <Button
                                                text="Close"
                                                color=Color::Blue
                                                on_click=move |_| {
                                                    fetch_headers.set(true);
                                                    blob_hash.set(String::new());
                                                }
                                            />

                                        </div>
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

#[component]
fn UndeleteItem(blob: DeletedBlob, blob_hash: RwSignal<String>) -> impl IntoView {
    let blob_id = blob.hash.clone();

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=blob_id/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500">{blob.collection}</span>
            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500">{format_size(blob.size, DECIMAL)}</span>
            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500">{blob.deleted_at.format_date_time()}</span>
            </ListItem>

            <ListItem>
                <span class="text-sm text-gray-500">
                    {HumanTime::from(blob.expires_at).to_string()}
                </span>
            </ListItem>

            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    on:click=move |_| {
                        blob_hash.set(blob.hash.clone());
                    }
                >

                    View
                </a>
            </ListItem>
        </tr>
    }
}
