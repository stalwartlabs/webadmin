
use std::{sync::Arc, vec};

use humansize::{format_size, DECIMAL};
use leptos::{html::Form, *};
use leptos_router::{use_navigate, use_params_map};
use pwhash::sha512_crypt;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button, input::{InputPassword, InputSize, InputText}, select::Select, stacked_badge::StackedBadge, stacked_input::StackedInput, value_is_email, value_is_not_empty, value_lowercase, value_remove_spaces, value_trim, ButtonBar, Form, FormItem, FormListValidator, FormSection, FormValidator, StringValidateFn, ValidateCb
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::config::{schema::Schemas, Settings, Type},
};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct FetchSettings {
    pub items: Settings,
    pub total: u64,
}


#[component]
pub fn SettingsEdit(id: &'static str) -> impl IntoView {
    let schema = expect_context::<Arc<Schemas>>().get(id);
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let schema_ = schema.clone();
    let current_schema = create_memo(move |_| schema_.clone());

    let fetch_settings = create_resource(
        move || params().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get();
            let schema = current_schema.get();

            async move {
                if !name.is_empty() {
                    HttpRequest::get("/api/settings")
                    .with_authorization(&auth)
                    .with_parameter("prefix", format!("{}.{}", schema.prefix.unwrap(), name))
                    .send::<FetchSettings>().await
                    .map(|mut list| {
                        if !list.items.is_empty() {
                            list.items.insert("_id".to_string(), name.to_string());
                        }
                        list.items

                    })
                    
                } else {
                    Ok(schema.create())
                }
            }
        },
    );
    let (pending, set_pending) = create_signal(false);

    let settings = create_rw_signal(Settings::default());
    let login = FormValidator::new(String::new());
    let name = FormValidator::new(String::new());
    let typ: RwSignal<Type> = create_rw_signal(selected_type);
    let password = FormValidator::new(String::new());
    let quota = create_rw_signal(0u64);
    let member_of = create_rw_signal(Vec::<String>::new());
    let members = create_rw_signal(Vec::<String>::new());
    let email = FormValidator::new(String::new());
    let aliases = FormListValidator::new(Vec::<String>::new());

    let validate = move || {
        let aliases = aliases.validate([value_trim, value_lowercase], [value_is_email])?;

        Some(Settings {
            typ: typ.get().into(),
            quota: quota.get().into(),
            name: login
                .validate([value_remove_spaces, value_lowercase], [value_is_not_empty])?
                .into(),
            secrets: {
                let password = password.signal().get().ok()?;
                if !password.is_empty() {
                    vec![sha512_crypt::hash(password).unwrap()]
                } else {
                    vec![]
                }
            },
            emails: [email.validate([value_trim, value_lowercase], [value_is_email])?]
                .into_iter()
                .chain(aliases)
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>(),
            member_of: member_of.get(),
            members: members.get(),
            description: name
                .validate::<_, [_; 0], _, StringValidateFn>([value_trim], [])?
                .into(),
            ..Default::default()
        })
    };

    let save_changes = create_action(move |changes: &Settings| {
        let current = current_settings.get();
        let changes = changes.clone();
        let auth = auth.get();

        async move {
            set_pending.set(true);
            let result = if !current.is_blank() {
                let name = current.name.clone().unwrap_or_default();
                let updates = current.into_updates(changes);

                if !updates.is_empty() {
                    HttpRequest::patch(format!("/api/settings/{name}"))
                        .with_authorization(&auth)
                        .with_body(updates)
                        .unwrap()
                        .send::<()>()
                        .await
                } else {
                    Ok(())
                }
            } else {
                HttpRequest::post("/api/settings")
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
                        &format!("/manage/directory/{}", selected_type.resource_name(true)),
                        Default::default(),
                    );
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    let form = schema.form.sections.iter().map(|section| {
        let do_hide = create_memo(move |_| {
            section.display(&settings.get())
        });
        let title = section.title.map(|s| s.to_string());

        let components = section.fields.iter().cloned().map(|field| {
            let component = match field.typ_ {
                Type::Input => view! { <br/> }.into_view(),
                Type::InputMulti => todo!(),
                Type::Secret => todo!(),
                Type::Text => todo!(),
                Type::Expression => todo!(),
                Type::Select(_) => todo!(),
                Type::Checkbox => todo!(),
                Type::Duration => todo!(),
            };
            let hide = create_memo(move |_| {
                field.display(&settings.get())
            });
            let placeholder = create_memo(move |_| {
                field.placeholder(&settings.get()).unwrap_or_default().to_string()
            });


            view! {
                <FormItem label=field.label_form>
                    <InputText
                        placeholder=placeholder

                        value=name
                    />
                </FormItem>
            }

        }).collect_view();

        view! {
            <FormSection title=title hide=do_hide>
                {components}
            </FormSection>
        }

    }).collect_view();


    view! {
        <Form title=title subtitle=subtitle>

            <Transition fallback=Skeleton set_pending>

                {move || match fetch_settings.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(http::Error::NotFound)) => {
                        let url = format!(
                            "/manage/directory/{}",
                            selected_type.resource_name(true),
                        );
                        use_navigate()(&url, Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Ok(settings)) => {
                        login.update(settings.name.clone().unwrap_or_default());
                        typ.set(settings.typ.unwrap_or(selected_type));
                        name.update(settings.description.clone().unwrap_or_default());
                        password.update(String::new());
                        quota.set(settings.quota.unwrap_or_default());
                        member_of.set(settings.member_of.clone());
                        members.set(settings.members.clone());
                        email.update(settings.emails.first().cloned().unwrap_or_default());
                        aliases.update(settings.emails.iter().skip(1).cloned().collect::<Vec<_>>());
                        let used_quota = settings.used_quota.unwrap_or_default();
                        let total_quota = settings.quota.unwrap_or_default();
                        current_settings.set(settings);
                        Some(
                            view! {
                                <FormItem label=match selected_type {
                                    Type::Individual => "Login name",
                                    _ => "Name",
                                }>
                                    <InputText
                                        placeholder=match selected_type {
                                            Type::Individual => "Login name",
                                            _ => "Short Name",
                                        }

                                        value=login
                                    />
                                </FormItem>

                                <FormItem label=match selected_type {
                                    Type::Individual => "Name",
                                    _ => "Description",
                                }>
                                    <InputText
                                        placeholder=match selected_type {
                                            Type::Individual => "Full Name",
                                            _ => "Description",
                                        }

                                        value=name
                                    />
                                </FormItem>

                                <Show when=move || matches!(selected_type, Type::Individual)>
                                    <FormItem label="Type">
                                        <Select
                                            value=typ
                                            options=vec![Type::Individual, Type::Superuser]
                                        />

                                    </FormItem>

                                    <FormItem label="Password">
                                        <InputPassword value=password/>
                                    </FormItem>
                                </Show>

                                <FormItem label="Email">
                                    <InputText placeholder="user@example.org" value=email/>
                                </FormItem>

                                <FormItem label="Aliases">
                                    <StackedInput
                                        values=aliases
                                        placeholder="Email"
                                        add_button_text="Add Email".to_string()
                                    />
                                </FormItem>

                                <Show when=move || matches!(selected_type, Type::Individual)>
                                    <FormItem label="Disk quota">
                                        <div class="relative">
                                            <InputSize value=quota/>
                                            <Show when=move || { total_quota > 0 }>
                                                <p class="mt-3">
                                                    <label class="inline-flex items-center gap-x-1 text-xs text-black-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600">

                                                        {format!(
                                                            "{} used ({:.1}%)",
                                                            format_size(used_quota, DECIMAL),
                                                            (used_quota as f64 / total_quota as f64) * 100.0,
                                                        )}

                                                    </label>
                                                </p>

                                            </Show>

                                        </div>
                                    </FormItem>
                                </Show>

                                <Show when=move || {
                                    matches!(selected_type, Type::Group | Type::List)
                                }>
                                    <FormItem label="Members">
                                        <StackedBadge
                                            color=Color::Green
                                            values=members

                                            add_button_text="Add member".to_string()
                                            validate_item=Callback::new(move |(value, cb)| {
                                                settings_is_valid
                                                    .dispatch((
                                                        value,
                                                        cb,
                                                        if selected_type == Type::Group {
                                                            vec![Type::Individual, Type::Group]
                                                        } else {
                                                            vec![Type::Individual]
                                                        },
                                                    ));
                                            })
                                        />

                                    </FormItem>
                                </Show>

                                <Show when=move || {
                                    matches!(selected_type, Type::Individual | Type::Group)
                                }>
                                    <FormItem label="Member of">
                                        <StackedBadge
                                            color=Color::Blue

                                            values=member_of

                                            add_button_text="Add to group".to_string()
                                            validate_item=Callback::new(move |(value, cb)| {
                                                settings_is_valid
                                                    .dispatch((
                                                        value,
                                                        cb,
                                                        if selected_type == Type::Group {
                                                            vec![Type::Group]
                                                        } else {
                                                            vec![Type::Group, Type::List]
                                                        },
                                                    ));
                                            })
                                        />

                                    </FormItem>
                                </Show>
                            }
                                .into_view(),
                        )
                    }
                }}

            </Transition>

            <ButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()(
                            &format!("/manage/directory/{}", selected_type.resource_name(true)),
                            Default::default(),
                        );
                    }
                />

                <Button
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        if let Some(changes) = validate() {
                            save_changes.dispatch(changes);
                        }
                    })

                    disabled=pending
                />
            </ButtonBar>

        </Form>
    }
}
