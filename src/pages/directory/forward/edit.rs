/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::collections::HashSet;

use leptos::*;
use leptos_router::*;

use serde::Deserialize;

use crate::{
    components::{
        form::{button::Button, Form, FormButtonBar, FormItem, FormSection},
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::config::{Settings, UpdateSettings},
};

#[derive(Deserialize, Default)]
struct SettingsList {
    pub items: Settings,
}

use super::{
    forward_prefix,
    list::{parse_forward_settings, rebuild_sieve_for_domains},
    ForwardRule,
};

#[component]
pub fn ForwardEdit() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();

    let is_create = create_memo(move |_| {
        params.get().get("id").map(|id| id == "_new_").unwrap_or(true)
    });

    let from_param = create_memo(move |_| {
        if is_create.get() {
            String::new()
        } else {
            params
                .get()
                .get("id")
                .map(|id| id.replace("%40", "@").replace("%2B", "+"))
                .unwrap_or_default()
        }
    });

    let from = create_rw_signal(String::new());
    let to_input = create_rw_signal(String::new()); // comma/newline separated in textarea
    let keep_copy = create_rw_signal(true);

    // Load existing rule when editing
    let _load = create_resource(
        move || from_param.get(),
        move |addr| {
            let auth = auth.get();
            async move {
                if addr.is_empty() {
                    return Ok(ForwardRule::default());
                }
                let raw = HttpRequest::get("/api/settings/list")
                    .with_parameter("prefix", format!("mail.forward.{addr}"))
                    .with_authorization(&auth)
                    .send::<SettingsList>()
                    .await?;

                let mut rule = ForwardRule {
                    from: addr.clone(),
                    keep_copy: true,
                    to: Vec::new(),
                };
                for (key, value) in &raw.items {
                    if key.starts_with("to.") {
                        rule.to.push(value.clone());
                    } else if key == "keep-copy" {
                        rule.keep_copy = value == "true";
                    }
                }
                rule.to.sort();
                Ok::<ForwardRule, http::Error>(rule)
            }
        },
    );

    create_effect(move |_| {
        if let Some(Ok(rule)) = _load.get() {
            from.set(rule.from.clone());
            to_input.set(rule.to.join("\n"));
            keep_copy.set(rule.keep_copy);
        }
    });

    let save_action = create_action(move |_: &()| {
        let auth = auth.get();
        let from_val = from.get().trim().to_lowercase();
        let to_val: Vec<String> = to_input
            .get()
            .split([',', '\n', ';'])
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.contains('@'))
            .collect();
        let keep = keep_copy.get();
        let old_from = from_param.get();
        let creating = is_create.get();

        async move {
            if from_val.is_empty() || !from_val.contains('@') {
                alert.set(Alert::error("From address must be a valid email address."));
                return Ok(());
            }
            if to_val.is_empty() {
                alert.set(Alert::error(
                    "At least one valid destination address is required.",
                ));
                return Ok(());
            }

            let mut updates: Vec<UpdateSettings> = Vec::new();
            let mut affected: HashSet<String> = HashSet::new();

            if !creating && old_from != from_val {
                let old_domain = old_from.splitn(2, '@').nth(1).unwrap_or("").to_string();
                affected.insert(old_domain);
                updates.push(UpdateSettings::Clear {
                    prefix: format!("{}.", forward_prefix(&old_from)),
                    filter: None,
                });
            }

            // Clear existing entries for this address
            updates.push(UpdateSettings::Clear {
                prefix: format!("{}.", forward_prefix(&from_val)),
                filter: None,
            });

            // Insert new values
            let mut values: Vec<(String, String)> = to_val
                .iter()
                .enumerate()
                .map(|(i, addr)| (format!("to.{i}"), addr.clone()))
                .collect();
            values.push(("keep-copy".to_string(), keep.to_string()));

            updates.push(UpdateSettings::Insert {
                prefix: Some(forward_prefix(&from_val)),
                values,
                assert_empty: false,
            });

            HttpRequest::post("/api/settings")
                .with_authorization(&auth)
                .with_body(updates)?
                .send::<Option<String>>()
                .await?;

            // Rebuild Sieve scripts for all affected domains
            let new_domain = from_val.splitn(2, '@').nth(1).unwrap_or("").to_string();
            affected.insert(new_domain);

            let all_raw = HttpRequest::get("/api/settings/list")
                .with_parameter("prefix", "mail.forward")
                .with_authorization(&auth)
                .send::<SettingsList>()
                .await?;
            let all_rules = parse_forward_settings(&all_raw.items);

            rebuild_sieve_for_domains(&auth, &affected, &all_rules).await?;

            Ok::<(), http::Error>(())
        }
    });

    create_effect(move |_| {
        if let Some(result) = save_action.value().get() {
            match result {
                Ok(_) => {
                    alert.set(Alert::success("Forward saved. Sieve script updated."));
                    use_navigate()("/manage/directory/forwards", Default::default());
                }
                Err(e) => alert.set(Alert::from(e)),
            }
        }
    });

    view! {
        <Form
            title=Signal::derive(move || {
                if is_create.get() { "Create Email Forward".to_string() } else { "Edit Email Forward".to_string() }
            })
            subtitle="Configure an email forwarding rule. Changes are immediately applied to the Sieve script."
        >
            <FormSection>
                <FormItem label="From Address" tooltip="The email address whose incoming messages will be forwarded">
                    <input
                        type="email"
                        placeholder="alice@example.com"
                        prop:value=move || from.get()
                        prop:disabled=move || !is_create.get()
                        on:input=move |ev| from.set(event_target_value(&ev))
                        class="py-2 px-3 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    />
                </FormItem>

                <FormItem
                    label="Forward To"
                    tooltip="One or more destination addresses, one per line (or comma-separated)"
                >
                    <textarea
                        placeholder="dest1@example.com\ndest2@example.com"
                        prop:value=move || to_input.get()
                        on:input=move |ev| to_input.set(event_target_value(&ev))
                        rows="4"
                        class="py-2 px-3 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                    />
                </FormItem>

                <FormItem
                    label="Keep Local Copy"
                    tooltip="When enabled, a copy of the message is also delivered to the local mailbox in addition to being forwarded"
                >
                    <div class="flex items-center gap-x-3">
                        <input
                            type="checkbox"
                            prop:checked=move || keep_copy.get()
                            on:change=move |_| keep_copy.update(|v| *v = !*v)
                            class="shrink-0 w-4 h-4 border-gray-300 rounded text-blue-600 focus:ring-blue-500 dark:bg-slate-800 dark:border-gray-600"
                        />
                        <span class="text-sm text-gray-600 dark:text-gray-400">
                            "Deliver a copy to the local mailbox"
                        </span>
                    </div>
                </FormItem>
            </FormSection>

            <FormButtonBar>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=Callback::new(move |_| {
                        use_navigate()("/manage/directory/forwards", Default::default());
                    })
                />
                <Button
                    text=Signal::derive(move || {
                        if save_action.pending().get() {
                            "Saving…".to_string()
                        } else {
                            "Save Forward".to_string()
                        }
                    })
                    color=Color::Blue
                    on_click=Callback::new(move |_| save_action.dispatch(()))
                    disabled=save_action.pending()
                />
            </FormButtonBar>
        </Form>
    }
}
