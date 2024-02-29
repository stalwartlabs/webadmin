use std::{collections::HashSet, sync::Arc};

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        icon::{IconAdd, IconTrash},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ListSection, ListTable, Toolbar, ZeroResults,
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
    pages::{directory::List, maybe_plural},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Domain {
    name: String,
    addresses: u32,
}

const PAGE_SIZE: u32 = 10;

#[component]
pub fn DomainList() -> impl IntoView {
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

    let domains = create_resource(
        move || (page(), filter()),
        move |(page, filter)| {
            let auth = auth.get();

            async move {
                let domain_names = HttpRequest::get("https://127.0.0.1/api/domain")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_optional_parameter("filter", filter)
                    .send::<List<String>>()
                    .await?;
                let mut items = Vec::with_capacity(domain_names.items.len());

                for name in domain_names.items {
                    let records = HttpRequest::get("https://127.0.0.1/api/principal")
                        .with_authorization(&auth)
                        .with_parameter("filter", &name)
                        .send::<List<String>>()
                        .await?;
                    items.push(Domain {
                        name,
                        addresses: records.total as u32,
                    });
                }

                Ok(Arc::new(List {
                    items,
                    total: domain_names.total,
                }))
            }
        },
    );

    let delete_action = create_action(move |items: &Arc<HashSet<String>>| {
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items.iter() {
                if let Err(err) =
                    HttpRequest::delete(format!("https://127.0.0.1/api/domain/{item}"))
                        .with_authorization(&auth)
                        .send::<()>()
                        .await
                {
                    alert.set(Alert::from(err));
                    return;
                }
            }
            domains.refetch();
            alert.set(Alert::success(format!(
                "Deleted {}.",
                maybe_plural(items.len(), "domain", "domains")
            )));
        }
    });

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection>
            <ListTable title="Domain" subtitle="Manage local domain names">
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new("/manage/directory/domains")
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                    <ToolbarButton
                        text=Signal::derive(move || {
                            let ns = selected.get().len();
                            if ns > 0 { format!("Delete ({ns})") } else { "Delete".to_string() }
                        })

                        color=Color::Red
                        on_click=Callback::new(move |_| {
                            let to_delete = selected.get().len();
                            if to_delete > 0 {
                                let text = maybe_plural(to_delete, "domain", "domains");
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
                                                        Arc::new(
                                                            selected.try_update(std::mem::take).unwrap_or_default(),
                                                        ),
                                                    );
                                            }),
                                    )
                            }
                        })
                    >

                        <IconTrash/>
                    </ToolbarButton>

                    <ToolbarButton
                        text=format!("Add {}", "domain")
                        color=Color::Blue
                        on_click=move |_| {
                            use_navigate()("/manage/directory/domain", Default::default());
                        }
                    >

                        <IconAdd size=16 attr:class="flex-shrink-0 size-3"/>
                    </ToolbarButton>

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match domains.get() {
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
                        Some(Ok(domains)) if !domains.items.is_empty() => {
                            total_results.set(Some(domains.total as u32));
                            let domains_ = domains.clone();
                            Some(
                                view! {
                                    <ColumnList
                                        headers=vec!["Name".to_string(), "Accounts".to_string()]

                                        select_all=Callback::new(move |_| {
                                            domains_
                                                .items
                                                .iter()
                                                .map(|p| p.name.to_string())
                                                .collect::<Vec<_>>()
                                        })
                                    >

                                        <For
                                            each=move || domains.items.clone()
                                            key=|domain| domain.name.clone()
                                            let:domain
                                        >
                                            <DomainItem domain/>
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
                                        subtitle="Your search did not yield any results."
                                        button_text=format!("Create a new {}", "domain")

                                        button_action=Callback::new(move |_| {
                                            use_navigate()(
                                                "/manage/directory/domain",
                                                Default::default(),
                                            );
                                        })
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
                                &UrlBuilder::new("/manage/directory/domains")
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
fn DomainItem(domain: Domain) -> impl IntoView {
    let action_url = format!("/manage/directory/accounts?filter={}", domain.name);
    let domain_id = domain.name.clone();

    view! {
        <tr>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 py-3">
                    <label class="flex">
                        <SelectItem item_id=domain_id/>

                        <span class="sr-only">Checkbox</span>
                    </label>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                    <div class="flex items-center gap-x-3">
                        <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                            {domain.name}
                        </span>
                    </div>
                </div>
            </td>

            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-1.5">
                    <a
                        class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        href=action_url
                    >
                        {domain.addresses}
                        accounts
                    </a>
                </div>
            </td>
        </tr>
    }
}
