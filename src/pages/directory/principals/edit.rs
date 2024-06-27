/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, vec};

use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use pwhash::sha512_crypt;

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputSize, InputText},
            select::Select,
            stacked_badge::StackedBadge,
            stacked_input::StackedInput,
            Form, FormButtonBar, FormElement, FormItem, FormSection, ValidateCb,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        form::FormData,
        http::{self, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, Source, Transformer, Type, Validator},
    },
    pages::directory::{Principal, PrincipalType},
};

#[component]
pub fn PrincipalEdit() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let fetch_principal = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();

            async move {
                if !name.is_empty() {
                    HttpRequest::get(("/api/principal", &name))
                        .with_authorization(&auth)
                        .send::<Principal>()
                        .await
                } else {
                    Ok(Principal::default())
                }
            }
        },
    );
    let selected_type = create_memo(move |_| {
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
    let (pending, set_pending) = create_signal(false);

    let current_principal = create_rw_signal(Principal::default());
    let data = expect_context::<Arc<Schemas>>()
        .build_form("principals")
        .into_signal();

    let principal_is_valid = create_action(
        move |(name, cb, expected_types): &(String, ValidateCb, Vec<PrincipalType>)| {
            let name = name.clone();
            let login_name = data.get().value::<String>("name").unwrap_or_default();
            let auth = auth.get();
            let expected_types = expected_types.clone();
            let cb = *cb;

            async move {
                if name == login_name {
                    cb.call(Err(
                        "Principal name cannot be the same as the current principal".to_string(),
                    ));
                    return;
                }

                let result = match HttpRequest::get(("/api/principal", &name))
                    .with_authorization(&auth)
                    .send::<Principal>()
                    .await
                {
                    Ok(principal)
                        if principal
                            .typ
                            .as_ref()
                            .map_or(false, |t| expected_types.contains(t)) =>
                    {
                        Ok(name)
                    }
                    Ok(_) => Err(format!(
                        "Principal is not a {}",
                        expected_types.first().unwrap().item_name(false)
                    )),
                    Err(http::Error::NotFound) => Err("Principal does not exist".to_string()),
                    Err(http::Error::Unauthorized) => {
                        use_navigate()("/login", Default::default());
                        Err("Unauthorized".to_string())
                    }
                    Err(err) => Err(format!("Request failed: {err:?}")),
                };

                cb.call(result);
            }
        },
    );
    let save_changes = create_action(move |changes: &Principal| {
        let current = current_principal.get();
        let changes = changes.clone();
        let auth = auth.get();
        let selected_type = selected_type.get();

        async move {
            set_pending.set(true);
            let result = if !current.is_blank() {
                let name = current.name.clone().unwrap_or_default();
                let updates = current.into_updates(changes);

                if !updates.is_empty() {
                    HttpRequest::patch(("/api/principal", &name))
                        .with_authorization(&auth)
                        .with_body(updates)
                        .unwrap()
                        .send::<()>()
                        .await
                } else {
                    Ok(())
                }
            } else {
                HttpRequest::post("/api/principal")
                    .with_authorization(&auth)
                    .with_body(changes)
                    .unwrap()
                    .send::<u32>()
                    .await
                    .map(|_| ())
            };
            set_pending.set(false);

            match result {
                Ok(_) => {
                    use_navigate()(
                        &format!("/manage/directory/{}", selected_type.resource_name()),
                        Default::default(),
                    );
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    let subtitle = create_memo(move |_| {
        match selected_type.get() {
            PrincipalType::Individual => "Manage account details, password and email addresses.",
            PrincipalType::Group => "Manage group members and member groups.",
            PrincipalType::List => "Manage list details and members.",
            _ => unreachable!(),
        }
        .to_string()
    });
    let title = create_memo(move |_| {
        if let Some(name) = params.get().get("id") {
            match selected_type.get() {
                PrincipalType::Individual => {
                    format!("Update '{name}' Account")
                }
                PrincipalType::Group => {
                    format!("Update '{name}' Group")
                }
                PrincipalType::List => {
                    format!("Update '{name}' List")
                }
                _ => unreachable!(),
            }
        } else {
            match selected_type.get() {
                PrincipalType::Individual => "Create Account",
                PrincipalType::Group => "Create Group",
                PrincipalType::List => "Create List",
                _ => unreachable!(),
            }
            .to_string()
        }
    });

    view! {
        <Form title=title subtitle=subtitle>

            <Transition fallback=Skeleton set_pending>

                {move || match fetch_principal.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(http::Error::NotFound)) => {
                        let url = format!(
                            "/manage/directory/{}",
                            selected_type.get().resource_name(),
                        );
                        use_navigate()(&url, Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Ok(principal)) => {
                        data.update(|data| {
                            data.from_principal(&principal, selected_type.get());
                        });
                        let used_quota = principal.used_quota.unwrap_or_default();
                        let total_quota = principal.quota.unwrap_or_default();
                        current_principal.set(principal);
                        Some(
                            view! {
                                <FormSection>
                                    <FormItem label=Signal::derive(move || {
                                        match selected_type.get() {
                                            PrincipalType::Individual => "Login name",
                                            _ => "Name",
                                        }
                                            .to_string()
                                    })>

                                        <InputText
                                            placeholder=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual => "Login name",
                                                    _ => "Short Name",
                                                }
                                                    .to_string()
                                            })

                                            element=FormElement::new("name", data)
                                        />
                                    </FormItem>

                                    <FormItem label=Signal::derive(move || {
                                        match selected_type.get() {
                                            PrincipalType::Individual => "Name",
                                            _ => "Description",
                                        }
                                            .to_string()
                                    })>
                                        <InputText
                                            placeholder=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual => "Full Name",
                                                    _ => "Description",
                                                }
                                                    .to_string()
                                            })

                                            element=FormElement::new("description", data)
                                        />
                                    </FormItem>

                                    <Show when=move || {
                                        matches!(selected_type.get(), PrincipalType::Individual)
                                    }>
                                        <FormItem label="Type">
                                            <Select element=FormElement::new("type", data)/>

                                        </FormItem>

                                        <FormItem label="Password">
                                            <InputPassword element=FormElement::new("password", data)/>
                                        </FormItem>
                                    </Show>

                                    <FormItem label="Email">
                                        <InputText
                                            placeholder="user@example.org"
                                            element=FormElement::new("email", data)
                                        />
                                    </FormItem>

                                    <FormItem label="Aliases">
                                        <StackedInput
                                            element=FormElement::new("aliases", data)
                                            placeholder="Email"
                                            add_button_text="Add Email".to_string()
                                        />
                                    </FormItem>

                                    <Show when=move || {
                                        matches!(selected_type.get(), PrincipalType::Individual)
                                    }>
                                        <FormItem label="Disk quota">
                                            <div class="relative">
                                                <InputSize element=FormElement::new("quota", data)/>
                                                <Show when=move || { used_quota > 0 }>
                                                    <p class="mt-3">
                                                        <label class="inline-flex items-center gap-x-1 text-xs text-black-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600">

                                                            {if total_quota > 0 {
                                                                format!(
                                                                    "{} used ({:.1}%)",
                                                                    format_size(used_quota, DECIMAL),
                                                                    (used_quota as f64 / total_quota as f64) * 100.0,
                                                                )
                                                            } else {
                                                                format!("{} used", format_size(used_quota, DECIMAL))
                                                            }}

                                                        </label>
                                                    </p>

                                                </Show>

                                            </div>
                                        </FormItem>
                                    </Show>

                                    <Show when=move || {
                                        matches!(
                                            selected_type.get(),
                                            PrincipalType::Group | PrincipalType::List
                                        )
                                    }>
                                        <FormItem label="Members">
                                            <StackedBadge
                                                color=Color::Green
                                                element=FormElement::new("members", data)

                                                add_button_text="Add member".to_string()
                                                validate_item=Callback::new(move |(value, cb)| {
                                                    principal_is_valid
                                                        .dispatch((
                                                            value,
                                                            cb,
                                                            if selected_type.get() == PrincipalType::Group {
                                                                vec![PrincipalType::Individual, PrincipalType::Group]
                                                            } else {
                                                                vec![PrincipalType::Individual]
                                                            },
                                                        ));
                                                })
                                            />

                                        </FormItem>
                                    </Show>

                                    <Show when=move || {
                                        matches!(
                                            selected_type.get(),
                                            PrincipalType::Individual | PrincipalType::Group
                                        )
                                    }>
                                        <FormItem label="Member of">
                                            <StackedBadge
                                                color=Color::Blue

                                                element=FormElement::new("member-of", data)

                                                add_button_text="Add to group".to_string()
                                                validate_item=Callback::new(move |(value, cb)| {
                                                    principal_is_valid
                                                        .dispatch((
                                                            value,
                                                            cb,
                                                            if selected_type.get() == PrincipalType::Group {
                                                                vec![PrincipalType::Group]
                                                            } else {
                                                                vec![PrincipalType::Group, PrincipalType::List]
                                                            },
                                                        ));
                                                })
                                            />

                                        </FormItem>
                                    </Show>
                                </FormSection>
                            }
                                .into_view(),
                        )
                    }
                }}

            </Transition>

            <FormButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()(
                            &format!("/manage/directory/{}", selected_type.get().resource_name()),
                            Default::default(),
                        );
                    }
                />

                <Button
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if let Some(changes) = data.to_principal() {
                                save_changes.dispatch(changes);
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}

#[allow(clippy::wrong_self_convention)]
impl FormData {
    fn from_principal(&mut self, principal: &Principal, default_type: PrincipalType) {
        self.set("name", principal.name.clone().unwrap_or_default());
        self.set(
            "description",
            principal.description.clone().unwrap_or_default(),
        );
        match principal.quota {
            Some(quota) if quota > 0 => {
                self.set("quota", quota.to_string());
            }
            _ => {}
        }
        self.set(
            "type",
            principal.typ.unwrap_or(default_type).id().to_string(),
        );
        if let Some(email) = principal.emails.first() {
            self.set("email", email);
        }
        self.array_set("member-of", principal.member_of.iter());
        self.array_set("members", principal.members.iter());
        self.array_set("aliases", principal.emails.iter().skip(1));
    }

    fn to_principal(&mut self) -> Option<Principal> {
        if self.validate_form() {
            Some(Principal {
                typ: self.value::<PrincipalType>("type").unwrap().into(),
                quota: self.value("quota"),
                name: self.value::<String>("name").unwrap().into(),
                secrets: {
                    if let Some(password) = self.value::<String>("password") {
                        vec![sha512_crypt::hash(password).unwrap()]
                    } else {
                        vec![]
                    }
                },
                emails: [self.value::<String>("email").unwrap_or_default()]
                    .into_iter()
                    .chain(self.array_value("aliases").map(|m| m.to_string()))
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>(),
                member_of: self
                    .array_value("member-of")
                    .map(|m| m.to_string())
                    .collect(),
                members: self.array_value("members").map(|m| m.to_string()).collect(),
                description: self.value("description"),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl Builder<Schemas, ()> {
    pub fn build_principals(self) -> Self {
        const IDS: &[(&str, &str)] = &[
            (
                PrincipalType::Individual.id(),
                PrincipalType::Individual.name(),
            ),
            (
                PrincipalType::Superuser.id(),
                PrincipalType::Superuser.name(),
            ),
        ];

        self.new_schema("principals")
            .new_field("name")
            .typ(Type::Input)
            .input_check(
                [Transformer::RemoveSpaces, Transformer::Lowercase],
                [Validator::Required],
            )
            .build()
            .new_field("email")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim, Transformer::Lowercase],
                [Validator::IsEmail],
            )
            .build()
            .new_field("aliases")
            .typ(Type::Array)
            .input_check(
                [Transformer::Trim, Transformer::Lowercase],
                [Validator::IsEmail],
            )
            .build()
            .new_field("description")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("type")
            .typ(Type::Select {
                source: Source::Static(IDS),
                multi: false,
            })
            .default(PrincipalType::Individual.id())
            .build()
            .build()
    }
}
