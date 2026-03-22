/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod edit;
pub mod list;

use leptos::*;
use serde::{Deserialize, Serialize};

use crate::components::{
    form::{FormItem, FormSection},
    icon::{IconPlus, IconXMark},
};

/// A single email forwarding rule.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardRule {
    /// Source address (e.g. `alice@example.com`).
    pub from: String,
    /// One or more destination addresses.
    pub to: Vec<String>,
    /// When true a copy is also delivered to the local mailbox.
    pub keep_copy: bool,
}

impl ForwardRule {
    /// Domain part of the source address (e.g. `example.com`).
    pub fn domain(&self) -> &str {
        self.from.splitn(2, '@').nth(1).unwrap_or("")
    }
}

/// Canonical settings key prefix for a forward rule.
pub fn forward_prefix(from: &str) -> String {
    format!("mail.forward.{from}")
}

/// Generate a per-domain Sieve script from a list of rules.
///
/// Rules belonging to other domains are silently ignored, so callers
/// can pass the full list and this function handles the filtering.
pub fn build_sieve_script(domain: &str, rules: &[ForwardRule]) -> String {
    let domain_rules: Vec<&ForwardRule> = rules
        .iter()
        .filter(|r| r.domain() == domain && !r.to.is_empty())
        .collect();

    if domain_rules.is_empty() {
        return String::new();
    }

    let mut s = String::from("require [\"redirect\", \"envelope\", \"copy\"];\n\n");

    for rule in domain_rules {
        s.push_str(&format!(
            "if envelope :is \"to\" \"{}\" {{\n",
            rule.from
        ));
        for dest in &rule.to {
            if rule.keep_copy {
                s.push_str(&format!("  redirect :copy \"{dest}\";\n"));
            } else {
                s.push_str(&format!("  redirect \"{dest}\";\n"));
            }
        }
        s.push_str("}\n\n");
    }

    s
}

/// Sieve script settings key for a domain (underscores replace dots).
pub fn domain_script_id(domain: &str) -> String {
    domain.replace('.', "_")
}

/// Inline forwarding section for the account edit page.
/// Props are owned by `PrincipalEdit`; saving is done via the main Save button.
#[component]
pub fn AccountForwardSection(
    forward_to: RwSignal<Vec<String>>,
    forward_keep_copy: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <FormSection>
            <FormItem
                label="Forward To"
                tooltip="Forward incoming mail to these addresses. Leave empty to disable forwarding."
            >
                <div class="space-y-3">
                    <For
                        each=move || {
                            forward_to
                                .get()
                                .into_iter()
                                .enumerate()
                                .collect::<Vec<_>>()
                        }
                        key=move |(idx, item)| {
                            format!(
                                "{idx}_{}",
                                item.as_bytes().iter().map(|v| *v as usize).sum::<usize>(),
                            )
                        }
                        children=move |(idx, addr)| {
                            view! {
                                <div class="relative">
                                    <input
                                        type="text"
                                        class="py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm text-sm rounded-lg focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        prop:value=addr
                                        placeholder="dest@example.com"
                                        on:change=move |ev| {
                                            forward_to
                                                .update(|v| {
                                                    if let Some(item) = v.get_mut(idx) {
                                                        *item = event_target_value(&ev);
                                                    }
                                                });
                                        }
                                    />
                                    <button
                                        type="button"
                                        class="absolute top-0 end-0 p-2.5 rounded-e-md dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                        on:click=move |_| {
                                            forward_to.update(|v| { v.remove(idx); });
                                        }
                                    >
                                        <IconXMark/>
                                    </button>
                                </div>
                            }
                        }
                    />
                </div>
                <p class="mt-3 text-end">
                    <button
                        type="button"
                        class="py-1.5 px-2 inline-flex items-center gap-x-1 text-xs font-medium rounded-full border border-dashed border-gray-200 bg-white text-gray-800 hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-gray-800 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        on:click=move |_| {
                            let items = forward_to.get();
                            if items.last().map(|s: &String| !s.is_empty()).unwrap_or(true) {
                                forward_to.update(|v| v.push(String::new()));
                            }
                        }
                    >
                        <IconPlus attr:class="flex-shrink-0 size-3.5"/>
                        "Add address"
                    </button>
                </p>
            </FormItem>

            <FormItem
                label="Keep Local Copy"
                tooltip="When enabled, a copy is also delivered to the local mailbox in addition to being forwarded"
            >
                <div class="flex items-center gap-x-3">
                    <input
                        type="checkbox"
                        prop:checked=move || forward_keep_copy.get()
                        on:change=move |_| forward_keep_copy.update(|v| *v = !*v)
                        class="shrink-0 w-4 h-4 border-gray-300 rounded text-blue-600 focus:ring-blue-500 dark:bg-slate-800 dark:border-gray-600"
                    />
                    <span class="text-sm text-gray-600 dark:text-gray-400">
                        "Deliver a copy to the local mailbox"
                    </span>
                </div>
            </FormItem>
        </FormSection>
    }
}
