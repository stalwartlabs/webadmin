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
use leptos_router::use_navigate;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            button::Button, input::InputText, Form, FormButtonBar, FormElement, FormItem,
            FormSection,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        http::{Error, HttpRequest, ManagementApiError},
        oauth::use_authorization,
        schema::{Builder, Schemas, Transformer, Type, Validator},
    },
};

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
pub fn DomainCreate() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let data = expect_context::<Arc<Schemas>>()
        .build_form("domains")
        .into_signal();

    let save_changes = create_action(move |name: &String| {
        let auth = auth.get();
        let name = name.clone();

        async move {
            set_pending.set(true);
            let mut result = HttpRequest::post(format!("/api/domain/{name}"))
                .with_authorization(&auth)
                .send::<()>()
                .await
                .map(|_| ());

            // Create DKIM keys
            if result.is_ok() {
                for algo in [Algorithm::Ed25519, Algorithm::Rsa] {
                    result = HttpRequest::post("/api/dkim")
                        .with_authorization(&auth)
                        .with_body(DkimSignature {
                            algorithm: algo,
                            domain: name.clone(),
                            ..Default::default()
                        })
                        .unwrap()
                        .send::<()>()
                        .await
                        .map(|_| ());

                    if !matches!(
                        result,
                        Ok(_) | Err(Error::Server(ManagementApiError::FieldAlreadyExists { .. }))
                    ) {
                        break;
                    }
                }
            }

            set_pending.set(false);

            match result {
                Ok(_) | Err(Error::Server(ManagementApiError::FieldAlreadyExists { .. })) => {
                    use_navigate()(
                        &format!("/manage/directory/domains/{name}/view"),
                        Default::default(),
                    );
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="Create domain" subtitle="Create a new local domain name">

            <FormSection>
                <FormItem label="Domain name">
                    <InputText
                        placeholder="example.org"

                        element=FormElement::new("domain", data)
                    />
                </FormItem>

            </FormSection>

            <FormButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()("/manage/directory/domains", Default::default());
                    }
                />

                <Button
                    text="Create"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        data.update(|data| {
                            if data.validate_form() {
                                save_changes.dispatch(data.value("domain").unwrap());
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
    pub fn build_domains(self) -> Self {
        self.new_schema("domains")
            .new_field("domain")
            .typ(Type::Input)
            .input_check(
                [Transformer::RemoveSpaces, Transformer::Lowercase],
                [Validator::Required, Validator::IsDomain],
            )
            .build()
            .build()
    }
}
