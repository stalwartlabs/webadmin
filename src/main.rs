#![allow(unstable_name_collisions)]
use std::time::Duration;

use components::layout::MenuItem;
use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{
    components::{
        layout::Layout,
        messages::{alert::init_alerts, modal::init_modals},
    },
    core::oauth::{oauth_refresh_token, AuthToken},
    pages::{
        directory::{
            domains::{edit::DomainCreate, list::DomainList},
            principals::{edit::PrincipalEdit, list::PrincipalList},
            Type,
        },
        login::Login,
        notfound::NotFound,
        queue::{
            messages::{list::QueueList, manage::QueueManage},
            reports::{display::ReportDisplay, list::ReportList},
        },
        reports::{display::IncomingReportDisplay, list::IncomingReportList, ReportType},
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
    init_modals();

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
                        path="/directory/accounts"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/account/:id?"
                        view=move || view! { <PrincipalEdit selected_type=Type::Individual/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/groups"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/group/:id?"
                        view=move || view! { <PrincipalEdit selected_type=Type::Group/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />

                    <ProtectedRoute
                        path="/directory/lists"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/list/:id?"
                        view=move || view! { <PrincipalEdit selected_type=Type::List/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/domains"
                        view=DomainList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/directory/domain"
                        view=DomainCreate
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/queue/messages"
                        view=QueueList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/queue/message/:id"
                        view=QueueManage
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/queue/reports"
                        view=ReportList
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/queue/report/:id"
                        view=ReportDisplay
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/dmarc"
                        view=move || view! { <IncomingReportList report_type=ReportType::Dmarc/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/tls"
                        view=move || view! { <IncomingReportList report_type=ReportType::Tls/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/arf"
                        view=move || view! { <IncomingReportList report_type=ReportType::Arf/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/dmarc/:id"
                        view=move || {
                            view! { <IncomingReportDisplay report_type=ReportType::Dmarc/> }
                        }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/tls/:id"
                        view=move || view! { <IncomingReportDisplay report_type=ReportType::Tls/> }
                        redirect_path="/login"
                        condition=is_logged_in
                    />
                    <ProtectedRoute
                        path="/reports/arf/:id"
                        view=move || view! { <IncomingReportDisplay report_type=ReportType::Arf/> }
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
                MenuItem::child("Accounts", "/manage/directory/accounts")
                    .with_match_route("/manage/directory/account"),
                MenuItem::child("Groups", "/manage/directory/groups")
                    .with_match_route("/manage/directory/group"),
                MenuItem::child("Lists", "/manage/directory/lists")
                    .with_match_route("/manage/directory/list"),
                MenuItem::child("Domains", "/manage/directory/domains")
                    .with_match_route("/manage/directory/domain"),
            ],
        ),
        MenuItem::parent_with_icon(
            "Queues",
            "queue",
            vec![
                MenuItem::child("Messages", "/manage/queue/messages")
                    .with_match_route("/manage/queue/message"),
                MenuItem::child("Reports", "/manage/queue/reports")
                    .with_match_route("/manage/queue/report"),
            ],
        ),
        MenuItem::parent_with_icon(
            "Reports",
            "report",
            vec![
                MenuItem::child("DMARC Aggregate", "/manage/reports/dmarc"),
                MenuItem::child("TLS Aggregate", "/manage/reports/tls"),
                MenuItem::child("Failures", "/manage/reports/arf"),
            ],
        ),
    ]
}
