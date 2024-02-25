use std::{collections::HashSet, sync::Arc};

use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::*;

use crate::{
    components::{
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{ButtonIcon, SearchBox, ToolbarButton},
            Footer, ListSection, Toolbar, ZeroResults,
        },
        messages::{
            alert::{use_alerts, Alert},
            modal::{use_modals, Modal},
        },
        skeleton::Skeleton,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
    pages::{
        directory::{List, Principal, Type},
        maybe_plural,
    },
};

const PAGE_SIZE: u32 = 10;

#[component]
pub fn AccountList() -> impl IntoView {
    let query = use_query_map();
    let page = create_memo(move |_| {
        query
            .with(|q| q.get("page").and_then(|page| page.parse::<u32>().ok()))
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

    let principals = create_resource(
        move || (page(), filter()),
        move |(page, filter)| {
            let auth = auth.get();

            async move {
                let account_names = HttpRequest::get("https://127.0.0.1/api/principal")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_parameter("type", "individual")
                    .with_optional_parameter("filter", filter)
                    .send::<List<String>>()
                    .await?;
                let mut items = Vec::with_capacity(account_names.items.len());

                for name in account_names.items {
                    items.push(
                        HttpRequest::get(format!("https://127.0.0.1/api/principal/{}", name))
                            .with_authorization(&auth)
                            .send::<Principal>()
                            .await?,
                    );
                }

                Ok(Arc::new(List {
                    items,
                    total: account_names.total,
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
                    HttpRequest::delete(format!("https://127.0.0.1/api/principal/{item}"))
                        .with_authorization(&auth)
                        .send::<()>()
                        .await
                {
                    alert.set(Alert::from(err));
                    return;
                }
            }
            principals.refetch();
            alert.set(Alert::success(format!(
                "Deleted {}.",
                maybe_plural(items.len(), "account", "accounts")
            )));
        }
    });

    let total_results = create_rw_signal(None::<u32>);

    view! {
        <ListSection title="Accounts" subtitle="Manage user accounts">
            <Toolbar slot>
                <SearchBox
                    value=filter
                    on_search=move |value| {
                        use_navigate()(
                            &UrlBuilder::new("/manage/accounts")
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

                    icon=ButtonIcon::Delete
                    on_click=Callback::new(move |_| {
                        let to_delete = selected.get().len();
                        if to_delete > 0 {
                            let text = maybe_plural(to_delete, "account", "accounts");
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
                />

                <ToolbarButton
                    text="Add user"
                    icon=ButtonIcon::Add
                    on_click=move |_| {
                        use_navigate()("/manage/account", Default::default());
                    }
                />

            </Toolbar>

            <Transition fallback=Skeleton>
                {move || match principals.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <Skeleton/> }.into_view())
                    }
                    Some(Ok(principals)) if !principals.items.is_empty() => {
                        total_results.set(Some(principals.total as u32));
                        let principals_ = principals.clone();
                        Some(
                            view! {
                                <ColumnList
                                    headers=vec![
                                        "Name".to_string(),
                                        "E-mail".to_string(),
                                        "Type".to_string(),
                                        "Quota".to_string(),
                                        "Member of".to_string(),
                                        "".to_string(),
                                    ]

                                    select_all=Callback::new(move |_| {
                                        principals_
                                            .items
                                            .iter()
                                            .map(|p| p.name.as_deref().unwrap_or_default().to_string())
                                            .collect::<Vec<_>>()
                                    })
                                >

                                    <For
                                        each=move || principals.items.clone()
                                        key=|principal| principal.name.clone().unwrap_or_default()
                                        let:principal
                                    >
                                        <AccountItem principal/>
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
                                    button_text="Create a new account"
                                    button_action=Callback::new(move |_| {
                                        use_navigate()("/manage/account", Default::default());
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
                            &UrlBuilder::new("/manage/accounts")
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
pub fn AccountItem(principal: Principal) -> impl IntoView {
    let display_name = principal
        .description
        .as_deref()
        .or(principal.name.as_deref())
        .unwrap_or_default();
    let pct_quota = match (principal.quota, principal.used_quota) {
        (Some(quota), Some(used_quota)) if quota > 0 => format!(
            "{} ({}%)",
            format_size(used_quota, DECIMAL),
            (used_quota as f64 / quota as f64 * 100.0).round() as u8
        ),
        _ => "N/A".to_string(),
    };
    let principal_id = principal.name.as_deref().unwrap_or_default().to_string();
    let manage_url = format!("/manage/account/{}", principal_id);

    view! {
        <tr>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 py-3">
                    <label class="flex">
                        <SelectItem item_id=principal_id/>

                        <span class="sr-only">Checkbox</span>
                    </label>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                    <div class="flex items-center gap-x-3">
                        <span class="inline-flex items-center justify-center size-[38px] rounded-full bg-gray-300 dark:bg-gray-700">
                            <span class="font-medium text-gray-800 leading-none dark:text-gray-200">
                                {display_name
                                    .chars()
                                    .next()
                                    .and_then(|ch| ch.to_uppercase().next())
                                    .unwrap_or_default()}
                            </span>
                        </span>
                        <div class="grow">
                            <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                                {display_name.to_string()}
                            </span>
                            <span class="block text-sm text-gray-500">
                                {principal.name.as_deref().unwrap_or("unknown").to_string()}
                            </span>
                        </div>
                    </div>
                </div>
            </td>
            <td class="h-px w-72 whitespace-nowrap">
                <div class="px-6 py-3">
                    <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                        {principal.emails.first().cloned().unwrap_or_default()}
                    </span>
                    <span class="block text-sm text-gray-500">
                        {maybe_plural(principal.emails.len(), "address", "addresses")}
                    </span>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-3">
                    {if matches!(principal.typ, Some(Type::Superuser)) {
                        view! {
                            <span class="py-1 px-1.5 inline-flex items-center gap-x-1 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full dark:bg-yellow-500/10 dark:text-yellow-500">
                                Admin
                            </span>
                        }
                    } else {
                        view! {
                            <span class="py-1 px-1.5 inline-flex items-center gap-x-1 text-xs font-medium bg-teal-100 text-teal-800 rounded-full dark:bg-teal-500/10 dark:text-teal-500">
                                Individual
                            </span>
                        }
                    }}

                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-3">
                    <span class="text-sm text-gray-500">{pct_quota}</span>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-3">
                    <span class="text-sm text-gray-500">
                        {maybe_plural(principal.member_of.len(), "group", "groups")}
                    </span>
                </div>
            </td>
            <td class="size-px whitespace-nowrap">
                <div class="px-6 py-1.5">
                    <a
                        class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        href=manage_url
                    >
                        Edit
                    </a>
                </div>
            </td>
        </tr>
    }
}
