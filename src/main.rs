#![allow(unstable_name_collisions)]
use std::time::Duration;

use components::layout::MenuItem;
use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{
    components::{layout::Layout, messages::alert::init_alerts},
    core::oauth::{oauth_refresh_token, AuthToken},
    pages::{
        directory::accounts::{edit::AccountEdit, list::AccountList},
        login::Login,
        notfound::NotFound,
    },
};

pub mod components;
pub mod core;
pub mod pages;

pub const STATE_STORAGE_KEY: &str = "webadmin_state";
pub const STATE_LOGIN_NAME_KEY: &str = "webadmin_login_name";

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
pub fn App() -> impl IntoView {
    let auth_token = create_rw_signal(
        SessionStorage::get::<AuthToken>(STATE_STORAGE_KEY)
            .map(|mut t| {
                // Force token refresh on reload
                t.is_valid = false;
                t
            })
            .unwrap_or_default(),
    );
    provide_meta_context();
    provide_context(auth_token);
    init_alerts();

    // Create a resource to refresh the OAuth token
    let _refresh_token_resource = create_resource(auth_token, move |changed_auth_token| {
        let changed_auth_token = changed_auth_token.clone();

        async move {
            if !changed_auth_token.is_valid && !changed_auth_token.refresh_token.is_empty() {
                if let Some(grant) = oauth_refresh_token(&changed_auth_token.refresh_token).await {
                    let refresh_token = grant.refresh_token.unwrap_or_default();
                    auth_token.update(|auth_token| {
                        auth_token.access_token = grant.access_token.into();
                        auth_token.refresh_token = refresh_token.clone().into();
                        auth_token.is_valid = true;

                        if let Err(err) = SessionStorage::set(STATE_STORAGE_KEY, auth_token.clone())
                        {
                            log::error!(
                                "Failed to save authorization token to session storage: {}",
                                err
                            );
                        }
                    });
                    // Set timer to refresh token
                    if grant.expires_in > 0 && !refresh_token.is_empty() {
                        log::debug!("Next OAuth token refresh in {} seconds.", grant.expires_in);
                        set_timeout(
                            move || {
                                auth_token.update(|auth_token| {
                                    auth_token.is_valid = false;
                                });
                            },
                            Duration::from_secs(grant.expires_in),
                        );
                    }
                }
            }
        }
    });

    let is_logged_in = move || auth_token.get().is_logged_in();

    view! {
        <Router>
            <Routes>
                <ProtectedRoute
                    path="/manage"
                    view=|| {
                        view! { <Layout menu_items=menu_items()/> }
                    }

                    redirect_path="/login"
                    condition=is_logged_in
                >
                    <ProtectedRoute
                        path="/accounts"
                        view=AccountList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/account/:id"
                        view=AccountEdit
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/account"
                        view=AccountEdit
                        redirect_path="/login"
                        condition=is_logged_in
                    />

                </ProtectedRoute>
                <Route path="/" view=Login/>
                <Route path="/login" view=Login/>
                <Route path="/*any" view=NotFound/>
            </Routes>
        </Router>
        <div id="portal_root"></div>
    }
}

pub(crate) fn menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::parent_with_icon(
            "Directory",
            "users",
            vec![
                MenuItem::child("Accounts", "/manage/accounts"),
                MenuItem::child("Groups", "/manage/groups"),
                MenuItem::child("Lists", "/manage/lists"),
                MenuItem::child("Domains", "/manage/domains"),
            ],
        ),
        MenuItem::parent_with_icon(
            "Queues",
            "queue",
            vec![
                MenuItem::child("Messages", "/manage/messages"),
                MenuItem::child("Reports", "/manage/reports"),
            ],
        ),
        MenuItem::parent_with_icon(
            "Nested Test",
            "test",
            vec![
                MenuItem::parent(
                    "Test 1",
                    vec![
                        MenuItem::child("Test 1.1", "/manage/test1"),
                        MenuItem::child("Test 1.2", "/manage/test2"),
                    ],
                ),
                MenuItem::parent(
                    "Test 2",
                    vec![
                        MenuItem::child("Test 2.1", "/manage/test3"),
                        MenuItem::child("Test 2.2", "/manage/test4"),
                    ],
                ),
                MenuItem::parent(
                    "Test 3",
                    vec![
                        MenuItem::child("Test 3.1", "/manage/test5"),
                        MenuItem::child("Test 3.2", "/manage/test6"),
                    ],
                ),
            ],
        ),
        MenuItem::child_with_icon("Documentation", "test", "/test"),
    ]
}
