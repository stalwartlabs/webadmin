/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::sync::Arc;

use leptos::*;
use leptos_router::use_query_map;

use crate::{
    components::{list::ZeroResults, report::ReportView},
    pages::config::Schemas,
};

use super::{Field, Form, Section};

#[component]
pub fn SettingsSearch() -> impl IntoView {
    let query = use_query_map();
    let schemas = expect_context::<Arc<Schemas>>();

    let results = create_memo(move |_| {
        let params = query.with(|q| {
            q.get("query")
                .map(|s| {
                    s.split_whitespace()
                        .map(|s| s.to_lowercase())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        });

        schemas
            .schemas
            .values()
            .filter_map(|s| {
                let (title, matches) = s.form.contains_string(&params)?;

                Some((s.id, title, matches))
            })
            .collect::<Vec<_>>()
    });

    view! {
        {move || {
            let results = results.get();
            let has_results = !results.is_empty();
            let results = results
                .into_iter()
                .map(|(id, title, matches)| {
                    let url = format!("/settings/{id}/edit");
                    view! {
                        <a
                            class="group flex flex-col bg-white border shadow-sm rounded-xl hover:shadow-md transition dark:bg-slate-900 dark:border-gray-800"
                            href=url
                        >
                            <div class="p-4 md:p-5">
                                <div class="flex justify-between items-center">
                                    <div>
                                        <h3 class="group-hover:text-blue-600 font-semibold text-gray-800 dark:group-hover:text-gray-400 dark:text-gray-200">
                                            {title}
                                        </h3>
                                        <p class="text-sm text-gray-500">{matches}</p>
                                    </div>
                                    <div class="ps-3">
                                        <svg
                                            class="flex-shrink-0 size-5"
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <path d="m9 18 6-6-6-6"></path>
                                        </svg>
                                    </div>
                                </div>
                            </div>
                        </a>
                    }
                        .into_view()
                })
                .collect_view();
            if has_results {
                view! {
                    <div class="max-w-[85rem] px-4 py-10 sm:px-6 lg:px-8 lg:py-14 mx-auto">
                        <div class="grid sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-3 sm:gap-6">
                            {results}
                        </div>

                    </div>
                }
                    .into_view()
            } else {
                view! {
                    <ReportView>
                        <ZeroResults
                            title="No results"
                            subtitle="No search settings were found with the selected criteria."
                        />
                    </ReportView>
                }
                    .into_view()
            }
        }}
    }
}

trait ContainsString {
    fn contains_string(&self, query: &[String]) -> Option<&'static str>;
}

impl Form {
    fn contains_string(&self, query: &[String]) -> Option<(&'static str, &'static str)> {
        self.title
            .contains_string(query)
            .or_else(|| self.subtitle.contains_string(query))
            .map(|matches| (self.title, matches))
            .or_else(|| {
                self.sections.iter().find_map(|s| {
                    s.contains_string(query)
                        .map(|m| (s.title.as_ref().copied().unwrap_or(self.title), m))
                })
            })
    }
}

impl ContainsString for Section {
    fn contains_string(&self, query: &[String]) -> Option<&'static str> {
        self.title
            .as_ref()
            .and_then(|t| t.contains_string(query))
            .or_else(|| self.fields.iter().find_map(|f| f.contains_string(query)))
    }
}

impl ContainsString for Field {
    fn contains_string(&self, query: &[String]) -> Option<&'static str> {
        self.label_form
            .contains_string(query)
            .or_else(|| self.help.as_ref().and_then(|h| h.contains_string(query)))
            .or_else(|| self.id.contains_string(query))
    }
}

impl ContainsString for &'static str {
    fn contains_string(&self, query: &[String]) -> Option<&'static str> {
        let s = self.to_ascii_lowercase();
        if query.iter().all(|q| s.contains(q)) {
            Some(self)
        } else {
            None
        }
    }
}
