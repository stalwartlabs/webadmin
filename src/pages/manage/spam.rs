/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use leptos::*;
use leptos_router::use_navigate;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputText, TextArea},
            select::Select,
            stacked_input::StackedInput,
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        http::{Error, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, SelectType, Source, Transformer, Type, Validator},
        url::UrlBuilder,
    },
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpamClassifyRequest {
    pub message: String,

    // Session details
    pub remote_ip: IpAddr,
    #[serde(default)]
    pub ehlo_domain: String,
    #[serde(default)]
    pub authenticated_as: Option<String>,

    // TLS
    #[serde(default)]
    pub is_tls: bool,

    // Envelope
    pub env_from: String,
    pub env_from_flags: u64,
    pub env_rcpt_to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpamClassifyResponse {
    pub score: f64,
    pub tags: BTreeMap<String, SpamFilterDisposition<f64>>,
    pub disposition: SpamFilterDisposition<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum SpamFilterDisposition<T> {
    Allow { value: T },
    Discard,
    Reject,
}

#[component]
pub fn SpamTest() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let mut data = expect_context::<Arc<Schemas>>().build_form("spam-test");
    data.apply_defaults(false);
    let data = data.into_signal();

    let start_classify = create_action(move |req: &Arc<SpamClassifyRequest>| {
        let auth = auth.get();
        let req = req.clone();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post("/api/spam-filter/classify")
                .with_authorization(&auth)
                .with_body(req.as_ref())
                .unwrap()
                .send::<SpamClassifyResponse>()
                .await;

            set_pending.set(false);

            match result {
                Ok(response) => {
                    let title = match response.disposition {
                        SpamFilterDisposition::Allow { value } => {
                            format!(
                                "Message was classified as {} with a score of {:.2}",
                                if value.contains(": Yes") {
                                    "SPAM"
                                } else {
                                    "HAM"
                                },
                                response.score
                            )
                        }
                        SpamFilterDisposition::Discard => format!(
                            "Message was discarded with a score of {:.2}",
                            response.score
                        ),
                        SpamFilterDisposition::Reject => {
                            format!("Message was rejected with a score of {:.2}", response.score)
                        }
                    };

                    alert.set(
                        Alert::success(title)
                            .with_details_list(response.tags.into_iter().map(|(name, score)| {
                                match score {
                                    SpamFilterDisposition::Allow { value } => {
                                        format!("{name}: {value:.2}")
                                    }
                                    SpamFilterDisposition::Discard => format!("{name}: DISCARD"),
                                    SpamFilterDisposition::Reject => format!("{name}: REJECT"),
                                }
                            }))
                            .without_timeout(),
                    );
                }
                Err(Error::Unauthorized) => {
                    use_navigate()("/login", Default::default());
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="">

            <FormSection title="Test SPAM Filter".to_string()>
                <FormItem label="IP Address" tooltip="IP address of the remote SMTP server">
                    <InputText element=FormElement::new("remote_ip", data)/>
                </FormItem>
                <FormItem label="EHLO Domain" tooltip="Hostname specified at the EHLO stage">
                    <InputText element=FormElement::new("helo_domain", data)/>
                </FormItem>
            </FormSection>

            <FormSection title="Envelope".to_string()>
                <FormItem label="Sender" tooltip="Envelope return path address">
                    <InputText element=FormElement::new("env_from", data)/>
                </FormItem>
                <FormItem label="To" tooltip="Message recipients">
                    <StackedInput
                        element=FormElement::new("env_to", data)
                        add_button_text="Add".to_string()
                    />
                </FormItem>
            </FormSection>

            <FormSection title="Message".to_string()>
                <FormItem label="Contents" tooltip="Message body">
                    <TextArea element=FormElement::new("message", data)/>
                </FormItem>
                <FormItem label="Parameters" tooltip="Body parameters">
                    <Select element=FormElement::new("env_from_flags", data)/>
                </FormItem>
            </FormSection>

            <FormButtonBar>

                <Button
                    text="Test"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                let req = SpamClassifyRequest {
                                    message: data.value("message").unwrap(),
                                    remote_ip: data
                                        .value("remote_ip")
                                        .unwrap_or(Ipv4Addr::UNSPECIFIED.into()),
                                    ehlo_domain: data.value("helo_domain").unwrap_or_default(),
                                    authenticated_as: None,
                                    is_tls: true,
                                    env_from: data.value("env_from").unwrap_or_default(),
                                    env_from_flags: 0,
                                    env_rcpt_to: data
                                        .array_value("env_to")
                                        .map(|v| v.to_string())
                                        .collect(),
                                };
                                start_classify.dispatch(Arc::new(req));
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}

#[component]
pub fn SpamTrain() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let mut data = expect_context::<Arc<Schemas>>().build_form("spam-train");
    data.apply_defaults(false);
    let data = data.into_signal();

    let start_train = create_action(move |req: &Arc<SpamFilterTrain>| {
        let auth = auth.get();
        let req = req.clone();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post(
                UrlBuilder::new("/api/spam-filter/train")
                    .with_subpath(req.train.as_str())
                    .with_optional_subpath(req.account.as_deref())
                    .finish(),
            )
            .with_authorization(&auth)
            .with_raw_body(req.message.clone())
            .send::<serde_json::Value>()
            .await;

            set_pending.set(false);

            match result {
                Ok(_) => {
                    data.update(|data| {
                        data.reset();
                    });
                    alert.set(Alert::success("Training successful"));
                }
                Err(Error::Unauthorized) => {
                    use_navigate()("/login", Default::default());
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="Train Spam filter" subtitle="Train the Bayes classifier">

            <FormSection>
                <FormItem label="Train">
                    <Select element=FormElement::new("train", data)/>
                </FormItem>
                <FormItem label="Account Name" is_optional=true>
                    <InputText element=FormElement::new("account", data)/>
                </FormItem>
                <FormItem label="Message">
                    <TextArea element=FormElement::new("message", data)/>
                </FormItem>

            </FormSection>

            <FormButtonBar>

                <Button
                    text="Train"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                start_train
                                    .dispatch(
                                        Arc::new(SpamFilterTrain {
                                            train: data.value("train").unwrap(),
                                            account: data.value("account"),
                                            message: data.value("message").unwrap(),
                                        }),
                                    );
                            }
                        });
                    })

                    disabled=pending
                />
            </FormButtonBar>

        </Form>
    }
}

struct SpamFilterTrain {
    train: String,
    account: Option<String>,
    message: String,
}

impl Builder<Schemas, ()> {
    pub fn build_spam_manage(self) -> Self {
        self.new_schema("spam-test")
            .new_field("message")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_field("remote_ip")
            .typ(Type::Text)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsIpOrMask],
            )
            .build()
            .new_field("helo_domain")
            .typ(Type::Text)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("env_from")
            .typ(Type::Text)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("env_to")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("env_from_flags")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(MAIL_BODY),
            })
            .default("")
            .build()
            .build()
            // SPAM train
            .new_schema("spam-train")
            .new_field("message")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_field("account")
            .typ(Type::Text)
            .build()
            .new_field("train")
            .default("spam")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(&[("spam", "SPAM"), ("ham", "HAM")]),
            })
            .build()
            .build()
    }
}

pub static MAIL_BODY: &[(&str, &str)] = &[
    ("", "Not specified"),
    ("7bit", "7bit"),
    ("8bitmime", "8bit MIME"),
    ("binarymime", "Binary MIME"),
    ("smtputf8", "SMTPUTF8"),
];
