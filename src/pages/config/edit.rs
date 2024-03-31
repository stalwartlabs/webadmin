use std::sync::Arc;

use ahash::AHashMap;
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            expression::InputExpression,
            input::{
                InputDuration, InputPassword, InputRate, InputSize, InputSwitch, InputText,
                TextArea,
            },
            select::{CheckboxGroup, Select, SelectCron},
            stacked_input::StackedInput,
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        icon::IconRefresh,
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        form::{ExternalSources, FormData},
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::{
        config::{ReloadSettings, Schema, SchemaType, Schemas, Settings, Type, UpdateSettings},
        List,
    },
};

#[derive(Clone, Serialize, Deserialize, Default)]
struct FetchSettings {
    pub items: Settings,
    pub total: u64,
}

#[derive(Clone, Serialize, Deserialize)]
enum FetchResult {
    Update {
        settings: Settings,
        external_sources: ExternalSources,
    },
    Create {
        external_sources: ExternalSources,
    },
    NotFound,
}

const DEFAULT_REDIRECT_URL: &str = "/settings/network/edit";

#[component]
pub fn SettingsEdit() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();

    let schemas = expect_context::<Arc<Schemas>>();
    let current_schema = create_memo(move |_| {
        if let Some(schema) = params()
            .get("object")
            .and_then(|id| schemas.schemas.get(id.as_str()))
        {
            schema.clone()
        } else {
            use_navigate()("/404", Default::default());
            Arc::new(Schema::default())
        }
    });

    let fetch_settings = create_resource(
        move || params().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();
            let schema = current_schema.get();
            let is_create = name.is_empty();

            async move {
                // Fetch external sources
                let mut external_sources = ExternalSources::new();
                for (schema, field) in schema.external_sources() {
                    let source_key = format!("{}_{}", schema.id, field.id);
                    if !external_sources.contains_key(&source_key) {
                        let items = HttpRequest::get("/api/settings/group")
                            .with_authorization(&auth)
                            .with_parameter("prefix", schema.unwrap_prefix())
                            .with_parameter(
                                "suffix",
                                schema.try_unwrap_suffix().unwrap_or_default(),
                            )
                            .with_parameter("field", field.id)
                            .send::<List<Settings>>()
                            .await?
                            .items;

                        external_sources.insert(
                            source_key,
                            items
                                .into_iter()
                                .filter_map(|mut item| {
                                    (
                                        item.remove("_id")?,
                                        item.remove(field.id).unwrap_or_default(),
                                    )
                                        .into()
                                })
                                .collect::<Vec<_>>(),
                        );
                    }
                }

                // Fetch settings
                match schema.typ {
                    SchemaType::Record { prefix, .. } => {
                        if !is_create {
                            HttpRequest::get("/api/settings/list")
                                .with_authorization(&auth)
                                .with_parameter("prefix", format!("{prefix}.{name}"))
                                .send::<FetchSettings>()
                                .await
                                .map(|mut list| {
                                    if !list.items.is_empty() {
                                        list.items.insert("_id".to_string(), name.to_string());
                                        FetchResult::Update {
                                            settings: list.items,
                                            external_sources,
                                        }
                                    } else {
                                        FetchResult::NotFound
                                    }
                                })
                        } else {
                            Ok(FetchResult::Create { external_sources })
                        }
                    }
                    SchemaType::Entry { prefix } => {
                        if !is_create {
                            HttpRequest::get("/api/settings/keys")
                                .with_authorization(&auth)
                                .with_parameter("keys", format!("{prefix}.{name}"))
                                .send::<AHashMap<String, Option<String>>>()
                                .await
                                .map(|list| {
                                    if let Some(value) = list.into_values().next().flatten() {
                                        let mut settings = Settings::new();
                                        settings.insert("_id".to_string(), name.to_string());
                                        settings.insert("_value".to_string(), value);
                                        FetchResult::Update {
                                            settings,
                                            external_sources,
                                        }
                                    } else {
                                        FetchResult::NotFound
                                    }
                                })
                        } else {
                            Ok(FetchResult::Create { external_sources })
                        }
                    }
                    SchemaType::List => {
                        let mut keys = Vec::new();
                        let mut prefixes = Vec::new();

                        for field in schema.fields.values() {
                            if field.is_multivalue() {
                                prefixes.push(field.id);
                                keys.push(field.id);
                            } else {
                                keys.push(field.id);
                            }
                        }

                        HttpRequest::get("/api/settings/keys")
                            .with_authorization(&auth)
                            .with_parameter("keys", keys.join(","))
                            .with_parameter("prefixes", prefixes.join(","))
                            .send::<Settings>()
                            .await
                            .map(|mut list| {
                                let mut settings = Settings::new();
                                for (name, value) in list.drain() {
                                    settings.insert(name, value);
                                }

                                if !settings.is_empty() {
                                    FetchResult::Update {
                                        settings,
                                        external_sources,
                                    }
                                } else {
                                    FetchResult::Create { external_sources }
                                }
                            })
                    }
                }
            }
        },
    );
    let (pending, set_pending) = create_signal(false);
    let data = FormData::default().into_signal();

    let save_changes = create_action(
        move |(changes, reload): &(Arc<Vec<UpdateSettings>>, bool)| {
            let changes = changes.clone();
            let reload = *reload;
            let auth = auth.get();
            let schema = current_schema.get();

            log::debug!("Saving changes: {:?}", changes);

            async move {
                set_pending.set(true);
                match HttpRequest::post("/api/settings")
                    .with_authorization(&auth)
                    .with_body(changes)
                    .unwrap()
                    .send::<Option<String>>()
                    .await
                    .map(|_| ())
                {
                    Ok(_) => {
                        if reload {
                            match HttpRequest::get(format!(
                                "/api/reload/{}",
                                schema.reload_prefix.unwrap_or_default()
                            ))
                            .with_authorization(&auth)
                            .send::<ReloadSettings>()
                            .await
                            {
                                Ok(result) => {
                                    set_pending.set(false);
                                    if result.errors.is_empty() {
                                        use_navigate()(&schema.list_path(), Default::default());
                                    } else {
                                        alert.set(Alert::from(result));
                                    }
                                }
                                Err(http::Error::Unauthorized) => {
                                    use_navigate()("/login", Default::default());
                                }
                                Err(err) => {
                                    set_pending.set(false);
                                    alert.set(Alert::from(err));
                                }
                            }
                        } else {
                            set_pending.set(false);
                            use_navigate()(&schema.list_path(), Default::default());
                        }
                    }
                    Err(err) => {
                        set_pending.set(false);
                        match err {
                            http::Error::Unauthorized => {
                                use_navigate()("/login", Default::default());
                            }
                            http::Error::Server { error, .. } if error == "assertFailed" => {
                                alert.set(Alert::error("Record already exists").with_details(
                                    "Another record with the same ID already exists",
                                ));
                            }
                            err => {
                                alert.set(Alert::from(err));
                            }
                        }
                    }
                }
            }
        },
    );

    view! {
        <Form
            title=Signal::derive(move || current_schema.get().form.title.to_string())
            subtitle=Signal::derive(move || current_schema.get().form.subtitle.to_string())
        >

            <Transition fallback=Skeleton set_pending>

                {move || match fetch_settings.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(http::Error::NotFound) | Ok(FetchResult::NotFound)) => {
                        let url = format!("/settings/{}", current_schema.get().id);
                        use_navigate()(&url, Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Ok(result)) => {
                        let (is_create, settings, external_sources) = match result {
                            FetchResult::Update { settings, external_sources } => {
                                (false, Some(settings), external_sources)
                            }
                            FetchResult::Create { external_sources } => {
                                (true, None, external_sources)
                            }
                            FetchResult::NotFound => unreachable!(),
                        };
                        let schema = current_schema.get();
                        let sections = schema.form.sections.iter().cloned();
                        data.set(
                            FormData::from_settings(schema.clone(), settings)
                                .with_external_sources(external_sources),
                        );
                        Some(
                            sections
                                .map(|section| {
                                    let title = section.title.map(|s| s.to_string());
                                    let section_ = section.clone();
                                    let hide_section = create_memo(move |_| {
                                        !section_.display(&data.get())
                                    });
                                    let components = section
                                        .fields
                                        .iter()
                                        .cloned()
                                        .map(|field| {
                                            let is_disabled = field.readonly && !is_create;
                                            let field_label = field.label_form;
                                            let help = field.help;
                                            let field_ = field.clone();
                                            let hide_label = create_memo(move |_| {
                                                !field_.display(&data.get())
                                            });
                                            let field_ = field.clone();
                                            let is_optional = create_memo(move |_| {
                                                !field_.is_required(&data.get())
                                            });
                                            let is_switch = matches!(field.typ_, Type::Boolean);
                                            let component = match field.typ_ {
                                                Type::Input => {
                                                    view! {
                                                        <InputText
                                                            element=FormElement::new(field.id, data)
                                                            placeholder=create_memo(move |_| {
                                                                field
                                                                    .placeholder(&data.get())
                                                                    .unwrap_or_default()
                                                                    .to_string()
                                                            })

                                                            disabled=is_disabled
                                                        />
                                                    }
                                                        .into_view()
                                                }
                                                Type::Array => {
                                                    view! {
                                                        <StackedInput
                                                            add_button_text="Add".to_string()
                                                            element=FormElement::new(field.id, data)
                                                            placeholder=create_memo(move |_| {
                                                                field
                                                                    .placeholder(&data.get())
                                                                    .unwrap_or_default()
                                                                    .to_string()
                                                            })
                                                        />
                                                    }
                                                        .into_view()
                                                }
                                                Type::Secret => {
                                                    view! {
                                                        <InputPassword element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Select { multi: false, .. } => {
                                                    view! {
                                                        <Select
                                                            element=FormElement::new(field.id, data)
                                                            disabled=is_disabled
                                                        />
                                                    }
                                                        .into_view()
                                                }
                                                Type::Select { multi: true, .. } => {
                                                    view! {
                                                        <CheckboxGroup
                                                            element=FormElement::new(field.id, data)
                                                            disabled=is_disabled
                                                        />
                                                    }
                                                        .into_view()
                                                }
                                                Type::Size => {
                                                    view! {
                                                        <InputSize element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Boolean => {
                                                    view! {
                                                        <InputSwitch
                                                            label=field_label
                                                            tooltip=help.unwrap_or_default()
                                                            element=FormElement::new(field.id, data)
                                                        />
                                                    }
                                                        .into_view()
                                                }
                                                Type::Duration => {
                                                    view! {
                                                        <InputDuration element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Rate => {
                                                    view! {
                                                        <InputRate element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Expression => {
                                                    view! {
                                                        <InputExpression element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Cron => {
                                                    view! {
                                                        <SelectCron element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Text => {
                                                    view! {
                                                        <TextArea element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                            };
                                            if !is_switch {
                                                view! {
                                                    <FormItem
                                                        label=field_label
                                                        hide=hide_label
                                                        is_optional=is_optional
                                                        tooltip=help.unwrap_or_default()
                                                    >
                                                        {component}
                                                    </FormItem>
                                                }
                                            } else {
                                                view! {
                                                    <FormItem label="" hide=hide_label is_optional=is_optional>
                                                        {component}
                                                    </FormItem>
                                                }
                                            }
                                        })
                                        .collect_view();
                                    view! {
                                        <FormSection
                                            title=title.unwrap_or_default()
                                            hide=hide_section
                                        >
                                            {components}
                                        </FormSection>
                                    }
                                        .into_view()
                                })
                                .collect_view(),
                        )
                    }
                }}

            </Transition>

            <FormButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()(&current_schema.get().list_path(), Default::default());
                    }
                />

                <Button
                    text="Save & Reload"
                    color=Color::Gray
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                save_changes.dispatch((Arc::new(data.build_update()), true));
                            }
                        });
                    })

                    disabled=pending
                >

                    <IconRefresh/>
                </Button>

                <Button
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                save_changes.dispatch((Arc::new(data.build_update()), false));
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}

impl Schema {
    fn list_path(&self) -> String {
        if !matches!(self.typ, SchemaType::List) {
            format!("/settings/{}", self.id)
        } else {
            DEFAULT_REDIRECT_URL.to_string()
        }
    }
}
