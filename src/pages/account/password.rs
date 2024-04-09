/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::sync::Arc;

use leptos::*;
use pwhash::sha512_crypt;

use crate::{
    components::{
        form::{
            button::Button, input::InputPassword, Form, FormButtonBar, FormElement, FormItem,
            FormSection,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        http::{Error, HttpRequest},
        oauth::use_authorization,
        schema::{Builder, Schemas, Type, Validator},
    },
};

#[component]
pub fn ChangePassword() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let (pending, set_pending) = create_signal(false);

    let data = expect_context::<Arc<Schemas>>()
        .build_form("change-pass")
        .into_signal();

    let change_password = create_action(move |(old_password, new_password): &(String, String)| {
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let auth = auth.get();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post("/api/password")
                .with_basic_authorization(auth.username.as_str(), &old_password)
                .with_base_url(&auth)
                .with_raw_body(new_password)
                .send::<()>()
                .await;
            set_pending.set(false);

            alert.set(match result {
                Ok(_) => Alert::success("Password changed")
                    .with_details("Your password has been changed successfully")
                    .without_timeout(),
                Err(Error::Unauthorized) => Alert::warning("Incorrect password")
                    .with_details("The password you entered is incorrect"),
                Err(err) => Alert::from(err),
            });
        }
    });

    view! {
        <Form title="Change Password" subtitle="Update your account password.">
            <FormSection>
                <FormItem label="Current Password">
                    <InputPassword element=FormElement::new("old-password", data)/>
                </FormItem>
                <FormItem label="New Password">
                    <InputPassword element=FormElement::new("new-password", data)/>
                </FormItem>

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
                                        data.value::<String>("old-password").unwrap(),
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
            .build()
    }
}
