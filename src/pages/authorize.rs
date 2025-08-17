/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::sync::Arc;

use leptos::*;
use leptos_meta::*;
use leptos_router::{use_params_map, use_query_map};

use crate::{
    components::{
        form::{
            input::{InputPassword, InputText},
            FormElement,
        },
        messages::alert::{use_alerts, Alert, Alerts},
    },
    core::{
        oauth::{
            oauth_device_authentication, oauth_user_authentication, AuthenticationResult,
            OAuthCodeRequest,
        },
        schema::{Builder, Schemas, Transformer, Type, Validator},
    },
};

const BASE_URL: &str = "";

#[component]
pub fn Authorize() -> impl IntoView {
    let alert = use_alerts();
    let query = use_query_map();
    let params = use_params_map();
    let is_device_auth = create_memo(move |_| params.get().get("type").is_none_or(|t| t != "code"));
    let redirect_uri = create_memo(move |_| query.get().get("redirect_uri").cloned());
    let client_id = create_memo(move |_| query.get().get("client_id").cloned());
    let nonce = create_memo(move |_| query.get().get("nonce").cloned());
    let show_totp = create_rw_signal(false);

    let login_action = create_action(
        move |(username, password, request): &(String, String, OAuthCodeRequest)| {
            let username = username.clone();
            let password = password.clone();
            let request = request.clone();
            let state = query.get().get("state").cloned();

            async move {
                match &request {
                    OAuthCodeRequest::Code { redirect_uri, .. } => {
                        match oauth_user_authentication(BASE_URL, &username, &password, &request)
                            .await
                        {
                            AuthenticationResult::Success(response) => {
                                let redirect_uri = redirect_uri.as_deref().unwrap_or_default();
                                let sep = if redirect_uri.contains('?') { '&' } else { '?' };
                                let url = if let Some(state) = state {
                                    format!(
                                        "{}{}code={}&state={}",
                                        redirect_uri,
                                        sep,
                                        response.code,
                                        state
                                    )
                                } else {
                                    format!(
                                        "{}{}code={}",
                                        redirect_uri,
                                        sep,
                                        response.code
                                    )
                                };

                                if let Err(err) = window().location().set_href(&url) {
                                    log::error!("Failed to redirect to {url}: {err:?}");
                                }
                            }
                            AuthenticationResult::TotpRequired => {
                                show_totp.set(true);
                            }
                            AuthenticationResult::Error(err) => {
                                alert.set(err);
                            }
                        }
                    }
                    OAuthCodeRequest::Device { .. } => {
                        let message = match oauth_device_authentication(
                            BASE_URL, &username, &password, &request,
                        )
                        .await
                        {
                            AuthenticationResult::Success(true) => {
                                Alert::success("Device authenticated")
                                    .with_details("You have successfully authenticated your device")
                                    .without_timeout()
                            }
                            AuthenticationResult::Success(false) => {
                                Alert::warning("Device authentication failed")
                                    .with_details("The code you entered is invalid or has expired")
                            }
                            AuthenticationResult::TotpRequired => {
                                show_totp.set(true);
                                return;
                            }
                            AuthenticationResult::Error(err) => err,
                        };

                        alert.set(message);
                    }
                }
            }
        },
    );

    let data = expect_context::<Arc<Schemas>>()
        .build_form("authorize")
        .with_value(
            "code",
            query
                .get_untracked()
                .get("code")
                .cloned()
                .unwrap_or_default(),
        )
        .into_signal();

    view! {
        <Body class="dark:bg-slate-900 bg-gray-100 flex h-full items-center py-16"/>
        <main class="w-full max-w-md mx-auto p-6">
            <div class="mt-7 bg-white border border-gray-200 rounded-xl shadow-sm dark:bg-gray-800 dark:border-gray-700">
                <div class="p-4 sm:p-7">
                    <div class="text-center p-6">
                        <img src="/logo.svg"/>

                    </div>

                    <div class="mt-5">
                        <Alerts/>
                        <form on:submit=|ev| ev.prevent_default()>
                            <div class="grid gap-y-4">

                                <Show when=move || !show_totp.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            Login
                                        </label>
                                        <InputText
                                            placeholder="user@example.org"
                                            element=FormElement::new("login", data)
                                        />
                                    </div>
                                    <div>
                                        <div class="flex justify-between items-center">
                                            <label class="block text-sm mb-2 dark:text-white">
                                                Password
                                            </label>

                                        </div>
                                        <InputPassword element=FormElement::new("password", data)/>
                                    </div>
                                </Show>

                                <Show when=move || show_totp.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            TOTP Token
                                        </label>
                                        <InputText element=FormElement::new("totp-code", data)/>
                                    </div>
                                </Show>

                                <Show when=move || is_device_auth.get() && !show_totp.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            Code
                                        </label>
                                        <InputText
                                            placeholder="Enter the device code"
                                            element=FormElement::new("code", data)
                                        />
                                    </div>
                                </Show>

                                <button
                                    type="submit"
                                    class="w-full py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                    on:click=move |_| {
                                        let is_auth_flow = !is_device_auth.get();
                                        let redirect_uri = match redirect_uri.get() {
                                            Some(redirect_uri) if redirect_uri.starts_with("http:") => {
                                                alert
                                                    .set(
                                                        Alert::error(
                                                            "Invalid redirect_uri parameter, must be a valid HTTPS URL",
                                                        ),
                                                    );
                                                return;
                                            }
                                            None if is_auth_flow => {
                                                alert
                                                    .set(
                                                        Alert::error("Missing redirect_uri in query parameters"),
                                                    );
                                                return;
                                            }
                                            redirect_uri => redirect_uri,
                                        };
                                        let client_id = client_id.get();
                                        let nonce = nonce.get();
                                        data.update(|data| {
                                            if is_auth_flow {
                                                data.set("code", "none");
                                            }
                                            if data.validate_form() {
                                                let login = data
                                                    .value::<String>("login")
                                                    .unwrap_or_default();
                                                let password = match (
                                                    data.value::<String>("password").unwrap_or_default(),
                                                    data.value::<String>("totp-code"),
                                                ) {
                                                    (password, Some(totp)) => format!("{}${}", password, totp),
                                                    (password, None) => password,
                                                };
                                                let request = if is_auth_flow {
                                                    OAuthCodeRequest::Code {
                                                        client_id: client_id.unwrap_or_default(),
                                                        redirect_uri,
                                                        nonce,
                                                    }
                                                } else {
                                                    OAuthCodeRequest::Device {
                                                        code: data.value::<String>("code").unwrap_or_default(),
                                                    }
                                                };
                                                login_action.dispatch((login, password, request));
                                            }
                                        });
                                    }
                                >

                                    Authorize
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </main>
    }
}

impl Builder<Schemas, ()> {
    pub fn build_authorize(self) -> Self {
        self.new_schema("authorize")
            .new_field("login")
            .typ(Type::Input)
            .input_check(
                [Transformer::RemoveSpaces, Transformer::Lowercase],
                [Validator::Required],
            )
            .build()
            .new_field("password")
            .typ(Type::Secret)
            .input_check([], [Validator::Required])
            .build()
            .new_field("code")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("totp-code")
            .input_check([Transformer::Trim], [])
            .build()
            .build()
    }
}
