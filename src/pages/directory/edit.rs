/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, vec};

use ahash::AHashMap;
use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use pwhash::sha512_crypt;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputSize, InputText},
            select::{CheckboxGroup, Select},
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
        directory::{Principal, PrincipalType, PERMISSIONS},
        List,
    },
};

use super::{build_app_password, parse_app_password, IntOrMany, SpecialSecrets};

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
            _ => PrincipalType::Individual,
        }
    });
    let is_enterprise = auth.get_untracked().is_enterprise();
    let principals: RwSignal<Arc<PrincipalMap>> = create_rw_signal(Arc::new(AHashMap::new()));
    let permissions = create_memo(move |_| {
        PERMISSIONS
            .iter()
            .map(|(id, name)| (id.to_string(), name.to_string()))
            .collect::<Vec<_>>()
    });

    let fetch_principal = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();
            let fetch_types = match (selected_type.get(), is_enterprise) {
                (PrincipalType::Individual, true) => Some("role,group,tenant,list"),
                (PrincipalType::Individual, false) => Some("role,group,list"),
                (PrincipalType::Group, true) => Some("individual,group,tenant,list"),
                (PrincipalType::Group, false) => Some("individual,group,list"),
                (PrincipalType::Domain, true) => Some("tenant"),
                (PrincipalType::Domain, false) => None,
                (PrincipalType::Tenant, _) => Some("role"),
                (PrincipalType::Role, true) => Some("role,tenant"),
                (PrincipalType::Role, false) => Some("role"),
                (PrincipalType::List, true) => Some("individual,group,tenant"),
                (PrincipalType::List, false) => Some("individual,group"),
                (PrincipalType::Resource | PrincipalType::Location | PrincipalType::Other, _) => {
                    None
                }
            };

            async move {
                let mut principals_: PrincipalMap = AHashMap::new();

                // Add default roles
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

                if let Some(fetch_types) = fetch_types {
                    for principal in HttpRequest::get("/api/principal")
                        .with_authorization(&auth)
                        .with_parameter("types", fetch_types)
                        .with_parameter("fields", "name,description")
                        .send::<List<Principal>>()
                        .await?
                        .items
                    {
                        let id = principal.name.unwrap_or_default();
                        if id != name {
                            let description = principal
                                .description
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
                                domain: changes.name.clone().unwrap(),
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
                        let used_quota = principal.used_quota.unwrap_or_default();
                        let total_quota = principal.quota.as_ref().map_or(0, |q| q.first());
                        current_principal.set(principal);
                        let typ = selected_type.get();
                        Some(
                            view! {
                                <Tab tabs=Signal::derive(move || {
                                    vec![
                                        Some("Details".to_string()),
                                        matches!(typ, PrincipalType::Individual)
                                            .then_some("Authentication".to_string()),
                                        matches!(
                                            typ,
                                            PrincipalType::Individual | PrincipalType::Tenant
                                        )
                                            .then_some("Limits".to_string()),
                                        (!matches!(
                                            typ,
                                            PrincipalType::Tenant | PrincipalType::Domain
                                        ))
                                            .then_some("Memberships".to_string()),
                                        matches!(
                                            typ,
                                            PrincipalType::Individual
                                            | PrincipalType::Role
                                            | PrincipalType::Tenant
                                        )
                                            .then_some("Security".to_string()),
                                    ]
                                })>

                                    <FormSection stacked=true>
                                        <FormItem
                                            stacked=true
                                            label=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual => "Login name",
                                                    _ => "Name",
                                                }
                                                    .to_string()
                                            })
                                        >

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

                                        <FormItem
                                            stacked=true
                                            label=Signal::derive(move || {
                                                match selected_type.get() {
                                                    PrincipalType::Individual => "Name",
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
                                                matches!(selected_type.get(), PrincipalType::Tenant)
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
                                                !matches!(selected_type.get(), PrincipalType::Tenant)
                                            })
                                        >

                                            <InputText element=FormElement::new("picture", data)/>
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
                                                    | PrincipalType::Tenant
                                                    | PrincipalType::Role
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
                                                    | PrincipalType::Role
                                                    | PrincipalType::Tenant
                                                )
                                            })
                                        >

                                            <CheckboxGroup
                                                element=FormElement::new("enabled-permissions", data)
                                                options=permissions
                                            />

                                        </FormItem>
                                        <FormItem
                                            label="Disabled"
                                            hide=Signal::derive(move || {
                                                !matches!(
                                                    selected_type.get(),
                                                    PrincipalType::Individual
                                                    | PrincipalType::Role
                                                    | PrincipalType::Tenant
                                                )
                                            })
                                        >

                                            <CheckboxGroup
                                                element=FormElement::new("disabled-permissions", data)
                                                options=permissions
                                            />

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
            ("name", principal.name.as_deref()),
            ("description", principal.description.as_deref()),
            ("picture", principal.picture.as_deref()),
            ("tenant", principal.tenant.as_deref()),
            ("email", principal.emails.first().map(|s| s.as_str())),
        ] {
            if let Some(value) = field {
                self.set(key, value.to_string());
            }
        }

        match &principal.quota {
            Some(IntOrMany::Int(quota)) => {
                if *quota > 0 {
                    self.set("quota", quota.to_string());
                }
            }
            Some(IntOrMany::Many(quotas)) => {
                for (pos, quota) in quotas.iter().enumerate() {
                    if *quota > 0 {
                        let field = match pos {
                            0 => "quota",
                            1 => "max_accounts",
                            2 => "max_groups",
                            3 => "max_resources",
                            4 => "max_locations",
                            5 => "max_lists",
                            6 => "max_other",
                            7 => "max_domains",
                            8 => "max_tenants",
                            9 => "max_roles",
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
        self.array_set("aliases", principal.emails.iter().skip(1));

        for (key, list) in [
            ("member-of", &principal.member_of),
            ("members", &principal.members),
            ("roles", &principal.roles),
            ("lists", &principal.lists),
            ("enabled-permissions", &principal.enabled_permissions),
            ("disabled-permissions", &principal.disabled_permissions),
        ] {
            self.array_set(key, list.iter());
        }

        let mut app_passwords = vec![];
        for secret in &principal.secrets {
            if let Some((app, _)) = parse_app_password(secret) {
                app_passwords.push(app);
            } else if secret.is_otp_auth() {
                self.set("otpauth_url", secret);
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
            }
            if let Some(otpauth_url) = self.value::<String>("otpauth_url") {
                secrets.push(otpauth_url);
            }

            let typ = self.value::<PrincipalType>("type").unwrap();
            let mut principal = Principal {
                typ: Some(typ),
                quota: self.quota(typ).into(),
                name: self.value::<String>("name").unwrap().into(),
                secrets,
                emails: [self.value::<String>("email").unwrap_or_default()]
                    .into_iter()
                    .chain(self.array_value("aliases").map(|m| m.to_string()))
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>(),
                picture: self.value("picture"),
                description: self.value("description"),
                ..Default::default()
            };

            for (key, list) in [
                ("member-of", &mut principal.member_of),
                ("members", &mut principal.members),
                ("roles", &mut principal.roles),
                ("lists", &mut principal.lists),
                ("enabled-permissions", &mut principal.enabled_permissions),
                ("disabled-permissions", &mut principal.disabled_permissions),
            ] {
                *list = self.array_value(key).map(|m| m.to_string()).collect();
            }

            Some(principal)
        } else {
            None
        }
    }

    pub fn quota(&mut self, typ: PrincipalType) -> IntOrMany {
        if typ == PrincipalType::Tenant {
            IntOrMany::Many(
                [
                    "quota",
                    "max_accounts",
                    "max_groups",
                    "max_resources",
                    "max_locations",
                    "max_lists",
                    "max_other",
                    "max_domains",
                    "max_tenants",
                    "max_roles",
                ]
                .iter()
                .map(|f| self.value::<u64>(f).unwrap_or_default())
                .collect(),
            )
        } else {
            IntOrMany::Int(self.value("quota").unwrap_or_default())
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
