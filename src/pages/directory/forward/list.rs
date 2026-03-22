/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{collections::HashSet, sync::Arc};

use leptos::*;
use leptos_router::*;

use crate::{
    components::{
        icon::{IconAdd, IconTrash},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ItemSelection, ListItem, ListSection, ListTable, ListTextItem, Toolbar,
            ZeroResults,
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
        AccessToken,
    },
    pages::maybe_plural,
};

use crate::pages::config::{Settings, UpdateSettings};

use super::{build_sieve_script, domain_script_id, ForwardRule};

const PAGE_SIZE: u32 = 10;

#[derive(serde::Deserialize, Default)]
pub struct SettingsList {
    pub items: Settings,
}

pub async fn fetch_all_forwards(auth: &AccessToken) -> Result<Vec<ForwardRule>, http::Error> {
    let raw = HttpRequest::get("/api/settings/list")
        .with_parameter("prefix", "mail.forward")
        .with_authorization(auth)
        .send::<SettingsList>()
        .await?;
    Ok(parse_forward_settings(&raw.items))
}

/// Parse a flat settings map (relative to prefix `mail.forward`) into ForwardRules.
pub fn parse_forward_settings(raw: &Settings) -> Vec<ForwardRule> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, ForwardRule> = BTreeMap::new();

    for (key, value) in raw {
        // keys are relative: "<from_addr>.<field>"
        // field is either "to.N" or "keep-copy"; split on last occurrence
        let (from, rest) = if let Some(pos) = key.rfind(".to.") {
            (&key[..pos], &key[pos + 1..])
        } else if let Some(stripped) = key.strip_suffix(".keep-copy") {
            (stripped, "keep-copy")
        } else {
            continue;
        };
        let from = from.to_string();

        let rule = map.entry(from.clone()).or_insert_with(|| ForwardRule {
            from: from.clone(),
            keep_copy: true,
            ..Default::default()
        });

        if rest.starts_with("to.") {
            rule.to.push(value.clone());
        } else if rest == "keep-copy" {
            rule.keep_copy = value == "true";
        }
    }

    let mut rules: Vec<ForwardRule> = map.into_values().collect();
    for r in &mut rules {
        r.to.sort();
    }
    rules
}

/// Regenerate (or delete) Sieve scripts for each affected domain.
pub async fn rebuild_sieve_for_domains(
    auth: &AccessToken,
    domains: &HashSet<String>,
    all_rules: &[ForwardRule],
) -> Result<(), http::Error> {
    let mut updates: Vec<UpdateSettings> = Vec::new();

    for domain in domains {
        let script_id = domain_script_id(domain);
        let script_key = format!("{script_id}.contents");
        let content = build_sieve_script(domain, all_rules);

        if content.is_empty() {
            updates.push(UpdateSettings::Delete {
                keys: vec![format!("sieve.trusted.scripts.{script_key}")],
            });
        } else {
            updates.push(UpdateSettings::Insert {
                prefix: Some("sieve.trusted.scripts".to_string()),
                values: vec![(script_key, content)],
                assert_empty: false,
            });
        }
    }

    if !updates.is_empty() {
        HttpRequest::post("/api/settings")
            .with_authorization(auth)
            .with_body(updates)?
            .send::<Option<String>>()
            .await?;
    }

    Ok(())
}

#[component]
pub fn ForwardList() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let selected = create_rw_signal::<ItemSelection>(ItemSelection::None);
    provide_context(selected);

    let query = use_query_map();
    let page = create_memo(move |_| {
        query
            .with(|q| q.get("page").and_then(|p| p.parse::<u32>().ok()))
            .filter(|&p| p > 0)
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

    let forwards = create_resource(
        move || (page.get(), filter.get()),
        move |_| {
            let auth = auth.get();
            async move { fetch_all_forwards(&auth).await }
        },
    );

    let total_results = create_rw_signal(None::<u32>);

    let delete_selected = create_action(move |items: &Arc<Vec<String>>| {
        let items = items.clone();
        let auth = auth.get();
        async move {
            let all_rules = fetch_all_forwards(&auth).await?;
            let mut updates: Vec<UpdateSettings> = Vec::new();
            let mut affected: HashSet<String> = HashSet::new();

            for from in items.iter() {
                affected.insert(from.splitn(2, '@').nth(1).unwrap_or("").to_string());
                updates.push(UpdateSettings::Clear {
                    prefix: format!("mail.forward.{from}."),
                    filter: None,
                });
            }

            HttpRequest::post("/api/settings")
                .with_authorization(&auth)
                .with_body(updates)
                ?
                .send::<Option<String>>()
                .await?;

            let remaining: Vec<ForwardRule> =
                all_rules.into_iter().filter(|r| !items.contains(&r.from)).collect();
            rebuild_sieve_for_domains(&auth, &affected, &remaining).await
        }
    });

    let delete_all = create_action(move |_: &()| {
        let auth = auth.get();
        async move {
            let all_rules = fetch_all_forwards(&auth).await?;
            let domains: HashSet<String> =
                all_rules.iter().map(|r| r.domain().to_string()).collect();

            HttpRequest::post("/api/settings")
                .with_authorization(&auth)
                .with_body(vec![UpdateSettings::Clear {
                    prefix: "mail.forward.".to_string(),
                    filter: None,
                }])?
                .send::<Option<String>>()
                .await?;

            rebuild_sieve_for_domains(&auth, &domains, &[]).await
        }
    });

    create_effect(move |_| {
        if let Some(result) = delete_selected.value().get() {
            match result {
                Ok(_) => {
                    forwards.refetch();
                    alert.set(Alert::success("Forward deleted. Sieve script updated."));
                }
                Err(e) => alert.set(Alert::from(e)),
            }
        }
    });

    create_effect(move |_| {
        if let Some(result) = delete_all.value().get() {
            match result {
                Ok(_) => {
                    forwards.refetch();
                    alert.set(Alert::success("All forwards deleted."));
                }
                Err(e) => alert.set(Alert::from(e)),
            }
        }
    });

    view! {
        <ListSection>
            <ListTable
                title="Email Forwards"
                subtitle="Manage email forwarding rules across all hosted domains"
            >
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new("/manage/directory/forwards")
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />
                    <ToolbarButton
                        text="Create"
                        color=Color::Blue
                        on_click=Callback::new(move |_| {
                            use_navigate()(
                                "/manage/directory/forwards/_new_/edit",
                                Default::default(),
                            );
                        })
                    >
                        <IconAdd/>
                    </ToolbarButton>
                    <ToolbarButton
                        text=Signal::derive(move || {
                            let n = selected.get().total_selected(total_results.get());
                            if n > 0 {
                                format!("Delete {}", maybe_plural(n, "forward", "forwards"))
                            } else {
                                "Delete".to_string()
                            }
                        })
                        color=Color::Red
                        on_click=Callback::new(move |_| {
                            let to_delete = selected.get();
                            modal.set(
                                Modal::with_title("Confirm deletion")
                                    .with_message(
                                        "Are you sure? The associated Sieve script will be updated.",
                                    )
                                    .with_button("Delete")
                                    .with_dangerous_callback(move || match &to_delete {
                                        ItemSelection::All => {
                                            delete_all.dispatch(());
                                        }
                                        ItemSelection::Some(items) => {
                                            delete_selected
                                                .dispatch(Arc::new(items.iter().cloned().collect()));
                                        }
                                        ItemSelection::None => {}
                                    }),
                            );
                        })
                    >
                        <IconTrash/>
                    </ToolbarButton>
                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match forwards.get() {
                        None => None,
                        Some(Err(http::Error::Unauthorized)) => {
                            use_navigate()("/login", Default::default());
                            None
                        }
                        Some(Err(err)) => {
                            total_results.set(Some(0));
                            alert.set(Alert::from(err));
                            Some(view! { <ZeroResults
                                title="Error loading forwards"
                                subtitle="Could not load forwarding rules."
                                button_text="Retry"
                                button_action=Callback::new(move |_| { forwards.refetch(); })
                            /> }.into_view())
                        }
                        Some(Ok(mut rules)) => {
                            if let Some(f) = filter.get() {
                                let f_lower = f.to_lowercase();
                                rules.retain(|r| {
                                    r.from.to_lowercase().contains(&f_lower)
                                        || r.to.iter().any(|t| t.to_lowercase().contains(&f_lower))
                                });
                            }
                            let total = rules.len() as u32;
                            total_results.set(Some(total));

                            if total == 0 {
                                return Some(view! {
                                    <ZeroResults
                                        title="No forwards configured"
                                        subtitle="No email forwarding rules found."
                                        button_text="Create the first forward"
                                        button_action=Callback::new(move |_| {
                                            use_navigate()(
                                                "/manage/directory/forwards/_new_/edit",
                                                Default::default(),
                                            );
                                        })
                                    />
                                }.into_view());
                            }

                            let start = ((page.get() - 1) * PAGE_SIZE) as usize;
                            let page_rules: Vec<_> = rules
                                .into_iter()
                                .skip(start)
                                .take(PAGE_SIZE as usize)
                                .collect();

                            Some(view! {
                                <ColumnList
                                    headers=vec![
                                        "From Address".to_string(),
                                        "Forward To".to_string(),
                                        "Keep Copy".to_string(),
                                    ]
                                    has_select_all=true
                                >
                                    <For
                                        each=move || page_rules.clone()
                                        key=|r| r.from.clone()
                                        children=move |rule| {
                                            let from = rule.from.clone();
                                            let from2 = from.clone();
                                            let encoded = from.replace('@', "%40").replace('+', "%2B");
                                            let edit_url = format!(
                                                "/manage/directory/forwards/{encoded}/edit",
                                            );
                                            let to_display = rule.to.join(", ");
                                            let keep = if rule.keep_copy { "Yes" } else { "No" };
                                            view! {
                                                <tr>
                                                    <ListItem>
                                                        <SelectItem item_id=from2/>
                                                    </ListItem>
                                                    <ListTextItem>
                                                        <a
                                                            href=edit_url
                                                            class="text-blue-600 hover:underline dark:text-blue-400"
                                                        >
                                                            {from}
                                                        </a>
                                                    </ListTextItem>
                                                    <ListTextItem>{to_display}</ListTextItem>
                                                    <ListTextItem>{keep}</ListTextItem>
                                                </tr>
                                            }
                                        }
                                    />
                                </ColumnList>
                            }.into_view())
                        }
                    }}
                </Transition>

                <Footer slot>
                    <Pagination
                        current_page=page
                        total_results=total_results.read_only()
                        page_size=PAGE_SIZE
                        on_page_change=move |p: u32| {
                            use_navigate()(
                                &UrlBuilder::new("/manage/directory/forwards")
                                    .with_parameter("page", p.to_string())
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
