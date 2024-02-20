use std::time::Duration;

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::use_navigate;

use crate::{
    components::main::alert::{Alert, Alerts},
    core::oauth::oauth_authenticate,
    GlobalState, STATE_LOGIN_NAME_KEY, STATE_STORAGE_KEY,
};

#[derive(Clone, Default)]
struct LoginData {
    email: Option<String>,
    password: Option<String>,
    remember_me: bool,
}

#[component]
pub fn Login() -> impl IntoView {
    let stored_login_name = LocalStorage::get(STATE_LOGIN_NAME_KEY).ok();
    let remember_me = stored_login_name.is_some();
    let login_data = create_rw_signal(LoginData {
        remember_me,
        email: stored_login_name.clone(),
        ..Default::default()
    });
    let alert = create_rw_signal(Alert::disabled());
    let state = use_context::<RwSignal<GlobalState>>().unwrap();

    let login_action = create_action(move |(user, password): &(String, String)| {
        let user = user.clone();
        let password = password.clone();
        async move {
            match oauth_authenticate(&user, &password).await {
                Ok(grant) => {
                    let refresh_token = grant.refresh_token.unwrap_or_default();
                    state.update(|state| {
                        state.access_token = grant.access_token.into();
                        state.refresh_token = refresh_token.clone().into();
                        state.is_valid = true;

                        if let Err(err) = SessionStorage::set(STATE_STORAGE_KEY, state.clone()) {
                            log::error!("Failed to save state to session storage: {}", err);
                        }
                    });

                    // Set timer to refresh token
                    if grant.expires_in > 0 && !refresh_token.is_empty() {
                        log::debug!("Next OAuth token refresh in {} seconds.", grant.expires_in);

                        set_timeout(
                            move || {
                                state.update(|state| {
                                    state.is_valid = false;
                                });
                            },
                            Duration::from_secs(grant.expires_in),
                        );
                    }
                    use_navigate()("/manage/accounts", Default::default());
                }
                Err(err) => {
                    alert.set(err);
                }
            }
        }
    });

    view! {
        <Body class="dark:bg-slate-900 bg-gray-100 flex h-full items-center py-16"/>
        <main class="w-full max-w-md mx-auto p-6">
            <div class="mt-7 bg-white border border-gray-200 rounded-xl shadow-sm dark:bg-gray-800 dark:border-gray-700">
                <div class="p-4 sm:p-7">
                    <div class="text-center">
                        <h1 class="block text-2xl font-bold text-gray-800 dark:text-white">
                            Sign in
                        </h1>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            to Stalwart Mail Server
                        </p>
                    </div>

                    <div class="mt-5">
                        <Alerts alert/>
                        <form on:submit=|ev| ev.prevent_default()>
                            <div class="grid gap-y-4">
                                <div>
                                    <label for="email" class="block text-sm mb-2 dark:text-white">
                                        Login
                                    </label>
                                    <div class="relative">
                                        <input
                                            type="text"
                                            id="email"
                                            name="email"
                                            class="py-3 px-4 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                            aria-describedby="email-error"
                                            value=stored_login_name.unwrap_or_default()
                                            on:keyup=move |ev: ev::KeyboardEvent| {
                                                let val = event_target_value(&ev);
                                                login_data
                                                    .update(|v| v.email = val.trim().to_string().into());
                                            }

                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                login_data
                                                    .update(|v| v.email = val.trim().to_string().into());
                                            }
                                        />

                                        <div
                                            class="absolute inset-y-0 end-0 pointer-events-none pe-3"
                                            class:hidden=move || login_data.get().has_user()
                                        >
                                            <svg
                                                class="size-5 text-red-500"
                                                width="16"
                                                height="16"
                                                fill="currentColor"
                                                viewBox="0 0 16 16"
                                                aria-hidden="true"
                                            >
                                                <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM8 4a.905.905 0 0 0-.9.995l.35 3.507a.552.552 0 0 0 1.1 0l.35-3.507A.905.905 0 0 0 8 4zm.002 6a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"></path>
                                            </svg>
                                        </div>
                                    </div>
                                    <p
                                        class="text-xs text-red-600 mt-2"
                                        id="email-error"
                                        class:hidden=move || login_data.get().has_user()
                                    >
                                        Please enter your account name
                                    </p>
                                </div>
                                <div>
                                    <div class="flex justify-between items-center">
                                        <label
                                            for="password"
                                            class="block text-sm mb-2 dark:text-white"
                                        >
                                            Password
                                        </label>

                                    </div>
                                    <div class="relative">
                                        <input
                                            type="password"
                                            id="password"
                                            name="password"
                                            class="py-3 px-4 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                            aria-describedby="password-error0"
                                            on:keyup=move |ev: ev::KeyboardEvent| {
                                                let val = event_target_value(&ev);
                                                login_data.update(|v| v.password = val.into());
                                            }

                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                login_data.update(|v| v.password = val.into());
                                            }
                                        />

                                        <div
                                            class="absolute inset-y-0 end-0 pointer-events-none pe-3"
                                            class:hidden=move || login_data.get().has_password()
                                        >
                                            <svg
                                                class="size-5 text-red-500"
                                                width="16"
                                                height="16"
                                                fill="currentColor"
                                                viewBox="0 0 16 16"
                                                aria-hidden="true"
                                            >
                                                <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM8 4a.905.905 0 0 0-.9.995l.35 3.507a.552.552 0 0 0 1.1 0l.35-3.507A.905.905 0 0 0 8 4zm.002 6a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"></path>
                                            </svg>
                                        </div>
                                    </div>
                                    <p
                                        class="hidden text-xs text-red-600 mt-2"
                                        id="password-error"
                                        class:hidden=move || login_data.get().has_password()
                                    >
                                        Please enter a valid password
                                    </p>
                                </div>
                                <div class="flex items-center">
                                    <div class="flex">
                                        <input
                                            id="remember-me"
                                            name="remember-me"
                                            type="checkbox"
                                            class="shrink-0 mt-0.5 border-gray-200 rounded text-blue-600 focus:ring-blue-500 dark:bg-gray-800 dark:border-gray-700 dark:checked:bg-blue-500 dark:checked:border-blue-500 dark:focus:ring-offset-gray-800"
                                            prop:checked=remember_me
                                            on:input=move |_| {
                                                login_data
                                                    .update(|t| {
                                                        t.remember_me = !t.remember_me;
                                                    })
                                            }
                                        />

                                    </div>
                                    <div class="ms-3">
                                        <label for="remember-me" class="text-sm dark:text-white">
                                            Remember me
                                        </label>
                                    </div>
                                </div>

                                <button
                                    type="submit"
                                    class="w-full py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                    on:click=move |_| {
                                        let data = login_data.get();
                                        match (data.email, data.password) {
                                            (
                                                Some(email),
                                                Some(password),
                                            ) if !email.is_empty() && !password.is_empty() => {
                                                if data.remember_me {
                                                    if let Err(err) = LocalStorage::set(
                                                        STATE_LOGIN_NAME_KEY,
                                                        &email,
                                                    ) {
                                                        log::error!(
                                                            "Failed to save login name to local storage: {}", err
                                                        );
                                                    }
                                                } else {
                                                    LocalStorage::delete(STATE_LOGIN_NAME_KEY);
                                                }
                                                login_action.dispatch((email, password));
                                            }
                                            _ => {
                                                login_data
                                                    .update(|t| {
                                                        t.email = t.email.take().unwrap_or_default().into();
                                                        t.password = t.password.take().unwrap_or_default().into();
                                                    });
                                            }
                                        }
                                    }
                                >

                                    Sign in
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </main>
    }
}

impl LoginData {
    fn has_user(&self) -> bool {
        self.email.as_ref().map_or(true, |e| !e.is_empty())
    }

    fn has_password(&self) -> bool {
        self.password.as_ref().map_or(true, |e| !e.is_empty())
    }
}
