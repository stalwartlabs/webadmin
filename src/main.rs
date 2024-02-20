use std::{sync::Arc, time::Duration};

use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use thaw::*;

use crate::{
    components::main::manage::ManagePage,
    core::oauth::oauth_refresh_token,
    pages::{directory::accounts::list::AccountList, login::Login, notfound::NotFound},
};

pub mod components;
pub mod core;
pub mod pages;

pub const STATE_STORAGE_KEY: &str = "webadmin_state";
pub const STATE_LOGIN_NAME_KEY: &str = "webadmin_login_name";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalState {
    pub access_token: Arc<String>,
    pub refresh_token: Arc<String>,
    pub is_valid: bool,
}

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
pub fn App() -> impl IntoView {
    let is_routing = create_rw_signal(false);
    let set_is_routing = SignalSetter::map(move |is_routing_data| {
        is_routing.set(is_routing_data);
    });

    let state =
        create_rw_signal(SessionStorage::get::<GlobalState>(STATE_STORAGE_KEY).unwrap_or_default());
    provide_meta_context();
    provide_context(state);

    let _refresh_token_resource = create_resource(state, move |changed_state| {
        let changed_state = changed_state.clone();

        async move {
            if !changed_state.is_valid && !changed_state.refresh_token.is_empty() {
                if let Some(grant) = oauth_refresh_token(&changed_state.refresh_token).await {
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
                }
            }
        }
    });

    view! {
        <Router set_is_routing>
            <TheProvider>
                <TheRouter state is_routing/>
            </TheProvider>
        </Router>
    }
}

#[component]
fn TheRouter(state: RwSignal<GlobalState>, is_routing: RwSignal<bool>) -> impl IntoView {
    let loading_bar = use_loading_bar();
    _ = is_routing.watch(move |is_routing| {
        if *is_routing {
            loading_bar.start();
        } else {
            loading_bar.finish();
        }
    });
    let is_logged_in = move || state.get().is_logged_in();

    view! {
        <Routes>
            <ProtectedRoute
                path="/manage"
                view=ManagePage
                redirect_path="/login"
                condition=is_logged_in
            >
                <ProtectedRoute
                    path="/accounts"
                    view=AccountList
                    redirect_path="/login"
                    condition=is_logged_in
                />

            </ProtectedRoute>
            <Route path="/" view=Login/>
            <Route path="/login" view=Login/>
            <Route path="/*any" view=NotFound/>
        </Routes>
    }
}

#[component]
fn TheProvider(children: Children) -> impl IntoView {
    fn use_query_value(key: &str) -> Option<String> {
        let query_map = use_query_map();
        query_map.with_untracked(|query| query.get(key).cloned())
    }
    let theme = use_query_value("theme").map_or_else(Theme::light, |name| {
        if name == "light" {
            Theme::light()
        } else if name == "dark" {
            Theme::dark()
        } else {
            Theme::light()
        }
    });
    let theme = create_rw_signal(theme);

    view! {
        <div id="root">
            <ThemeProvider theme>
                <GlobalStyle/>
                <MessageProvider>
                    <LoadingBarProvider>{children()}</LoadingBarProvider>
                </MessageProvider>
            </ThemeProvider>
        </div>
        <div id="portal_root"></div>
    }
}

impl GlobalState {
    pub fn is_logged_in(&self) -> bool {
        !self.access_token.is_empty()
    }
}

impl AsRef<GlobalState> for GlobalState {
    fn as_ref(&self) -> &GlobalState {
        self
    }
}
