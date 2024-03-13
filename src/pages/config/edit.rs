use std::sync::Arc;

use ahash::AHashMap;
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputSize, InputSwitch, InputText},
            select::Select,
            stacked_input::StackedInput,
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
    },
    pages::config::{Schema, SchemaType, Schemas, Settings, Type, UpdateSettings},
};

#[derive(Clone, Serialize, Deserialize, Default)]
struct FetchSettings {
    pub items: Settings,
    pub total: u64,
}

#[derive(Clone, Serialize, Deserialize)]
enum FetchResult {
    Update(Settings),
    Create,
    NotFound,
}

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
            let auth = auth.get();
            let schema = current_schema.get();
            let is_create = name.is_empty();

            async move {
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
                                        FetchResult::Update(list.items)
                                    } else {
                                        FetchResult::NotFound
                                    }
                                })
                        } else {
                            Ok(FetchResult::Create)
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
                                        FetchResult::Update(settings)
                                    } else {
                                        FetchResult::NotFound
                                    }
                                })
                        } else {
                            Ok(FetchResult::Create)
                        }
                    }
                    SchemaType::List => HttpRequest::get("/api/settings/keys")
                        .with_authorization(&auth)
                        .with_parameter(
                            "keys",
                            schema
                                .fields
                                .values()
                                .map(|field| field.id)
                                .collect::<Vec<_>>()
                                .join(","),
                        )
                        .send::<AHashMap<String, Option<String>>>()
                        .await
                        .map(|mut list| {
                            let mut settings = Settings::new();
                            for (name, value) in list.drain() {
                                if let Some(value) = value {
                                    settings.insert(name, value);
                                }
                            }

                            if !list.is_empty() {
                                FetchResult::Update(settings)
                            } else {
                                FetchResult::Create
                            }
                        }),
                }
            }
        },
    );
    let (pending, set_pending) = create_signal(false);
    let data = FormData::default().into_signal();

    let save_changes = create_action(move |changes: &Arc<Vec<UpdateSettings>>| {
        let changes = changes.clone();
        let auth = auth.get();
        let schema = current_schema.get();

        log::debug!("Saving changes: {:#?}", changes);

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
                    set_pending.set(false);
                    use_navigate()(&format!("/settings/{}", schema.id), Default::default());
                }
                Err(err) => {
                    set_pending.set(false);
                    match err {
                        http::Error::Unauthorized => {
                            use_navigate()("/login", Default::default());
                        }
                        http::Error::Server { error, .. } if error == "assertFailed" => {
                            alert
                                .set(Alert::error("Record already exists").with_details(
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
    });

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
                        let (is_create, settings) = match result {
                            FetchResult::Update(settings) => (false, Some(settings)),
                            FetchResult::Create => (true, None),
                            FetchResult::NotFound => unreachable!(),
                        };
                        let schema = current_schema.get();
                        let sections = schema.form.sections.iter().cloned();
                        data.set(FormData::from_settings(schema.clone(), settings));
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
                                            let component = match field.typ_ {
                                                Type::Input | Type::Duration => {
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
                                                Type::Text => todo!(),
                                                Type::Expression => todo!(),
                                                Type::Select(_) => {
                                                    view! {
                                                        <Select
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
                                                Type::Checkbox => {
                                                    view! {
                                                        <InputSwitch element=FormElement::new(field.id, data)/>
                                                    }
                                                        .into_view()
                                                }
                                                Type::Duration => view! { <p>checkbox</p> }.into_view(),
                                            };
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
                        use_navigate()(
                            &format!("/settings/{}", current_schema.get().id),
                            Default::default(),
                        );
                    }
                />

                <Button
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                save_changes.dispatch(Arc::new(data.build_update()));
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}
