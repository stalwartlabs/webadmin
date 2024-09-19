/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{str::FromStr, sync::Arc};

use leptos::*;
use leptos_router::use_navigate;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputText, TextArea},
            select::Select,
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        form::FormData,
        http::{self, Error, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, SelectType, Source, Transformer, Type, Validator},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum EncryptionType {
    PGP {
        algo: Algorithm,
        certs: String,
    },
    SMIME {
        algo: Algorithm,
        certs: String,
    },
    #[default]
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Algorithm {
    Aes128,
    Aes256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMethod {
    PGP,
    SMIME,
}

#[component]
pub fn ManageCrypto() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let show_totp = create_rw_signal(false);
    let fetch_crypto = create_resource(
        move || (),
        move |_| {
            let auth = auth.get_untracked();

            async move {
                HttpRequest::get("/api/account/crypto")
                    .with_authorization(&auth)
                    .send::<EncryptionType>()
                    .await
            }
        },
    );

    let (pending, set_pending) = create_signal(false);

    let data = expect_context::<Arc<Schemas>>()
        .build_form("crypto-at-rest")
        .into_signal();

    let save_changes = create_action(move |(changes, password): &(EncryptionType, String)| {
        let changes = changes.clone();
        let password = password.clone();
        let auth = auth.get();

        async move {
            let is_disable = matches!(changes, EncryptionType::Disabled);
            set_pending.set(true);
            let result = HttpRequest::post("/api/account/crypto")
                .with_basic_authorization(auth.username.as_str(), &password)
                .with_base_url(&auth)
                .with_body(changes)
                .unwrap()
                .send::<Option<u32>>()
                .await
                .map(|_| ());
            set_pending.set(false);

            alert.set(match result {
                Ok(_) => {
                    show_totp.set(false);

                    if !is_disable {
                        Alert::success("Encryption-at-rest enabled").with_details(concat!(
                            "Automatic encryption of plain text messages has been enabled. ",
                            "From now on all incoming plain-text messages will be encrypted ",
                            "before they reach your mailbox."
                        ))
                    } else {
                        Alert::success("Encryption-at-rest disabled").with_details(concat!(
                            "Automatic encryption of plain text messages has been disabled. ",
                            "From now on all incoming messages will be stored ",
                            "in their original form."
                        ))
                    }
                    .without_timeout()
                }
                Err(Error::Unauthorized) => Alert::warning("Incorrect password")
                    .with_details("The password you entered is incorrect"),
                Err(Error::TotpRequired) => {
                    show_totp.set(true);
                    return;
                }
                Err(err) => {
                    show_totp.set(false);
                    Alert::from(err)
                }
            });
        }
    });

    view! {
        <Form
            title="Encryption-at-rest"
            subtitle="Automatically encrypt plain-text messages before they reach your mailbox."
        >

            <Transition fallback=Skeleton set_pending>

                {move || match fetch_crypto.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Ok(crypto)) => {
                        data.update(|data| {
                            data.from_encryption_params(&crypto);
                        });
                        let has_no_crypto = create_memo(move |_| {
                            data.get().value::<EncryptionMethod>("type").is_none()
                        });
                        Some(
                            view! {
                                <FormSection>
                                    <Show when=move || show_totp.get()>
                                        <FormItem label="TOTP Token">
                                            <InputText element=FormElement::new("totp-code", data)/>
                                        </FormItem>
                                    </Show>

                                    <Show when=move || !show_totp.get()>
                                        <FormItem label="Current Password">
                                            <InputPassword element=FormElement::new("password", data)/>
                                        </FormItem>
                                        <FormItem
                                            label="Encryption type"
                                            tooltip="Whether to use OpenPGP or S/MIME for encryption."
                                        >
                                            <Select element=FormElement::new("type", data)/>
                                        </FormItem>

                                        <FormItem
                                            label="Algorithm"
                                            tooltip="The encryption algorithms to use"
                                            hide=has_no_crypto
                                        >
                                            <Select element=FormElement::new("algo", data)/>

                                        </FormItem>

                                        <FormItem
                                            label="Certificates"
                                            tooltip="The armored OpenPGP certificate or S/MIME certificate in PEM format."
                                            hide=has_no_crypto
                                        >
                                            <TextArea element=FormElement::new("certs", data)/>
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
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if let Some(changes) = data.to_encryption_params() {
                                save_changes
                                    .dispatch((
                                        changes,
                                        match (
                                            data.value::<String>("password").unwrap_or_default(),
                                            data.value::<String>("totp-code"),
                                        ) {
                                            (password, Some(totp)) => format!("{}${}", password, totp),
                                            (password, None) => password,
                                        },
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

#[allow(clippy::wrong_self_convention)]
impl FormData {
    fn from_encryption_params(&mut self, params: &EncryptionType) {
        match params {
            EncryptionType::PGP { algo, certs } => {
                self.set("type", EncryptionMethod::PGP.as_str());
                self.set("algo", algo.as_str());
                self.set("certs", certs);
            }
            EncryptionType::SMIME { algo, certs } => {
                self.set("type", EncryptionMethod::SMIME.as_str());
                self.set("algo", algo.as_str());
                self.set("certs", certs);
            }
            EncryptionType::Disabled => {
                self.set("type", "");
            }
        }
    }

    fn to_encryption_params(&mut self) -> Option<EncryptionType> {
        if self.validate_form() {
            match self.value::<EncryptionMethod>("type") {
                Some(EncryptionMethod::PGP) => EncryptionType::PGP {
                    algo: self.value("algo").unwrap(),
                    certs: self.value("certs").unwrap(),
                },
                Some(EncryptionMethod::SMIME) => EncryptionType::SMIME {
                    algo: self.value("algo").unwrap(),
                    certs: self.value("certs").unwrap(),
                },
                None => EncryptionType::Disabled,
            }
            .into()
        } else {
            None
        }
    }
}

impl Builder<Schemas, ()> {
    pub fn build_crypto(self) -> Self {
        const METHODS: &[(&str, &str)] = &[
            (EncryptionMethod::PGP.as_str(), "OpenPGP"),
            (EncryptionMethod::SMIME.as_str(), "S/MIME"),
            ("", "Disabled"),
        ];
        const ALGOS: &[(&str, &str)] = &[
            (Algorithm::Aes128.as_str(), "AES-128"),
            (Algorithm::Aes256.as_str(), "AES-256"),
        ];

        self.new_schema("crypto-at-rest")
            .new_field("type")
            .typ(Type::Select {
                source: Source::Static(METHODS),
                typ: SelectType::Single,
            })
            .default("")
            .build()
            .new_field("algo")
            .typ(Type::Select {
                source: Source::Static(ALGOS),
                typ: SelectType::Single,
            })
            .default(Algorithm::Aes256.as_str())
            .display_if_eq(
                "type",
                [
                    EncryptionMethod::PGP.as_str(),
                    EncryptionMethod::SMIME.as_str(),
                ],
            )
            .build()
            .new_field("certs")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .display_if_eq(
                "type",
                [
                    EncryptionMethod::PGP.as_str(),
                    EncryptionMethod::SMIME.as_str(),
                ],
            )
            .build()
            .new_field("password")
            .typ(Type::Text)
            .input_check([], [Validator::Required])
            .build()
            .new_field("totp-code")
            .input_check([Transformer::Trim], [])
            .build()
            .build()
    }
}

impl EncryptionMethod {
    pub const fn as_str(&self) -> &'static str {
        match self {
            EncryptionMethod::PGP => "pgp",
            EncryptionMethod::SMIME => "smime",
        }
    }
}

impl FromStr for EncryptionMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pgp" => Ok(EncryptionMethod::PGP),
            "smime" => Ok(EncryptionMethod::SMIME),
            _ => Err(()),
        }
    }
}

impl Algorithm {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Algorithm::Aes128 => "aes128",
            Algorithm::Aes256 => "aes256",
        }
    }
}

impl FromStr for Algorithm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aes128" => Ok(Algorithm::Aes128),
            "aes256" => Ok(Algorithm::Aes256),
            _ => Err(()),
        }
    }
}
