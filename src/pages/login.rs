/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{sync::Arc, time::Duration};

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use leptos::*;
use leptos_i18n::{t};
use leptos_meta::*;
use leptos_router::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        form::{
            input::{InputPassword, InputText},
            FormElement,
        },
        messages::alert::{use_alerts, Alert, Alerts},
    }, core::{
        oauth::{oauth_authenticate, AuthenticationResult},
        schema::{Builder, Schemas, Transformer, Type, Validator},
        AccessToken, Permissions,
    }, i18n::use_i18n, STATE_LOGIN_NAME_KEY, STATE_STORAGE_KEY
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SavedSession {
    login: String,
    base_url: String,
}

#[component]
pub fn Login() -> impl IntoView {
    let i18n = use_i18n();
    let stored_data: Option<SavedSession> = LocalStorage::get(STATE_LOGIN_NAME_KEY).ok();
    let remember_me = create_rw_signal(stored_data.is_some());
    let show_totp = create_rw_signal(false);
    let alert = use_alerts();
    let auth_token = use_context::<RwSignal<AccessToken>>().unwrap();
    let query = use_query_map();

    let login_action = create_action(
        move |(username, password, base_url): &(String, String, String)| {
            let username = username.clone();
            let password = password.clone();
            let base_url = base_url.clone();

            async move {
                match oauth_authenticate(&base_url, &username, &password).await {
                    AuthenticationResult::Success(response) => {
                        let permissions = Permissions::new(response.permissions);
                        let default_url = permissions.default_url(response.is_enterprise);

                        if default_url.is_empty() {
                            alert.set(Alert::error(
                                "You are not authorized to access this service.",
                            ));
                            return;
                        }

                        let refresh_token = response.grant.refresh_token.unwrap_or_default();
                        auth_token.update(|auth_token| {
                            auth_token.access_token = response.grant.access_token.into();
                            auth_token.refresh_token = refresh_token.clone().into();
                            auth_token.base_url = base_url.clone().into();
                            auth_token.username = username.into();
                            auth_token.is_valid = true;
                            auth_token.permissions = permissions;
                            auth_token.is_enterprise = response.is_enterprise;

                            if let Err(err) =
                                SessionStorage::set(STATE_STORAGE_KEY, auth_token.clone())
                            {
                                log::error!("Failed to save state to session storage: {}", err);
                            }
                        });

                        // Set timer to refresh token
                        if response.grant.expires_in > 0 && !refresh_token.is_empty() {
                            log::debug!(
                                "Next OAuth token refresh in {} seconds.",
                                response.grant.expires_in
                            );

                            set_timeout(
                                move || {
                                    auth_token.update(|auth_token| {
                                        auth_token.is_valid = false;
                                    });
                                },
                                Duration::from_secs(response.grant.expires_in),
                            );
                        }

                        use_navigate()(default_url, Default::default());
                    }
                    AuthenticationResult::TotpRequired => {
                        show_totp.set(true);
                    }
                    AuthenticationResult::Error(err) => {
                        alert.set(err);
                    }
                }
            }
        },
    );

    let (login, base_url) = stored_data.map_or_else(
        || (String::new(), String::new()),
        |session| (session.login, session.base_url),
    );
    let data = expect_context::<Arc<Schemas>>()
        .build_form("login")
        .with_value("base-url", base_url)
        .with_value("login", login)
        .into_signal();
    let has_remote = create_memo(move |_| {
        (query.get().get("remote").is_some()
            || data
                .get()
                .get("base-url")
                .filter(|v| !v.is_empty())
                .is_some())
            && !show_totp.get()
    });

    view! {
        <Body class="dark:bg-slate-900 bg-gray-100 flex h-full items-center py-16" />
        <main class="w-full max-w-md mx-auto p-6">
            <div class="mt-7 bg-white border border-gray-200 rounded-xl shadow-sm dark:bg-gray-800 dark:border-gray-700">
                <div class="p-4 sm:p-7">
                    <div class="text-center p-6">
                        <img src="/logo.svg" />

                    </div>

                    <div class="mt-5">
                        <Alerts />
                        <form on:submit=|ev| ev.prevent_default()>
                            <div class="grid gap-y-4">
                                <Show when=move || has_remote.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            {t!(i18n, host)}
                                        </label>
                                        <InputText
                                            placeholder="https://mail.example.org"
                                            element=FormElement::new("base-url", data)
                                        />
                                    </div>
                                </Show>
                                <Show when=move || !show_totp.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            {t!(i18n, login)}
                                        </label>
                                        <InputText
                                            placeholder="user@example.org"
                                            element=FormElement::new("login", data)
                                        />
                                    </div>
                                    <div>
                                        <div class="flex justify-between items-center">
                                            <label class="block text-sm mb-2 dark:text-white">
                                                {t!(i18n, password)}
                                            </label>

                                        </div>
                                        <InputPassword element=FormElement::new("password", data) />
                                    </div>
                                </Show>
                                <Show when=move || show_totp.get()>
                                    <div>
                                        <label class="block text-sm mb-2 dark:text-white">
                                            {t!(i18n, totp_token)}
                                        </label>
                                        <InputText element=FormElement::new("totp-code", data) />
                                    </div>
                                </Show>
                                <div class="flex items-center">
                                    <div class="flex">
                                        <input
                                            id="remember-me"
                                            name="remember-me"
                                            type="checkbox"
                                            class="shrink-0 mt-0.5 border-gray-200 rounded text-blue-600 focus:ring-blue-500 dark:bg-gray-800 dark:border-gray-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                            prop:checked=remember_me
                                            on:input=move |_| {
                                                remember_me
                                                    .update(|v| {
                                                        *v = !*v;
                                                    })
                                            }
                                        />

                                    </div>
                                    <div class="ms-3">
                                        <label for="remember-me" class="text-sm dark:text-white">
                                            {t!(i18n, remember_me)}
                                        </label>
                                    </div>
                                </div>

                                <button
                                    type="submit"
                                    class="w-full py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                    on:click=move |_| {
                                        data.update(|data| {
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
                                                let base_url = data
                                                    .value::<String>("base-url")
                                                    .unwrap_or_default();
                                                if remember_me.get() {
                                                    if let Err(err) = LocalStorage::set(
                                                        STATE_LOGIN_NAME_KEY,
                                                        SavedSession {
                                                            login: login.clone(),
                                                            base_url: base_url.clone(),
                                                        },
                                                    ) {
                                                        log::error!(
                                                            "Failed to save login name to local storage: {}", err
                                                        );
                                                    }
                                                } else {
                                                    LocalStorage::delete(STATE_LOGIN_NAME_KEY);
                                                }
                                                login_action.dispatch((login, password, base_url));
                                            }
                                        });
                                    }
                                >

                                    {t!(i18n, sign_in)}
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
    pub fn build_login(self) -> Self {
        self.new_schema("login")
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
            .new_field("base-url")
            .input_check([Transformer::Trim], [Validator::IsUrl])
            .build()
            .new_field("totp-code")
            .input_check([Transformer::Trim], [])
            .build()
            .build()
    }
}
