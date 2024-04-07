use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::{
        icon::{IconCheckCircle, IconComputerDesktop, IconPower, IconRefresh, IconShieldCheck},
        messages::alert::{use_alerts, Alert, Alerts},
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::config::ReloadSettings,
};

#[derive(Debug, Clone, Copy)]
struct Action {
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    url: &'static str,
    success_message: &'static str,
}

const ACTIONS: &[Action] = &[
    Action {
        title: "Reload configuration",
        description: "Performs a hot reload of the server settings. Changes to listeners or stores require a server restart.",
        icon: "refresh",
        url: "/api/reload",
        success_message: "Successfully reloaded configuration",
    },
    Action {
        title: "Validate configuration",
        description: "Checks the configuration for errors and warnings.",
        icon: "check_circle",
        url: "/api/reload?dry-run=true",
        success_message: "Configuration is valid",
    },
    Action {
        title: "Restart server",
        description: "Restarts the server. This will interrupt any active connections.",
        icon: "power",
        url: "/api/restart",
        success_message: "Restarting server, try reloading this page in a few seconds.",
    },
    Action {
        title: "Update SPAM rules",
        description: "Downloads and installs the latest SPAM rules from the Github repository.",
        icon: "shield_check",
        url: "/api/update/spam-filter",
        success_message: "Successfully updated SPAM rules to the latest version",
    },
    Action {
        title: "Update Webadmin",
        description: "Downloads and installs the latest version of the Stalwart Webadmin from the Github repository.",
        icon: "computer_desktop",
        url: "/api/update/webadmin",
        success_message: "Successfully updated the web admin to the latest version",
    },

];

#[component]
pub fn Maintenance() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let (pending, set_pending) = create_signal(false);

    let execute = create_action(move |idx: &usize| {
        let auth = auth.get();
        let action = ACTIONS[*idx];

        async move {
            set_pending.set(true);

            let err = if action.url.starts_with("/api/reload") {
                match HttpRequest::get(action.url)
                    .with_authorization(&auth)
                    .send::<ReloadSettings>()
                    .await
                {
                    Ok(result) => {
                        set_pending.set(false);
                        if result.errors.is_empty() && result.warnings.is_empty() {
                            alert.set(Alert::success(action.success_message).without_timeout());
                        } else {
                            alert.set(Alert::from(result));
                        }
                        return;
                    }
                    Err(err) => err,
                }
            } else {
                match HttpRequest::get(action.url)
                    .with_authorization(&auth)
                    .send::<Option<String>>()
                    .await
                {
                    Ok(_) => {
                        set_pending.set(false);
                        alert.set(Alert::success(action.success_message).without_timeout());
                        return;
                    }
                    Err(err) => err,
                }
            };

            match err {
                http::Error::Unauthorized => {
                    use_navigate()("/login", Default::default());
                }
                err => {
                    set_pending.set(false);
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    let actions = ACTIONS.iter().enumerate().map(|(idx, action)| {
        let icon_class = "mt-1 flex-shrink-0 size-5 text-gray-800 dark:text-gray-200";
        let icon = match action.icon {
            "refresh" => view! { <IconRefresh attr:class=icon_class/> },
            "check_circle" => view! { <IconCheckCircle attr:class=icon_class/> },
            "power" => view! { <IconPower attr:class=icon_class/> },
            "shield_check" => view! { <IconShieldCheck attr:class=icon_class/> },
            "computer_desktop" => view! { <IconComputerDesktop attr:class=icon_class/> },
            _ => unreachable!("No icon specified"),
        };

        view! {
            <a
                class="group flex flex-col bg-white border shadow-sm rounded-xl hover:shadow-md transition dark:bg-slate-900 dark:border-gray-800"
                href="#"
                on:click=move |_| {
                    execute.dispatch(idx);
                }

                disabled=move || pending.get()
            >

                <div class="p-4 md:p-5">
                    <div class="flex">
                        {icon} <div class="grow ms-5">
                            <h3 class="group-hover:text-blue-600 font-semibold text-gray-800 dark:group-hover:text-gray-400 dark:text-gray-200">
                                {action.title}
                            </h3>
                            <p class="text-sm text-gray-500">{action.description}</p>
                        </div>
                    </div>
                </div>
            </a>
        }

    }).collect_view();

    view! {
        <div class="max-w-5xl px-4 py-10 sm:px-6 lg:px-8 lg:py-14 mx-auto">
            <Alerts/>
            <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-6">

                {actions}

            </div>

        </div>
    }
}
