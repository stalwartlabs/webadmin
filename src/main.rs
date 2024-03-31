#![allow(unstable_name_collisions)]
use core::schema::Schemas;
use std::{sync::Arc, time::Duration};

use components::{
    icon::{IconDocumentChartBar, IconQueueList, IconUserGroup},
    layout::MenuItem,
};
use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{
    components::{
        layout::{Layout, LayoutBuilder},
        messages::{alert::init_alerts, modal::init_modals},
    },
    core::oauth::{oauth_refresh_token, AuthToken},
    pages::{
        config::{edit::SettingsEdit, list::SettingsList},
        directory::{
            domains::{edit::DomainCreate, list::DomainList},
            principals::{edit::PrincipalEdit, list::PrincipalList},
        },
        login::Login,
        notfound::NotFound,
        queue::{
            messages::{list::QueueList, manage::QueueManage},
            reports::{display::ReportDisplay, list::ReportList},
        },
        reports::{display::IncomingReportDisplay, list::IncomingReportList},
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
    provide_context(build_schemas());
    init_alerts();
    init_modals();

    // Create a resource to refresh the OAuth token
    let _refresh_token_resource = create_resource(auth_token, move |changed_auth_token| {
        let changed_auth_token = changed_auth_token.clone();

        async move {
            if !changed_auth_token.is_valid && !changed_auth_token.refresh_token.is_empty() {
                if let Some(grant) = oauth_refresh_token(
                    &changed_auth_token.base_url,
                    &changed_auth_token.refresh_token,
                )
                .await
                {
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

    let is_logged_in = create_memo(move |_| auth_token.get().is_logged_in());

    view! {
        <Router>
            <Routes>
                <ProtectedRoute
                    path="/manage"
                    view=move || {
                        view! { <Layout menu_items=LayoutBuilder::manage()/> }
                    }

                    redirect_path="/login"
                    condition=move || is_logged_in.get()
                >
                    <ProtectedRoute
                        path="/directory/domains"
                        view=DomainList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/directory/domains/edit"
                        view=DomainCreate
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />

                    <ProtectedRoute
                        path="/directory/:object"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/directory/:object/:id?/edit"
                        view=PrincipalEdit
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/queue/messages"
                        view=QueueList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/queue/message/:id"
                        view=QueueManage
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/queue/reports"
                        view=ReportList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/queue/report/:id"
                        view=ReportDisplay
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/reports/:object"
                        view=IncomingReportList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/reports/:object/:id"
                        view=IncomingReportDisplay

                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                </ProtectedRoute>
                <ProtectedRoute
                    path="/settings"
                    view=move || {
                        view! { <Layout menu_items=LayoutBuilder::settings()/> }
                    }

                    redirect_path="/login"
                    condition=move || is_logged_in.get()
                >
                    <ProtectedRoute
                        path="/:object"
                        view=SettingsList
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/:object/:id?/edit"
                        view=SettingsEdit
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
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

impl LayoutBuilder {
    pub fn manage() -> Vec<MenuItem> {
        LayoutBuilder::new("/manage")
            .create("Directory")
            .icon(view! { <IconUserGroup/> })
            .create("Accounts")
            .route("/directory/accounts")
            .insert()
            .create("Groups")
            .route("/directory/groups")
            .insert()
            .create("Lists")
            .route("/directory/lists")
            .insert()
            .create("Domains")
            .route("/directory/domains")
            .insert()
            .insert()
            .create("Queues")
            .icon(view! { <IconQueueList/> })
            .create("Messages")
            .route("/queue/messages")
            .insert()
            .create("Reports")
            .route("/queue/reports")
            .insert()
            .insert()
            .create("Reports")
            .icon(view! { <IconDocumentChartBar/> })
            .create("DMARC Aggregate")
            .route("/reports/dmarc")
            .insert()
            .create("TLS Aggregate")
            .route("/reports/tls")
            .insert()
            .create("Failures")
            .route("/reports/arf")
            .insert()
            .insert()
            .menu_items
    }
}

pub fn build_schemas() -> Arc<Schemas> {
    Schemas::builder()
        .build_login()
        //.build_principals()
        //.build_domains()
        .build_store()
        .build_directory()
        .build_authentication()
        .build_storage()
        .build_tls()
        .build_server()
        .build_listener()
        .build_tracing()
        .build_smtp_inbound()
        .build_smtp_outbound()
        .build_mail_auth()
        .build_jmap()
        .build_imap()
        .build_sieve()
        .build_spam_lists()
        .build()
        .into()
}
