/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::sync::Arc;

use leptos::*;
use pwhash::sha512_crypt;

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputText},
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        http::{Error, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, Transformer, Type, Validator},
    },
    pages::account::AccountAuthRequest,
};

#[component]
pub fn ChangePassword() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let (pending, set_pending) = create_signal(false);

    let data = expect_context::<Arc<Schemas>>()
        .build_form("change-pass")
        .into_signal();
    let show_totp = create_rw_signal(false);

    let change_password = create_action(move |(old_password, new_password): &(String, String)| {
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let auth = auth.get();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post("/api/account/auth")
                .with_basic_authorization(auth.username.as_str(), &old_password)
                .with_base_url(&auth)
                .with_body(vec![AccountAuthRequest::SetPassword {
                    password: new_password,
                }])
                .unwrap()
                .send::<serde_json::Value>()
                .await;
            set_pending.set(false);

            alert.set(match result {
                Ok(_) => {
                    show_totp.set(false);

                    Alert::success("Password changed")
                        .with_details("Your password has been changed successfully")
                        .without_timeout()
                }
                Err(Error::Unauthorized) => {
                    show_totp.set(false);

                    Alert::warning("Incorrect password")
                        .with_details("The password you entered is incorrect")
                }
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
        <Form title="Change Password" subtitle="Update your account password.">
            <FormSection>
                <Show when=move || !show_totp.get()>
                    <FormItem label="Current Password">
                        <InputPassword element=FormElement::new("old-password", data)/>
                    </FormItem>
                    <FormItem label="New Password">
                        <InputPassword element=FormElement::new("new-password", data)/>
                    </FormItem>
                </Show>

                <Show when=move || show_totp.get()>
                    <FormItem label="TOTP Token">
                        <InputText element=FormElement::new("totp-code", data)/>
                    </FormItem>
                </Show>

            </FormSection>

            <FormButtonBar>

                <Button
                    text="Change Password"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                change_password
                                    .dispatch((
                                        match (
                                            data.value::<String>("old-password").unwrap_or_default(),
                                            data.value::<String>("totp-code"),
                                        ) {
                                            (password, Some(totp)) => format!("{}${}", password, totp),
                                            (password, None) => password,
                                        },
                                        data
                                            .value::<String>("new-password")
                                            .map(|password| sha512_crypt::hash(password).unwrap())
                                            .unwrap(),
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
    pub fn build_password_change(self) -> Self {
        self.new_schema("change-pass")
            .new_field("old-password")
            .typ(Type::Secret)
            .input_check([], [Validator::Required])
            .new_field("new-password")
            .build()
            .new_field("totp-code")
            .input_check([Transformer::Trim], [])
            .build()
            .build()
    }
}
