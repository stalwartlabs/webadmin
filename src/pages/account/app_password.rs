/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, vec};

use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use leptos::*;
use leptos_router::{use_navigate, use_query_map};
use pwhash::sha512_crypt;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button, input::InputText, Form, FormButtonBar, FormElement, FormItem,
            FormSection,
        },
        icon::{IconAdd, IconTrash},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
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
        schema::{Builder, Schemas, Transformer, Type, Validator},
        url::UrlBuilder,
    },
    pages::{
        account::{AccountAuthRequest, AccountAuthResponse},
        maybe_plural, List,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppPassword {
    id: String,
    name: String,
    created: Option<DateTime<Utc>>,
}

const PAGE_SIZE: u32 = 10;

#[component]
pub fn AppPasswords() -> impl IntoView {
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
    let selected = create_rw_signal::<ItemSelection>(ItemSelection::None);
    provide_context(selected);

    let passwords = create_resource(
        move || (page.get(), filter.get()),
        move |(page, filter)| {
            let auth = auth.get_untracked();

            async move {
                let response = HttpRequest::get("/api/account/auth")
                    .with_authorization(&auth)
                    .send::<AccountAuthResponse>()
                    .await?;
                let mut items = Vec::with_capacity(response.app_passwords.len());
                let mut offset = PAGE_SIZE * page.saturating_sub(1);
                let total = response.app_passwords.len() as u64;

                for id in response.app_passwords {
                    let mut app_password = AppPassword {
                        id,
                        name: String::new(),
                        created: None,
                    };

                    if let Some((name, created)) = STANDARD
                        .decode(&app_password.id)
                        .ok()
                        .and_then(|id| String::from_utf8(id).ok())
                        .and_then(|id| {
                            id.rsplit_once('$').map(|(name, created)| {
                                (
                                    name.to_string(),
                                    DateTime::parse_from_rfc3339(created)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&Utc)),
                                )
                            })
                        })
                    {
                        app_password.name = name;
                        app_password.created = created;
                    } else {
                        app_password.name.clone_from(&app_password.id);
                    }

                    if filter
                        .as_ref()
                        .is_none_or(|filter| app_password.name.contains(filter))
                    {
                        if offset == 0 {
                            items.push(app_password);
                            if items.len() as u32 >= PAGE_SIZE {
                                break;
                            }
                        } else {
                            offset -= 1;
                        }
                    }
                }

                Ok(Arc::new(List { items, total }))
            }
        },
    );

    let total_results = create_rw_signal(None::<u32>);
    let delete_action = create_action(move |items: &Arc<ItemSelection>| {
        let items = items.clone();
        let auth = auth.get();

        async move {
            if let Err(err) = HttpRequest::post("/api/account/auth")
                .with_authorization(&auth)
                .with_body(match items.as_ref() {
                    ItemSelection::All => {
                        vec![AccountAuthRequest::RemoveAppPassword { name: None }]
                    }
                    ItemSelection::Some(items) => items
                        .iter()
                        .map(|id| AccountAuthRequest::RemoveAppPassword {
                            name: id.to_string().into(),
                        })
                        .collect::<Vec<_>>(),
                    ItemSelection::None => unreachable!(),
                })
                .unwrap()
                .send::<serde_json::Value>()
                .await
            {
                alert.set(Alert::from(err));
            } else {
                alert.set(Alert::success(format!(
                    "Deleted {}.",
                    maybe_plural(
                        items.total_selected(total_results.get()),
                        "password",
                        "passwords"
                    )
                )));
                passwords.refetch();
            }
        }
    });

    view! {
        <ListSection>
            <ListTable title="App Passwords" subtitle="Manage your application passwords">
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new("/account/app-passwords")
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                    <ToolbarButton
                        text=Signal::derive(move || {
                            let ns = selected.get().total_selected(total_results.get());
                            if ns > 0 { format!("Delete ({ns})") } else { "Delete".to_string() }
                        })

                        color=Color::Red
                        on_click=Callback::new(move |_| {
                            let to_delete = selected.get().total_selected(total_results.get());
                            if to_delete > 0 {
                                let text = maybe_plural(to_delete, "password", "passwords");
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
                        text=format!("Add {}", "password")
                        color=Color::Blue
                        on_click=move |_| {
                            use_navigate()("/account/app-passwords/edit", Default::default());
                        }
                    >

                        <IconAdd size=16 attr:class="flex-shrink-0 size-3"/>
                    </ToolbarButton>

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match passwords.get() {
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
                        Some(Ok(passwords)) if !passwords.items.is_empty() => {
                            total_results.set(Some(passwords.total as u32));
                            Some(
                                view! {
                                    <ColumnList
                                        headers=vec!["Name".to_string(), "Created".to_string()]
                                        has_select_all=true
                                    >

                                        <For
                                            each=move || passwords.items.clone()
                                            key=|password| password.name.clone()
                                            let:password
                                        >
                                            <PasswordItem password/>
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
                                        button_text=format!("Create a new {}", "password")

                                        button_action=Callback::new(move |_| {
                                            use_navigate()(
                                                "/account/app-passwords/edit",
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
                                &UrlBuilder::new("/account/app-passwords")
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
fn PasswordItem(password: AppPassword) -> impl IntoView {
    let password_id = password.id.clone();

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=password_id/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>

            <ListItem subclass="ps-6 lg:ps-3 xl:ps-0 pe-6 py-3">
                <div class="flex items-center gap-x-3">
                    <span class="block text-sm font-semibold text-gray-800 dark:text-gray-200">
                        {password.name}
                    </span>
                </div>
            </ListItem>

            <ListItem subclass="px-6 py-1.5">
                {password
                    .created
                    .map(|created| HumanTime::from(created).to_string())
                    .unwrap_or_default()}
            </ListItem>

        </tr>
    }
}

pub fn generate_app_password() -> String {
    let mut app_password = String::with_capacity(19);
    for _ in 0..20 {
        app_password.push(rand::thread_rng().gen_range(b'a'..=b'z') as char);
        // Add a space every 4 characters
        if app_password.len().is_multiple_of(5) {
            app_password.push(' ');
        }
    }
    app_password
}

#[component]
pub fn AppPasswordCreate() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let mut data = expect_context::<Arc<Schemas>>().build_form("app-password");

    // Generate a random Application Password
    data.set("password", generate_app_password());

    let data = data.into_signal();

    let save_changes = create_action(move |(name, password): &(String, String)| {
        let auth = auth.get();
        let name = name.clone();
        let password = password.clone();

        async move {
            set_pending.set(true);

            let result = HttpRequest::post("/api/account/auth")
                .with_authorization(&auth)
                .with_body(vec![AccountAuthRequest::AddAppPassword {
                    name: STANDARD.encode(format!("{}${}", name, Utc::now().to_rfc3339())),
                    password: sha512_crypt::hash(password).unwrap(),
                }])
                .unwrap()
                .send::<serde_json::Value>()
                .await;

            set_pending.set(false);

            match result {
                Ok(_) => {
                    use_navigate()("/account/app-passwords", Default::default());
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="Create App Password" subtitle="Create a new application password">

            <FormSection>
                <FormItem label="Application Name">
                    <InputText element=FormElement::new("name", data)/>
                </FormItem>
                <FormItem label="Password">
                    <InputText element=FormElement::new("password", data)/>
                </FormItem>

            </FormSection>

            <FormButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()("/account/app-passwords", Default::default());
                    }
                />

                <Button
                    text="Create"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                save_changes
                                    .dispatch((
                                        data.value("name").unwrap(),
                                        data.value("password").unwrap(),
                                    ));
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}

impl Builder<Schemas, ()> {
    pub fn build_app_passwords(self) -> Self {
        self.new_schema("app-password")
            .new_field("name")
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("password")
            .typ(Type::Secret)
            .input_check([], [Validator::Required])
            .build()
            .build()
    }
}
