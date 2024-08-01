/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{net::IpAddr, sync::Arc};

use ahash::AHashMap;
use leptos::*;
use leptos_router::use_navigate;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputSwitch, InputText, TextArea},
            select::Select,
            stacked_input::StackedInput,
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        form::FormValue,
        http::{Error, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, SelectType, Source, Transformer, Type, Validator},
    },
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "lowercase")]
pub enum Response {
    Accept {
        modifications: Vec<Modification>,
    },
    Replace {
        message: String,
        modifications: Vec<Modification>,
    },
    Reject {
        reason: String,
    },
    Discard,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "camelCase")]
pub enum Modification {
    SetEnvelope { name: Envelope, value: String },
    AddHeader { name: String, value: String },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Envelope {
    From,
    To,
    ByTimeAbsolute,
    ByTimeRelative,
    ByMode,
    ByTrace,
    Notify,
    Orcpt,
    Ret,
    Envid,
}

#[component]
pub fn SpamTest() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let mut data = expect_context::<Arc<Schemas>>().build_form("spam-test");
    data.apply_defaults(false);
    let data = data.into_signal();

    let save_changes = create_action(
        move |(variables, message): &(AHashMap<String, String>, String)| {
            let auth = auth.get();
            let variables = variables.clone();
            let message = message.clone();

            async move {
                set_pending.set(true);
                let result = HttpRequest::post("/api/sieve/spam-filter")
                    .with_authorization(&auth)
                    .with_parameters(variables)
                    .with_raw_body(message)
                    .send::<Response>()
                    .await;

                set_pending.set(false);

                match result {
                    Ok(
                        Response::Accept { modifications }
                        | Response::Replace { modifications, .. },
                    ) => {
                        alert.set(
                            Alert::success("Message accepted by filter")
                                .with_details_list(modifications.into_iter().filter_map(
                                    |modification| {
                                        if let Modification::AddHeader { name, value } =
                                            modification
                                        {
                                            Some(format!("{name}: {value}"))
                                        } else {
                                            None
                                        }
                                    },
                                ))
                                .without_timeout(),
                        );
                    }
                    Ok(Response::Reject { reason }) => {
                        alert
                            .set(Alert::warning("Message rejected by filter").with_details(reason));
                    }
                    Ok(Response::Discard) => {
                        alert.set(Alert::warning("Message discarded by filter"));
                    }
                    Err(Error::Unauthorized) => {
                        use_navigate()("/login", Default::default());
                    }
                    Err(err) => {
                        alert.set(Alert::from(err));
                    }
                }
            }
        },
    );

    view! {
        <Form title="">

            <FormSection title="Test SPAM Filter".to_string()>
                <FormItem label="IP Address" tooltip="IP address of the remote SMTP server">
                    <InputText element=FormElement::new("remote_ip", data)/>
                </FormItem>
                <FormItem label="PTR Address" is_optional=true tooltip="Reverse IP lookup">
                    <InputText element=FormElement::new("iprev.ptr", data)/>
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

            <FormSection title="Authentication Results".to_string()>
                <FormItem label="SPF" tooltip="SPF authentication results">
                    <Select element=FormElement::new("spf.result", data)/>
                </FormItem>
                <FormItem label="SPF EHLO" tooltip="SPF EHLO authentication results">
                    <Select element=FormElement::new("spf_ehlo.result", data)/>
                </FormItem>
                <FormItem label="DKIM" tooltip="DKIM authentication results">
                    <Select element=FormElement::new("dkim.result", data)/>
                </FormItem>
                <FormItem
                    label="DKIM Domains"
                    tooltip="Domain names passing DKIM validation"
                    is_optional=true
                >
                    <InputText element=FormElement::new("dkim.domains", data)/>
                </FormItem>
                <FormItem label="ARC" tooltip="ARC authentication results">
                    <Select element=FormElement::new("arc.result", data)/>
                </FormItem>
                <FormItem label="DMARC" tooltip="DMARC authentcation results">
                    <Select element=FormElement::new("dmarc.result", data)/>
                </FormItem>
                <FormItem label="DMARC Policy" tooltip="DMARC policy of the sender domain">
                    <Select element=FormElement::new("dmarc.policy", data)/>
                </FormItem>
                <FormItem label="Reverse IP" tooltip="Reverse IP validation results">
                    <Select element=FormElement::new("iprev.result", data)/>
                </FormItem>
            </FormSection>

            <FormSection title="Message".to_string()>
                <FormItem label="Contents" tooltip="Message body">
                    <TextArea element=FormElement::new("message", data)/>
                </FormItem>
                <FormItem label="Parameters" tooltip="SMTP BODY parameter">
                    <Select element=FormElement::new("param.body", data)/>
                </FormItem>
                <FormItem label="">
                    <InputSwitch
                        label="SMTPUTF8"
                        tooltip="Enable SMTPUTF8 support for the message"
                        element=FormElement::new("param.smtputf8", data)
                    />
                </FormItem>
            </FormSection>

            <FormButtonBar>

                <Button
                    text="Test"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                let mut message = String::new();
                                let mut variables = AHashMap::new();
                                for (key, value) in data.values.iter() {
                                    let value = match value {
                                        FormValue::Value(value) if !value.is_empty() => value,
                                        FormValue::Array(values) if !values.is_empty() => {
                                            for (i, value) in values.iter().enumerate() {
                                                variables.insert(format!("{key}_{i}"), value.clone());
                                            }
                                            continue;
                                        }
                                        _ => continue,
                                    };
                                    match key.as_str() {
                                        "message" => {
                                            message.clone_from(value);
                                            continue;
                                        }
                                        "remote_ip" => {
                                            if let Ok(ip) = value.parse::<IpAddr>() {
                                                variables
                                                    .insert(
                                                        "remote_ip.reverse".to_string(),
                                                        to_reverse_name(ip),
                                                    );
                                            }
                                        }
                                        _ => {}
                                    }
                                    variables.insert(key.clone(), value.clone());
                                }
                                save_changes.dispatch((variables, data.value("message").unwrap()));
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

    let save_changes = create_action(move |(train, message): &(String, String)| {
        let auth = auth.get();
        let train = train.clone();
        let message = message.clone();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post("/api/sieve/train")
                .with_authorization(&auth)
                .with_parameter("train", train)
                .with_raw_body(message)
                .send::<Response>()
                .await;

            set_pending.set(false);

            match result {
                Ok(Response::Accept { .. }) => {
                    data.update(|data| {
                        data.reset();
                    });
                    alert.set(Alert::success("Training successful"));
                }
                Ok(Response::Reject { reason }) => {
                    alert.set(Alert::warning("Training failed").with_details(reason));
                }
                Ok(_) => {
                    alert.set(Alert::error("Unexpected server response"));
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
        <Form title="Train SPAM filter" subtitle="Train the SPAM filter classifier">

            <FormSection>
                <FormItem label="Train">
                    <Select element=FormElement::new("train", data)/>
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
                                save_changes
                                    .dispatch((
                                        data.value("train").unwrap(),
                                        data.value("message").unwrap(),
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

fn to_reverse_name(ip: IpAddr) -> String {
    use std::fmt::Write;

    match ip {
        IpAddr::V4(ip) => {
            let mut segments = String::with_capacity(15);
            for octet in ip.octets().iter().rev() {
                if !segments.is_empty() {
                    segments.push('.');
                }
                let _ = write!(&mut segments, "{}", octet);
            }
            segments
        }
        IpAddr::V6(ip) => {
            let mut segments = String::with_capacity(63);
            for segment in ip.segments().iter().rev() {
                for &p in format!("{segment:04x}").as_bytes().iter().rev() {
                    if !segments.is_empty() {
                        segments.push('.');
                    }
                    segments.push(char::from(p));
                }
            }
            segments
        }
    }
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
            .new_field("iprev.ptr")
            .typ(Type::Text)
            .input_check([], [])
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
            .new_field("dkim.domains")
            .typ(Type::Text)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("iprev.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(IPREV_RESULT),
            })
            .default("none")
            .build()
            .new_field("spf.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(SPF_RESULT),
            })
            .default("none")
            .build()
            .new_field("spf_ehlo.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(SPF_RESULT),
            })
            .default("none")
            .build()
            .new_field("arc.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(DKIM_RESULT),
            })
            .default("none")
            .build()
            .new_field("dkim.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(DKIM_RESULT),
            })
            .default("none")
            .build()
            .new_field("dmarc.result")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(DMARC_RESULT),
            })
            .default("none")
            .build()
            .new_field("dmarc.policy")
            .typ(Type::Select {
                typ: SelectType::Single,
                source: Source::Static(DMARC_POLICY),
            })
            .default("none")
            .build()
            .new_field("param.body")
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

pub static SPF_RESULT: &[(&str, &str)] = &[
    ("pass", "Pass"),
    ("fail", "Fail"),
    ("softfail", "Soft Fail"),
    ("neutral", "Neutral"),
    ("temperror", "Temporary Error"),
    ("permerror", "Permanent Error"),
    ("none", "None"),
];

pub static IPREV_RESULT: &[(&str, &str)] = &[
    ("pass", "Pass"),
    ("fail", "Fail"),
    ("temperror", "Temporary Error"),
    ("permerror", "Permanent Error"),
    ("none", "None"),
];

pub static DKIM_RESULT: &[(&str, &str)] = &[
    ("pass", "Pass"),
    ("fail", "Fail"),
    ("neutral", "Neutral"),
    ("none", "None"),
    ("permerror", "Permanent Error"),
    ("temperror", "Temporary Error"),
];

pub static DMARC_RESULT: &[(&str, &str)] = &[
    ("pass", "Pass"),
    ("fail", "Fail"),
    ("temperror", "Temporary Error"),
    ("permerror", "Permanent Error"),
    ("none", "None"),
];

pub static DMARC_POLICY: &[(&str, &str)] = &[
    ("reject", "Reject"),
    ("quarantine", "Quarantine"),
    ("none", "None"),
];

pub static MAIL_BODY: &[(&str, &str)] = &[
    ("", "Not specified"),
    ("7bit", "7bit"),
    ("8bitmime", "8bit MIME"),
    ("binarymime", "Binary MIME"),
];
