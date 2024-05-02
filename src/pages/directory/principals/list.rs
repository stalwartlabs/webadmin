/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::{collections::HashSet, sync::Arc};

use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::*;

use crate::{
    components::{
        badge::Badge,
        icon::{IconAdd, IconTrash},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ListItem, ListSection, ListTable, ListTextItem, Toolbar, ZeroResults,
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
        directory::{Principal, PrincipalType},
        maybe_plural, List,
    },
};

const PAGE_SIZE: u32 = 10;

#[component]
pub fn PrincipalList() -> impl IntoView {
    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
    provide_context(selected);

    let params = use_params_map();
    let selected_type = create_memo(move |_| {
        selected.set(Default::default());
        match params
            .get()
            .get("object")
            .map(|id| id.as_str())
            .unwrap_or_default()
        {
            "accounts" => PrincipalType::Individual,
            "groups" => PrincipalType::Group,
            "lists" => PrincipalType::List,
            _ => PrincipalType::Individual,
        }
    });

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

    let principals = create_resource(
        move || (page.get(), filter.get()),
        move |(page, filter)| {
            let auth = auth.get_untracked();
            let selected_type = selected_type.get();

            async move {
                let principal_names = HttpRequest::get("/api/principal")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", PAGE_SIZE.to_string())
                    .with_parameter("type", selected_type.id())
                    .with_optional_parameter("filter", filter)
                    .send::<List<String>>()
                    .await?;
                let mut items = Vec::with_capacity(principal_names.items.len());

                for name in principal_names.items {
                    match HttpRequest::get(("/api/principal", &name))
                        .with_authorization(&auth)
                        .send::<Principal>()
                        .await
                    {
                        Ok(principal) => {
                            items.push(principal);
                        }
                        Err(http::Error::NotFound) => {
                            log::debug!("Principal {name} not found.");
                        }
                        Err(err) => return Err(err),
                    }
                }

                Ok(Arc::new(List {
                    items,
                    total: principal_names.total,
                }))
            }
        },
    );

    let delete_action = create_action(move |items: &Arc<HashSet<String>>| {
        let items = items.clone();
        let auth = auth.get();

        async move {
            for item in items.iter() {
                if let Err(err) = HttpRequest::delete(("/api/principal", item))
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
                maybe_plural(
                    items.len(),
                    selected_type.get().item_name(false),
                    selected_type.get().item_name(true)
                )
            )));
        }
    });

    let total_results = create_rw_signal(None::<u32>);
    let title = Signal::derive(move || {
        match selected_type.get() {
            PrincipalType::Individual => "Accounts",
            PrincipalType::Group => "Groups",
            PrincipalType::List => "Mailing Lists",
            _ => unreachable!("Invalid type."),
        }
        .to_string()
    });
    let subtitle = Signal::derive(move || {
        match selected_type.get() {
            PrincipalType::Individual => "Manage user accounts",
            PrincipalType::Group => "Manage groups",
            PrincipalType::List => "Manage mailing lists",
            _ => unreachable!("Invalid type."),
        }
        .to_string()
    });

    view! {
        <ListSection>
            <ListTable title=title subtitle=subtitle>

                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new(
                                        format!(
                                            "/manage/directory/{}",
                                            selected_type.get().resource_name(),
                                        ),
                                    )
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
                                let text = maybe_plural(
                                    to_delete,
                                    selected_type.get().item_name(false),
                                    selected_type.get().item_name(true),
                                );
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
                        text=create_memo(move |_| {
                            format!("Create {}", selected_type.get().item_name(false))
                        })

                        color=Color::Blue
                        on_click=move |_| {
                            use_navigate()(
                                &format!(
                                    "/manage/directory/{}/edit",
                                    selected_type.get().resource_name(),
                                ),
                                Default::default(),
                            );
                        }
                    >

                        <IconAdd size=16 attr:class="flex-shrink-0 size-3"/>
                    </ToolbarButton>

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match principals.get() {
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
                        Some(Ok(principals)) if !principals.items.is_empty() => {
                            total_results.set(Some(principals.total as u32));
                            let principals_ = principals.clone();
                            let headers = match selected_type.get() {
                                PrincipalType::Individual => {
                                    vec![
                                        "Name".to_string(),
                                        "E-mail".to_string(),
                                        "Type".to_string(),
                                        "Quota".to_string(),
                                        "Member of".to_string(),
                                        "".to_string(),
                                    ]
                                }
                                PrincipalType::Group => {
                                    vec![
                                        "Name".to_string(),
                                        "E-mail".to_string(),
                                        "Type".to_string(),
                                        "Members".to_string(),
                                        "Member of".to_string(),
                                        "".to_string(),
                                    ]
                                }
                                PrincipalType::List => {
                                    vec![
                                        "Name".to_string(),
                                        "E-mail".to_string(),
                                        "Type".to_string(),
                                        "Members".to_string(),
                                        "".to_string(),
                                    ]
                                }
                                _ => unreachable!("Invalid type."),
                            };
                            Some(
                                view! {
                                    <ColumnList
                                        headers=headers
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
                                            <PrincipalItem principal selected_type=selected_type.get()/>
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
                                        button_text=format!(
                                            "Create a new {}",
                                            selected_type.get().item_name(false),
                                        )

                                        button_action=Callback::new(move |_| {
                                            use_navigate()(
                                                &format!(
                                                    "/manage/directory/{}/edit",
                                                    selected_type.get().resource_name(),
                                                ),
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
                                &UrlBuilder::new(
                                        format!(
                                            "/manage/directory/{}",
                                            selected_type.get().resource_name(),
                                        ),
                                    )
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
fn PrincipalItem(principal: Principal, selected_type: PrincipalType) -> impl IntoView {
    let name = principal.name.as_deref().unwrap_or("unknown").to_string();
    let display_name = principal
        .description
        .as_deref()
        .or(principal.name.as_deref())
        .unwrap_or_default()
        .to_string();
    let display_letter = display_name
        .chars()
        .next()
        .and_then(|ch| ch.to_uppercase().next())
        .unwrap_or_default();
    let principal_id = principal.name.as_deref().unwrap_or_default().to_string();
    let manage_url = format!(
        "/manage/directory/{}/{}/edit",
        selected_type.resource_name(),
        principal_id
    );
    let num_members = principal.members.len();
    let num_member_of = principal.member_of.len();

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=principal_id/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>
            <ListItem subclass="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                <div class="flex items-center gap-x-3">
                    <span class="inline-flex items-center justify-center size-[38px] rounded-full bg-gray-300 dark:bg-gray-700">
                        <span class="font-medium text-gray-800 leading-none dark:text-gray-200">
                            {display_letter}
                        </span>
                    </span>
                    <div class="grow">
                        <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                            {display_name}
                        </span>
                        <span class="block text-sm text-gray-500">{name}</span>
                    </div>
                </div>
            </ListItem>

            <ListItem class="h-px w-72 whitespace-nowrap">
                <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                    {principal.emails.first().cloned().unwrap_or_default()}
                </span>
                <span class="block text-sm text-gray-500">
                    {maybe_plural(principal.emails.len().saturating_sub(1), "alias", "aliases")}
                </span>
            </ListItem>

            <ListItem>
                <Badge color=match principal.typ.unwrap_or(selected_type) {
                    PrincipalType::Superuser => Color::Yellow,
                    PrincipalType::Individual => Color::Green,
                    PrincipalType::Group => Color::Red,
                    PrincipalType::List => Color::Blue,
                    _ => Color::Red,
                }>

                    {principal.typ.unwrap_or(selected_type).name()}
                </Badge>

            </ListItem>
            <Show when=move || { selected_type == PrincipalType::Individual }>
                <ListTextItem>
                    {match (principal.quota, principal.used_quota) {
                        (Some(quota), Some(used_quota)) if quota > 0 => {
                            format!(
                                "{} ({}%)",
                                format_size(used_quota, DECIMAL),
                                (used_quota as f64 / quota as f64 * 100.0).round() as u8,
                            )
                        }
                        _ => "N/A".to_string(),
                    }}

                </ListTextItem>
            </Show>
            <Show when=move || {
                matches!(selected_type, PrincipalType::List | PrincipalType::Group)
            }>
                <ListTextItem>{maybe_plural(num_members, "member", "members")}</ListTextItem>
            </Show>
            <Show when=move || {
                matches!(selected_type, PrincipalType::Individual | PrincipalType::Group)
            }>
                <ListTextItem>{maybe_plural(num_member_of, "group", "groups")}</ListTextItem>
            </Show>
            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=manage_url
                >
                    Edit
                </a>
            </ListItem>

        </tr>
    }
}
