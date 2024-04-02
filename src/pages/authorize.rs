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
        oauth::{oauth_device_authentication, oauth_user_authentication, OAuthCodeRequest},
        schema::{Builder, Schemas, Transformer, Type, Validator},
    },
};

//const BASE_URL: &str = "https://127.0.0.1";
const BASE_URL: &str = "";

#[component]
pub fn Authorize() -> impl IntoView {
    let alert = use_alerts();
    let query = use_query_map();
    let params = use_params_map();
    let is_device_auth = create_memo(move |_| params().get("type").map_or(true, |t| t != "code"));
    let redirect_uri = create_memo(move |_| query.get().get("redirect_uri").cloned());
    let client_id = create_memo(move |_| query.get().get("client_id").cloned());

    let login_action = create_action(
        move |(username, password, request): &(String, String, OAuthCodeRequest)| {
            let username = username.clone();
            let password = password.clone();
            let request = request.clone();

            async move {
                match &request {
                    OAuthCodeRequest::Code {
                        client_id,
                        redirect_uri,
                    } => {
                        match oauth_user_authentication(
                            BASE_URL,
                            &username,
                            &password,
                            client_id,
                            redirect_uri.as_deref(),
                        )
                        .await
                        {
                            Ok(response) => {
                                let url = format!(
                                    "{}?code={}",
                                    redirect_uri.as_deref().unwrap_or_default(),
                                    response.code
                                );

                                if let Err(err) = window().location().set_href(&url) {
                                    log::error!("Failed to redirect to {url}: {err:?}");
                                }
                            }
                            Err(err) => {
                                alert.set(err);
                            }
                        }
                    }
                    OAuthCodeRequest::Device { code } => {
                        alert.set(
                            oauth_device_authentication(BASE_URL, &username, &password, code).await,
                        );
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

                                <div>
                                    <label class="block text-sm mb-2 dark:text-white">Login</label>
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

                                <Show when=is_device_auth>
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
                                        let (redirect_uri, client_id) = if !is_device_auth.get() {
                                            let redirect_uri = redirect_uri.get();
                                            match &redirect_uri {
                                                Some(redirect_uri) if redirect_uri.starts_with("https") => {}
                                                Some(_) => {
                                                    alert
                                                        .set(
                                                            Alert::error(
                                                                "Invalid redirect_uri parameter, must be a valid HTTPS URL",
                                                            ),
                                                        );
                                                    return;
                                                }
                                                None => {
                                                    alert
                                                        .set(
                                                            Alert::error("Missing redirect_uri in query parameters"),
                                                        );
                                                    return;
                                                }
                                            }
                                            (redirect_uri, client_id.get().unwrap_or_default().into())
                                        } else {
                                            (None, None)
                                        };
                                        data.update(|data| {
                                            if client_id.is_some() {
                                                data.set("code", "none");
                                            }
                                            if data.validate_form() {
                                                let login = data
                                                    .value::<String>("login")
                                                    .unwrap_or_default();
                                                let password = data
                                                    .value::<String>("password")
                                                    .unwrap_or_default();
                                                let request = if let Some(client_id) = client_id {
                                                    OAuthCodeRequest::Code {
                                                        client_id,
                                                        redirect_uri,
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
            .build()
    }
}
