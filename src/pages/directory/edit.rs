/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, vec};

use ahash::{AHashMap, AHashSet};
use base64::{engine::general_purpose, Engine};
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use pwhash::sha512_crypt;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputSize, InputText},
            select::Select,
            stacked_badge::StackedBadge,
            stacked_input::StackedInput,
            tab::Tab,
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        form::FormData,
        http::{self, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, Transformer, Type, Validator},
        Permission,
    },
    pages::{
        directory::{Principal, PrincipalType, PrincipalValue, PERMISSIONS},
        List,
    },
};

use super::{build_app_password, parse_app_password, SpecialSecrets};

type PrincipalMap = AHashMap<PrincipalType, Vec<(String, String)>>;

#[derive(Debug, Serialize, Deserialize, Default)]
enum Algorithm {
    #[default]
    Rsa,
    Ed25519,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct DkimSignature {
    id: Option<String>,
    algorithm: Algorithm,
    domain: String,
    selector: Option<String>,
}

#[component]
pub fn PrincipalEdit() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
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
            "tenants" => PrincipalType::Tenant,
            "domains" => PrincipalType::Domain,
            "roles" => PrincipalType::Role,
            "api-keys" => PrincipalType::ApiKey,
            "oauth-clients" => PrincipalType::OauthClient,
            _ => PrincipalType::Individual,
        }
    });
    let is_enterprise = auth.get_untracked().is_enterprise();
    let is_tenant = !auth
        .get_untracked()
        .permissions()
        .has_access(Permission::TenantList);
    let principals: RwSignal<Arc<PrincipalMap>> = create_rw_signal(Arc::new(AHashMap::new()));

    let fetch_principal = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();
            let permissions = auth.permissions();
            let selected_type = selected_type.get();

            let needed_types = match selected_type {
                PrincipalType::Individual => &[
                    PrincipalType::Role,
                    PrincipalType::Group,
                    PrincipalType::Tenant,
                    PrincipalType::List,
                ][..],
                PrincipalType::Group => &[
                    PrincipalType::Individual,
                    PrincipalType::Group,
                    PrincipalType::Tenant,
                    PrincipalType::List,
                ][..],
                PrincipalType::Domain => &[PrincipalType::Tenant][..],
                PrincipalType::Tenant | PrincipalType::ApiKey => &[PrincipalType::Role][..],
                PrincipalType::Role => &[PrincipalType::Role, PrincipalType::Tenant][..],
                PrincipalType::List => &[
                    PrincipalType::Individual,
                    PrincipalType::Group,
                    PrincipalType::Tenant,
                ][..],
                PrincipalType::Resource
                | PrincipalType::Location
                | PrincipalType::Other
                | PrincipalType::OauthClient => &[][..],
            };
            let mut fetch_types = String::new();
            for typ in needed_types {
                let permission = match typ {
                    PrincipalType::Individual => Permission::IndividualList,
                    PrincipalType::Group => Permission::GroupList,
                    PrincipalType::List => Permission::MailingListList,
                    PrincipalType::Domain => Permission::DomainList,
                    PrincipalType::Tenant if is_enterprise => Permission::TenantList,
                    PrincipalType::Role => Permission::RoleList,
                    _ => continue,
                };
                if permissions.has_access(permission) {
                    if !fetch_types.is_empty() {
                        fetch_types.push(',');
                    }
                    fetch_types.push_str(typ.id());
                }
            }

            async move {
                // Fetch principal
                let principal = if !name.is_empty() {
                    HttpRequest::get(("/api/principal", &name))
                        .with_authorization(&auth)
                        .send::<Principal>()
                        .await?
                } else {
                    // Add default roles
                    let mut principal = Principal::default();
                    match selected_type {
                        PrincipalType::Individual => {
                            principal.roles = PrincipalValue::StringList(vec!["user".to_string()]);
                        }
                        PrincipalType::Tenant => {
                            principal.roles = PrincipalValue::StringList(vec![
                                "tenant-admin".to_string(),
                                "user".to_string(),
                            ]);
                        }
                        PrincipalType::Group => {
                            principal.enabled_permissions = PrincipalValue::StringList(vec![
                                "email-send".to_string(),
                                "email-receive".to_string(),
                            ]);
                        }
                        PrincipalType::ApiKey => {
                            principal.secrets = PrincipalValue::StringList(vec![thread_rng()
                                .sample_iter(Alphanumeric)
                                .take(30)
                                .map(char::from)
                                .collect::<String>()]);
                            principal.enabled_permissions =
                                PrincipalValue::StringList(vec!["authenticate".to_string()]);
                        }
                        _ => {}
                    }

                    principal
                };

                // Add default roles
                let mut principals_: PrincipalMap = AHashMap::new();
                principals_.insert(
                    PrincipalType::Role,
                    vec![
                        ("admin".to_string(), "Administrator".to_string()),
                        (
                            "tenant-admin".to_string(),
                            "Tenant Administrator".to_string(),
                        ),
                        ("user".to_string(), "User".to_string()),
                    ],
                );

                if !fetch_types.is_empty() {
                    for principal in HttpRequest::get("/api/principal")
                        .with_authorization(&auth)
                        .with_parameter("types", fetch_types)
                        .with_parameter("fields", "name,description")
                        .with_optional_parameter("tenant", principal.tenant.as_str())
                        .send::<List<Principal>>()
                        .await?
                        .items
                    {
                        let id = principal.name.unwrap_string();
                        if id != name {
                            let description = principal
                                .description
                                .try_unwrap_string()
                                .map(|d| format!("{d} ({id})"))
                                .unwrap_or_else(|| id.clone());
                            principals_
                                .entry(principal.typ.unwrap())
                                .or_default()
                                .push((id, description));
                        }
                    }
                }
                principals.set(Arc::new(principals_));

                Ok(principal)
            }
        },
    );
    let (pending, set_pending) = create_signal(false);
    let current_principal = create_rw_signal(Principal::default());
    let data = expect_context::<Arc<Schemas>>()
        .build_form("principals")
        .into_signal();

    let save_changes = create_action(move |changes: &Principal| {
        let current = current_principal.get();
        let changes = changes.clone();
        let auth = auth.get();
        let selected_type = selected_type.get();

        async move {
            set_pending.set(true);
            let result = if !current.is_blank() {
                let name = current.name().unwrap_or_default().to_string();
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
                let result = HttpRequest::post("/api/principal")
                    .with_authorization(&auth)
                    .with_body(&changes)
                    .unwrap()
                    .send::<u32>()
                    .await
                    .map(|_| ());

                // Create DKIM keys
                if matches!(changes.typ, Some(PrincipalType::Domain))
                    && result.is_ok()
                    && auth
                        .permissions()
                        .has_access(Permission::DkimSignatureCreate)
                {
                    for algo in [Algorithm::Ed25519, Algorithm::Rsa] {
                        let _ = HttpRequest::post("/api/dkim")
                            .with_authorization(&auth)
                            .with_body(DkimSignature {
                                algorithm: algo,
                                domain: changes.name().unwrap_or_default().to_string(),
                                ..Default::default()
                            })
                            .unwrap()
                            .send::<()>()
                            .await;
                    }
                }

                result
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
                PrincipalType::Tenant => {
                    format!("Update '{name}' Tenant")
                }
                PrincipalType::Domain => {
                    format!("Update '{name}' Domain")
                }
                PrincipalType::Role => {
                    format!("Update '{name}' Role")
                }
                PrincipalType::ApiKey => {
                    format!("Update '{name}' API Key")
                }
                PrincipalType::OauthClient => {
                    format!("Update '{name}' OAuth Client")
                }
                _ => unreachable!(),
            }
        } else {
            match selected_type.get() {
                PrincipalType::Individual => "Create Account",
                PrincipalType::Group => "Create Group",
                PrincipalType::List => "Create List",
                PrincipalType::Tenant => "Create Tenant",
                PrincipalType::Domain => "Create Domain",
                PrincipalType::Role => "Create Role",
                PrincipalType::ApiKey => "Create API Key",
                PrincipalType::OauthClient => "Create OAuth Client",
                _ => unreachable!(),
            }
            .to_string()
        }
    });

    view! {
        <Form title=title subtitle="".to_string()>

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
                        let used_quota = principal.used_quota.as_int().unwrap_or_default();
                        let total_quota = principal.quota.as_int().unwrap_or_default();
                        current_principal.set(principal);
                        let typ = selected_type.get();
                        Some(
                            view! {
                                <Tab tabs=Signal::derive(move || {
                                    vec![
                                        Some("Details".to_string()),
                                        matches!(
                                            typ,
                                            PrincipalType::Individual | PrincipalType::ApiKey
                                        )
                                            .then_some("Authentication".to_string()),
                                        matches!(
                                            typ,
                                            PrincipalType::Individual | PrincipalType::Tenant
                                        )
                                            .then_some("Limits".to_string()),
                                        (!matches!(
                                            typ,
                                            PrincipalType::Tenant
                                            | PrincipalType::Domain
                                            | PrincipalType::OauthClient
                                            | PrincipalType::ApiKey
                                        ))
                                            .then_some("Memberships".to_string()),
                                        matches!(
                                            typ,
                                            PrincipalType::Individual
                                            | PrincipalType::Group
                                            | PrincipalType::Role
                                            | PrincipalType::Tenant
                                            | PrincipalType::ApiKey
                                        )
                                            .then_some("Permissions".to_string()),
                                    ]
                                })>

                                    <FormSection stacked=true>
                                        <FormItem
                                            stacked=true
                                            label=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual => "Login name",
                                                    PrincipalType::Domain => "Domain name",
                                                    PrincipalType::ApiKey => "Key Id",
                                                    PrincipalType::OauthClient => "Client Id",
                                                    _ => "Name",
                                                }
                                                    .to_string()
                                            })
                                        >

                                            <InputText
                                                placeholder=Signal::derive(move || {
                                                    match selected_type.get() {
                                                        PrincipalType::Individual => "Login name",
                                                        PrincipalType::Domain => "example.org",
                                                        _ => "Short Name",
                                                    }
                                                        .to_string()
                                                })

                                                element=FormElement::new("name", data)
                                            />
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual | PrincipalType::OauthClient => {
                                                        "Name"
                                                    }
                                                    _ => "Description",
                                                }
                                                    .to_string()
                                            })
                                        >

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

                                        <FormItem
                                            stacked=true
                                            label="Tenant"

                                            hide=Signal::derive(move || {
                                                is_tenant
                                                    || matches!(
                                                        selected_type.get(),
                                                        PrincipalType::Tenant | PrincipalType::OauthClient
                                                    )
                                            })
                                        >

                                            <Select
                                                element=FormElement::new("tenant", data)
                                                add_none=true
                                                disabled=!is_enterprise
                                                options=create_memo(move |_| {
                                                    principals
                                                        .get()
                                                        .get(&PrincipalType::Tenant)
                                                        .cloned()
                                                        .unwrap_or_default()
                                                })
                                            />

                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="Email"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual
                                                    | PrincipalType::Group
                                                    | PrincipalType::List
                                                    | PrincipalType::OauthClient
                                                )
                                            })
                                        >

                                            <InputText
                                                placeholder="user@example.org"
                                                element=FormElement::new("email", data)
                                            />
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="Aliases"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual
                                                    | PrincipalType::Group
                                                    | PrincipalType::List
                                                )
                                            })
                                        >

                                            <StackedInput
                                                element=FormElement::new("aliases", data)
                                                placeholder="Email"
                                                add_button_text="Add Email".to_string()
                                            />
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="Logo URL"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Tenant
                                                    | PrincipalType::Domain
                                                    | PrincipalType::OauthClient
                                                )
                                            })
                                        >

                                            <InputText
                                                element=FormElement::new("picture", data)
                                                disabled=!is_enterprise
                                            />
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="Redirect URIs"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::OauthClient)
                                            })
                                        >

                                            <StackedInput
                                                element=FormElement::new("urls", data)
                                                placeholder="URI"
                                                add_button_text="Add URI".to_string()
                                            />
                                        </FormItem>

                                    </FormSection>

                                    <FormSection stacked=true>

                                        <FormItem
                                            stacked=true
                                            label="Password"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Individual)
                                            })
                                        >

                                            <InputPassword element=FormElement::new("password", data)/>
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="OTP Auth URL"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Individual)
                                            })
                                        >

                                            <InputPassword element=FormElement::new(
                                                "otpauth_url",
                                                data,
                                            )/>
                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="App Passwords"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Individual)
                                            })
                                        >

                                            <StackedBadge
                                                color=Color::Gray
                                                element=FormElement::new("app_passwords", data)
                                                add_button_text="Add password".to_string()
                                            />

                                        </FormItem>

                                        <FormItem
                                            stacked=true
                                            label="Key"

                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::ApiKey)
                                            })
                                        >

                                            <span class="block font-semibold font-mono text-gray-800 dark:text-gray-200">
                                                {move || {
                                                    let data = data.get();
                                                    let name = data.value::<String>("name").unwrap_or_default();
                                                    let secret = data
                                                        .value::<String>("api_secret")
                                                        .unwrap_or_default();
                                                    (!name.is_empty() && !secret.is_empty())
                                                        .then(|| {
                                                            format!(
                                                                "api_{}",
                                                                general_purpose::STANDARD
                                                                    .encode(format!("{}:{}", name, secret).as_bytes()),
                                                            )
                                                        })
                                                }}

                                            </span>

                                        </FormItem>

                                    </FormSection>

                                    <FormSection stacked=true>
                                        <FormItem
                                            stacked=true
                                            label="Disk quota"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual | PrincipalType::Tenant
                                                )
                                            })
                                        >

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

                                        <FormItem
                                            stacked=true
                                            label="Maximum number of Accounts"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_accounts", data)/>
                                        </FormItem>
                                        <FormItem
                                            stacked=true
                                            label="Maximum number of Domains"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_domains", data)/>
                                        </FormItem>
                                        <FormItem
                                            stacked=true
                                            label="Maximum number of Groups"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_groups", data)/>
                                        </FormItem>
                                        <FormItem
                                            stacked=true
                                            label="Maximum number of Lists"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_lists", data)/>
                                        </FormItem>
                                        <FormItem
                                            stacked=true
                                            label="Maximum number of Roles"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_roles", data)/>
                                        </FormItem>
                                        <FormItem
                                            stacked=true
                                            label="Maximum number of API Keys"
                                            hide=Signal::derive(move || {
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("max_api_keys", data)/>
                                        </FormItem>

                                    </FormSection>

                                    <FormSection>
                                        <FormItem
                                            label="Members"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Role
                                                    | PrincipalType::Group
                                                    | PrincipalType::List
                                                    | PrincipalType::Tenant
                                                )
                                            })
                                        >

                                            <StackedBadge
                                                color=Color::Green
                                                element=FormElement::new("members", data)
                                                add_button_text="Add member".to_string()
                                                options=create_memo(move |_| {
                                                    let principals = principals.get();
                                                    let mut results = Vec::new();
                                                    let types = match selected_type.get() {
                                                        PrincipalType::Group | PrincipalType::List => {
                                                            &[PrincipalType::Individual, PrincipalType::Group][..]
                                                        }
                                                        PrincipalType::Role => {
                                                            &[PrincipalType::Individual, PrincipalType::Role][..]
                                                        }
                                                        _ => &[][..],
                                                    };
                                                    for typ in types {
                                                        if let Some(principals) = principals.get(typ) {
                                                            for (id, name) in principals {
                                                                results
                                                                    .push((id.clone(), format!("{} - {name}", typ.name())));
                                                            }
                                                        }
                                                    }
                                                    results
                                                })
                                            />

                                        </FormItem>

                                        <FormItem
                                            label="Member of"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual | PrincipalType::Group
                                                )
                                            })
                                        >

                                            <StackedBadge
                                                color=Color::Blue
                                                element=FormElement::new("member-of", data)
                                                add_button_text="Add to group".to_string()
                                                options=create_memo(move |_| {
                                                    principals
                                                        .get()
                                                        .get(&PrincipalType::Group)
                                                        .cloned()
                                                        .unwrap_or_default()
                                                })
                                            />

                                        </FormItem>

                                        <FormItem
                                            label="Mailing lists"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual | PrincipalType::Group
                                                )
                                            })
                                        >

                                            <StackedBadge
                                                color=Color::Blue
                                                element=FormElement::new("lists", data)
                                                add_button_text="Add to list".to_string()
                                                options=create_memo(move |_| {
                                                    principals
                                                        .get()
                                                        .get(&PrincipalType::List)
                                                        .cloned()
                                                        .unwrap_or_default()
                                                })
                                            />

                                        </FormItem>
                                    </FormSection>

                                    <FormSection>
                                        <FormItem
                                            label="Roles"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual
                                                    | PrincipalType::Group
                                                    | PrincipalType::Tenant
                                                    | PrincipalType::Role
                                                    | PrincipalType::ApiKey
                                                )
                                            })
                                        >

                                            <StackedBadge
                                                color=Color::Blue
                                                element=FormElement::new("roles", data)
                                                add_button_text="Assign roles".to_string()
                                                options=create_memo(move |_| {
                                                    principals
                                                        .get()
                                                        .get(&PrincipalType::Role)
                                                        .cloned()
                                                        .unwrap_or_default()
                                                })
                                            />

                                        </FormItem>
                                        <FormItem
                                            label="Permissions"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual
                                                    | PrincipalType::Group
                                                    | PrincipalType::Role
                                                    | PrincipalType::Tenant
                                                    | PrincipalType::ApiKey
                                                )
                                            })
                                        >

                                            <div class="grid space-y-2">
                                                {move || {
                                                    let form_data = data.get();
                                                    let enabled_permissions = form_data
                                                        .array_value("enabled-permissions")
                                                        .collect::<AHashSet<_>>();
                                                    let disabled_permissions = form_data
                                                        .array_value("disabled-permissions")
                                                        .collect::<AHashSet<_>>();
                                                    PERMISSIONS
                                                        .iter()
                                                        .map(|(id, name)| {
                                                            view! {
                                                                <div class="flex flex-col space-y-3 p-3 w-full bg-white border border-gray-200 rounded-lg focus:border-blue-500 focus:ring-blue-500 dark:bg-neutral-900 dark:border-neutral-700">
                                                                    <div class="flex justify-between items-center">
                                                                        <span
                                                                            id="hs-radio-delete-description"
                                                                            class="block text-sm text-gray-600 dark:text-neutral-500"
                                                                        >
                                                                            {name.to_string()}
                                                                        </span>

                                                                        <div class="flex gap-x-6">
                                                                            <div class="flex">
                                                                                <input
                                                                                    type="radio"
                                                                                    name=id.to_string()
                                                                                    class="shrink-0 mt-0.5 border-gray-200 rounded-full text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-neutral-800 dark:border-neutral-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                                                                    id=format!("{id}-on")
                                                                                    prop:checked=enabled_permissions.contains(id)
                                                                                    on:input=move |_| {
                                                                                        data.update_untracked(|data| {
                                                                                            data.array_push(
                                                                                                "enabled-permissions",
                                                                                                id.to_string(),
                                                                                                true,
                                                                                            );
                                                                                            data.array_delete_item("disabled-permissions", id);
                                                                                        });
                                                                                    }
                                                                                />

                                                                                <label class="text-sm text-gray-500 ms-2 dark:text-neutral-400">
                                                                                    On
                                                                                </label>
                                                                            </div>

                                                                            <div class="flex">
                                                                                <input
                                                                                    type="radio"
                                                                                    name=id.to_string()
                                                                                    class="shrink-0 mt-0.5 border-gray-200 rounded-full text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-neutral-800 dark:border-neutral-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                                                                    id=format!("{id}-off")
                                                                                    prop:checked=disabled_permissions.contains(id)
                                                                                    on:input=move |_| {
                                                                                        data.update_untracked(|data| {
                                                                                            data.array_push(
                                                                                                "disabled-permissions",
                                                                                                id.to_string(),
                                                                                                true,
                                                                                            );
                                                                                            data.array_delete_item("enabled-permissions", id);
                                                                                        });
                                                                                    }
                                                                                />

                                                                                <label class="text-sm text-gray-500 ms-2 dark:text-neutral-400">
                                                                                    Off
                                                                                </label>
                                                                            </div>

                                                                            <div class="flex">
                                                                                <input
                                                                                    type="radio"
                                                                                    name=id.to_string()
                                                                                    class="shrink-0 mt-0.5 border-gray-200 rounded-full text-blue-600 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-neutral-800 dark:border-neutral-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                                                                    id=format!("{id}-inherit")
                                                                                    prop:checked=!enabled_permissions.contains(id)
                                                                                        && !disabled_permissions.contains(id)
                                                                                    on:input=move |_| {
                                                                                        data.update_untracked(|data| {
                                                                                            data.array_delete_item("disabled-permissions", id);
                                                                                            data.array_delete_item("enabled-permissions", id);
                                                                                        });
                                                                                    }
                                                                                />

                                                                                <label class="text-sm text-gray-500 ms-2 dark:text-neutral-400">
                                                                                    Default
                                                                                </label>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}

                                            </div>
                                        </FormItem>

                                    </FormSection>

                                </Tab>
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
        for (key, field) in [
            ("name", principal.name.as_str()),
            ("description", principal.description.as_str()),
            ("picture", principal.picture.as_str()),
            ("tenant", principal.tenant.as_str()),
            ("email", principal.emails.as_str()),
        ] {
            if let Some(value) = field {
                self.set(key, value.to_string());
            }
        }

        match &principal.quota {
            PrincipalValue::Integer(quota) => {
                if *quota > 0 {
                    self.set("quota", quota.to_string());
                }
            }
            PrincipalValue::IntegerList(quotas) => {
                for (pos, quota) in quotas.iter().enumerate() {
                    if *quota > 0 {
                        let field = match pos {
                            0 => "quota",
                            1 => "max_accounts",
                            2 => "max_groups",
                            3 => "max_resources",
                            4 => "max_locations",
                            6 => "max_lists",
                            7 => "max_other",
                            8 => "max_domains",
                            9 => "max_tenants",
                            10 => "max_roles",
                            11 => "max_api_keys",
                            _ => continue,
                        };

                        self.set(field, quota.to_string());
                    }
                }
            }
            _ => {}
        }
        self.set(
            "type",
            principal.typ.unwrap_or(default_type).id().to_string(),
        );
        self.array_set("aliases", principal.emails.as_string_list().iter().skip(1));

        for (key, list) in [
            ("member-of", principal.member_of.as_string_list()),
            ("members", principal.members.as_string_list()),
            ("roles", principal.roles.as_string_list()),
            ("lists", principal.lists.as_string_list()),
            (
                "enabled-permissions",
                principal.enabled_permissions.as_string_list(),
            ),
            (
                "disabled-permissions",
                principal.disabled_permissions.as_string_list(),
            ),
            ("urls", principal.urls.as_string_list()),
        ] {
            self.array_set(key, list.iter());
        }

        let mut app_passwords = vec![];
        for secret in principal.secrets.as_string_list() {
            if let Some((app, _)) = parse_app_password(secret) {
                app_passwords.push(app);
            } else if secret.is_otp_auth() {
                self.set("otpauth_url", secret);
            } else if default_type == PrincipalType::ApiKey {
                self.set("api_secret", secret);
            }
        }
        if !app_passwords.is_empty() {
            self.array_set("app_passwords", app_passwords);
        }
    }

    fn to_principal(&mut self) -> Option<Principal> {
        if self.validate_form() {
            let mut secrets = vec![];
            for app_name in self.array_value("app_passwords") {
                secrets.push(build_app_password(app_name, ""));
            }
            if let Some(password) = self.value::<String>("password") {
                secrets.push(sha512_crypt::hash(password).unwrap());
            } else if let Some(password) = self.value::<String>("api_secret") {
                secrets.push(password);
            }

            if let Some(otpauth_url) = self.value::<String>("otpauth_url") {
                secrets.push(otpauth_url);
            }

            let typ = self.value::<PrincipalType>("type").unwrap();
            let mut principal = Principal {
                typ: Some(typ),
                quota: self.quota(typ),
                name: PrincipalValue::String(self.value::<String>("name").unwrap_or_default()),
                tenant: PrincipalValue::String(self.value::<String>("tenant").unwrap_or_default()),
                secrets: PrincipalValue::StringList(secrets),
                emails: PrincipalValue::StringList(
                    [self.value::<String>("email").unwrap_or_default()]
                        .into_iter()
                        .chain(self.array_value("aliases").map(|m| m.to_string()))
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<_>>(),
                ),
                picture: PrincipalValue::String(self.value("picture").unwrap_or_default()),
                description: PrincipalValue::String(self.value("description").unwrap_or_default()),
                ..Default::default()
            };

            for (key, list) in [
                ("member-of", &mut principal.member_of),
                ("members", &mut principal.members),
                ("roles", &mut principal.roles),
                ("lists", &mut principal.lists),
                ("enabled-permissions", &mut principal.enabled_permissions),
                ("disabled-permissions", &mut principal.disabled_permissions),
                ("urls", &mut principal.urls),
            ] {
                *list = PrincipalValue::StringList(
                    self.array_value(key).map(|m| m.to_string()).collect(),
                );
            }

            Some(principal)
        } else {
            None
        }
    }

    pub fn quota(&mut self, typ: PrincipalType) -> PrincipalValue {
        if typ == PrincipalType::Tenant {
            PrincipalValue::IntegerList(
                [
                    "quota",
                    "max_accounts",
                    "max_groups",
                    "max_resources",
                    "max_locations",
                    "max_admins",
                    "max_lists",
                    "max_other",
                    "max_domains",
                    "max_tenants",
                    "max_roles",
                    "max_api_keys",
                ]
                .iter()
                .map(|f| self.value::<u64>(f).unwrap_or_default())
                .collect(),
            )
        } else {
            PrincipalValue::Integer(self.value("quota").unwrap_or_default())
        }
    }
}

impl Builder<Schemas, ()> {
    pub fn build_principals(self) -> Self {
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
            .new_field("urls")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::IsUrl])
            .build()
            .new_field("description")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("otpauth_url")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsUrl])
            .build()
            .build()
    }
}
