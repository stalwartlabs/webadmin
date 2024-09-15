/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

#![allow(unstable_name_collisions)]
use core::{schema::Schemas, AccessToken, Permission, Permissions};
use std::{sync::Arc, time::Duration};

use components::{
    icon::{
        IconAdjustmentsHorizontal, IconChartBarSquare, IconClock, IconDocumentChartBar, IconKey,
        IconLockClosed, IconQueueList, IconShieldCheck, IconSignal, IconSquare2x2, IconUserGroup,
        IconWrench,
    },
    layout::MenuItem,
};

use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pages::{
    account::{
        app_password::{AppPasswordCreate, AppPasswords},
        mfa::ManageMfa,
    },
    config::edit::DEFAULT_SETTINGS_URL,
    directory::{dns::DnsDisplay, edit::PrincipalEdit, list::PrincipalList},
    enterprise::{
        dashboard::Dashboard,
        tracing::{display::SpanDisplay, list::SpanList, live::LiveTracing},
        undelete::UndeleteList,
    },
    manage::spam::{SpamTest, SpamTrain},
};

pub static VERSION_NAME: &str = concat!("Stalwart Management UI v", env!("CARGO_PKG_VERSION"),);

use crate::{
    components::{
        layout::{Layout, LayoutBuilder},
        messages::{alert::init_alerts, modal::init_modals},
    },
    core::oauth::oauth_refresh_token,
    pages::{
        account::{crypto::ManageCrypto, password::ChangePassword},
        authorize::Authorize,
        config::{edit::SettingsEdit, list::SettingsList, search::SettingsSearch},
        login::Login,
        manage::{logs::Logs, maintenance::Maintenance},
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
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
pub fn App() -> impl IntoView {
    let auth_token = create_rw_signal(
        SessionStorage::get::<AccessToken>(STATE_STORAGE_KEY)
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
    let _refresh_token_resource = create_resource(
        move || auth_token.get(),
        move |changed_auth_token| {
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

                            if let Err(err) =
                                SessionStorage::set(STATE_STORAGE_KEY, auth_token.clone())
                            {
                                log::error!(
                                    "Failed to save authorization token to session storage: {}",
                                    err
                                );
                            }
                        });
                        // Set timer to refresh token
                        if grant.expires_in > 0 && !refresh_token.is_empty() {
                            log::debug!(
                                "Next OAuth token refresh in {} seconds.",
                                grant.expires_in
                            );
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
        },
    );

    let permissions = create_memo(move |_| {
        let auth_token = auth_token.get();
        if auth_token.is_logged_in() {
            Some(auth_token.permissions().clone())
        } else {
            None
        }
    });

    view! {
        <Router>
            <Routes>
                <ProtectedRoute
                    path="/manage"
                    view=move || {
                        let menu_items = LayoutBuilder::manage(
                            &permissions.get().unwrap_or_default(),
                        );
                        view! { <Layout menu_items=menu_items permissions=permissions/> }
                    }

                    redirect_path="/login"
                    condition=move || permissions.get().is_some()
                >
                    <ProtectedRoute
                        path="/dashboard/:object?"
                        view=Dashboard
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(
                                    false,
                                    |p| {
                                        p.has_access_all(
                                            &[Permission::MetricsList, Permission::MetricsLive],
                                        )
                                    },
                                )
                        }
                    />

                    <ProtectedRoute
                        path="/directory/:object"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(
                                    false,
                                    |p| {
                                        p.has_access_any(
                                            &[
                                                Permission::IndividualList,
                                                Permission::GroupList,
                                                Permission::RoleList,
                                                Permission::TenantList,
                                                Permission::DomainList,
                                                Permission::MailingListList,
                                            ],
                                        )
                                    },
                                )
                        }
                    />

                    <ProtectedRoute
                        path="/directory/:object/:id?/edit"
                        view=PrincipalEdit
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(
                                    false,
                                    |p| {
                                        p.has_access_any(
                                            &[
                                                Permission::IndividualList,
                                                Permission::GroupList,
                                                Permission::RoleList,
                                                Permission::TenantList,
                                                Permission::DomainList,
                                                Permission::MailingListList,
                                            ],
                                        )
                                    },
                                )
                        }
                    />

                    <ProtectedRoute
                        path="/dns/:id/view"
                        view=DnsDisplay
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(
                                    false,
                                    |p| {
                                        p.has_access_all(
                                            &[Permission::DkimSignatureGet, Permission::DomainGet],
                                        )
                                    },
                                )
                        }
                    />

                    <ProtectedRoute
                        path="/queue/messages"
                        view=QueueList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::MessageQueueList) })
                        }
                    />

                    <ProtectedRoute
                        path="/queue/message/:id"
                        view=QueueManage
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::MessageQueueGet) })
                        }
                    />

                    <ProtectedRoute
                        path="/queue/reports"
                        view=ReportList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::OutgoingReportList) })
                        }
                    />

                    <ProtectedRoute
                        path="/queue/report/:id"
                        view=ReportDisplay
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::OutgoingReportGet) })
                        }
                    />

                    <ProtectedRoute
                        path="/reports/:object"
                        view=IncomingReportList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::IncomingReportList) })
                        }
                    />

                    <ProtectedRoute
                        path="/reports/:object/:id"
                        view=IncomingReportDisplay
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::IncomingReportGet) })
                        }
                    />

                    <ProtectedRoute
                        path="/logs"
                        view=Logs
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::LogsView) })
                        }
                    />

                    <ProtectedRoute
                        path="/spam/train"
                        view=SpamTrain
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::SieveRun) })
                        }
                    />

                    <ProtectedRoute
                        path="/spam/test"
                        view=SpamTest
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::SieveRun) })
                        }
                    />

                    <ProtectedRoute
                        path="/maintenance"
                        view=Maintenance
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(
                                    false,
                                    |p| {
                                        p.has_access_any(
                                            &[
                                                Permission::SettingsReload,
                                                Permission::Restart,
                                                Permission::UpdateSpamFilter,
                                                Permission::UpdateWebadmin,
                                            ],
                                        )
                                    },
                                )
                        }
                    />

                    <ProtectedRoute
                        path="/undelete/:id"
                        view=UndeleteList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::Undelete) })
                        }
                    />

                    <ProtectedRoute
                        path="/tracing/span/:id"
                        view=SpanDisplay
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::TracingGet) })
                        }
                    />

                    <ProtectedRoute
                        path="/tracing/live"
                        view=LiveTracing
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::TracingLive) })
                        }
                    />

                    <ProtectedRoute
                        path="/tracing/:object"
                        view=SpanList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::TracingList) })
                        }
                    />

                </ProtectedRoute>
                <ProtectedRoute
                    path="/settings"
                    view=move || {
                        let menu_items = LayoutBuilder::settings(auth_token.get().default_url());
                        view! { <Layout menu_items=menu_items permissions=permissions/> }
                    }

                    redirect_path="/login"
                    condition=move || permissions.get().is_some()
                >
                    <ProtectedRoute
                        path="/:object"
                        view=SettingsList
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::SettingsList) })
                        }
                    />

                    <ProtectedRoute
                        path="/:object/:id?/edit"
                        view=SettingsEdit
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::SettingsUpdate) })
                        }
                    />

                    <ProtectedRoute
                        path="/search"
                        view=SettingsSearch
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::SettingsList) })
                        }
                    />

                </ProtectedRoute>
                <ProtectedRoute
                    path="/account"
                    view=move || {
                        let menu_items = LayoutBuilder::account(
                            &permissions.get().unwrap_or_default(),
                        );
                        view! { <Layout menu_items=menu_items permissions=permissions/> }
                    }

                    redirect_path="/login"
                    condition=move || permissions.get().is_some()
                >
                    <ProtectedRoute
                        path="/crypto"
                        view=ManageCrypto
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::ManageEncryption) })
                        }
                    />

                    <ProtectedRoute
                        path="/password"
                        view=ChangePassword
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::ManagePasswords) })
                        }
                    />

                    <ProtectedRoute
                        path="/mfa"
                        view=ManageMfa
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::ManagePasswords) })
                        }
                    />

                    <ProtectedRoute
                        path="/app-passwords"
                        view=AppPasswords
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::ManagePasswords) })
                        }
                    />

                    <ProtectedRoute
                        path="/app-passwords/edit"
                        view=AppPasswordCreate
                        redirect_path="/login"
                        condition=move || {
                            permissions
                                .get()
                                .map_or(false, |p| { p.has_access(Permission::ManagePasswords) })
                        }
                    />

                </ProtectedRoute>

                <Route path="/" view=Login/>
                <Route path="/login" view=Login/>
                <Route path="/authorize/:type?" view=Authorize/>
                <Route path="/*any" view=NotFound/>
            </Routes>
        </Router>
        <div id="portal_root"></div>
    }
}

impl LayoutBuilder {
    pub fn manage(permissions: &Permissions) -> Vec<MenuItem> {
        LayoutBuilder::new("/manage")
            .create("Dashboard")
            .icon(view! { <IconChartBarSquare/> })
            .create("Overview")
            .route("/dashboard/overview")
            .insert(true)
            .create("Network")
            .route("/dashboard/network")
            .insert(true)
            .create("Security")
            .route("/dashboard/security")
            .insert(true)
            .create("Delivery")
            .route("/dashboard/delivery")
            .insert(true)
            .create("Performance")
            .route("/dashboard/performance")
            .insert(true)
            .insert(permissions.has_access_all(&[Permission::MetricsList, Permission::MetricsLive]))
            .create("Directory")
            .icon(view! { <IconUserGroup/> })
            .create("Accounts")
            .route("/directory/accounts")
            .insert(permissions.has_access(Permission::IndividualList))
            .create("Groups")
            .route("/directory/groups")
            .insert(permissions.has_access(Permission::GroupList))
            .create("Lists")
            .route("/directory/lists")
            .insert(permissions.has_access(Permission::MailingListList))
            .create("Domains")
            .route("/directory/domains")
            .insert(permissions.has_access(Permission::DomainList))
            .create("Roles")
            .route("/directory/roles")
            .insert(permissions.has_access(Permission::RoleList))
            .create("Tenants")
            .route("/directory/tenants")
            .insert(permissions.has_access(Permission::TenantList))
            .insert(permissions.has_access_any(&[
                Permission::IndividualList,
                Permission::GroupList,
                Permission::RoleList,
                Permission::TenantList,
                Permission::DomainList,
                Permission::MailingListList,
            ]))
            .create("Queues")
            .icon(view! { <IconQueueList/> })
            .create("Messages")
            .route("/queue/messages")
            .insert(permissions.has_access(Permission::MessageQueueList))
            .create("Reports")
            .route("/queue/reports")
            .insert(permissions.has_access(Permission::OutgoingReportList))
            .insert(
                permissions.has_access_any(&[
                    Permission::MessageQueueList,
                    Permission::OutgoingReportList,
                ]),
            )
            .create("Reports")
            .icon(view! { <IconDocumentChartBar/> })
            .create("DMARC Aggregate")
            .route("/reports/dmarc")
            .insert(true)
            .create("TLS Aggregate")
            .route("/reports/tls")
            .insert(true)
            .create("Failures")
            .route("/reports/arf")
            .insert(true)
            .insert(permissions.has_access(Permission::IncomingReportList))
            .create("History")
            .icon(view! { <IconClock/> })
            .create("Received Messages")
            .route("/tracing/received")
            .insert(true)
            .create("Delivery Attempts")
            .route("/tracing/delivery")
            .insert(true)
            .insert(permissions.has_access(Permission::TracingList))
            .create("Telemetry")
            .icon(view! { <IconSignal/> })
            .create("Logs")
            .route("/logs")
            .insert(permissions.has_access(Permission::LogsView))
            .create("Live tracing")
            .route("/tracing/live")
            .insert(permissions.has_access(Permission::TracingLive))
            .insert(permissions.has_access_any(&[Permission::LogsView, Permission::TracingLive]))
            .create("Antispam")
            .icon(view! { <IconShieldCheck/> })
            .create("Train")
            .route("/spam/train")
            .insert(true)
            .create("Test")
            .route("/spam/test")
            .insert(true)
            .insert(permissions.has_access(Permission::SieveRun))
            .create("Settings")
            .icon(view! { <IconAdjustmentsHorizontal/> })
            .raw_route(DEFAULT_SETTINGS_URL)
            .insert(permissions.has_access(Permission::SettingsList))
            .create("Maintenance")
            .icon(view! { <IconWrench/> })
            .route("/maintenance")
            .insert(permissions.has_access_any(&[
                Permission::SettingsReload,
                Permission::Restart,
                Permission::UpdateSpamFilter,
                Permission::UpdateWebadmin,
            ]))
            .menu_items
    }

    pub fn account(permissions: &Permissions) -> Vec<MenuItem> {
        LayoutBuilder::new("/account")
            .create("Encryption-at-rest")
            .icon(view! { <IconLockClosed/> })
            .route("/crypto")
            .insert(permissions.has_access(Permission::ManageEncryption))
            .create("Change Password")
            .icon(view! { <IconKey/> })
            .route("/password")
            .insert(permissions.has_access(Permission::ManagePasswords))
            .create("Two-factor Auth")
            .icon(view! { <IconShieldCheck/> })
            .route("/mfa")
            .insert(permissions.has_access(Permission::ManagePasswords))
            .create("App Passwords")
            .icon(view! { <IconSquare2x2/> })
            .route("/app-passwords")
            .insert(permissions.has_access(Permission::ManagePasswords))
            .menu_items
    }
}

pub fn build_schemas() -> Arc<Schemas> {
    Schemas::builder()
        .build_login()
        .build_principals()
        .build_store()
        .build_directory()
        .build_authentication()
        .build_storage()
        .build_tls()
        .build_server()
        .build_listener()
        .build_telemetry()
        .build_smtp_inbound()
        .build_smtp_outbound()
        .build_mail_auth()
        .build_jmap()
        .build_imap()
        .build_sieve()
        .build_spam_lists()
        .build_spam_manage()
        .build_password_change()
        .build_crypto()
        .build_authorize()
        .build_mfa()
        .build_app_passwords()
        .build_live_tracing()
        .build()
        .into()
}
