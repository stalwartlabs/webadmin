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

use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        card::{Card, CardItem},
        form::button::Button,
        icon::{IconEnvelope, IconShieldCheck, IconUserGroup},
        list::table::{Table, TableRow},
        messages::alert::{use_alerts, Alert, Alerts},
        report::ReportView,
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::List,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnsRecord {
    #[serde(rename = "type")]
    typ: String,
    name: String,
    content: String,
}

#[component]
pub fn DomainDisplay() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let params = use_params_map();
    let domain_details = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();

            async move {
                let result = HttpRequest::get(("/api/domain", &name))
                    .with_authorization(&auth)
                    .send::<Vec<DnsRecord>>()
                    .await?;
                let user_count = HttpRequest::get("/api/principal")
                    .with_authorization(&auth)
                    .with_parameter("filter", &name)
                    .send::<List<String>>()
                    .await
                    .map(|r| r.total)
                    .unwrap_or_default();

                Ok((result, user_count))
            }
        },
    );

    view! {
        <Alerts/>
        <Transition fallback=Skeleton>

            {move || match domain_details.get() {
                None => None,
                Some(Err(http::Error::Unauthorized)) => {
                    use_navigate()("/login", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(http::Error::NotFound)) => {
                    use_navigate()("/manage/directory/domains", Default::default());
                    Some(view! { <div></div> }.into_view())
                }
                Some(Err(err)) => {
                    alert.set(Alert::from(err));
                    Some(view! { <div></div> }.into_view())
                }
                Some(Ok((records, user_count))) => {
                    let signature_count = records
                        .iter()
                        .filter(|r| r.typ == "TXT" && r.content.contains("DKIM"))
                        .count()
                        .to_string();
                    Some(
                        view! {
                            <Card>
                                <CardItem
                                    title="Domain"
                                    contents=Signal::derive(move || {
                                        params.get().get("id").cloned().unwrap_or_default()
                                    })
                                >

                                    <IconEnvelope attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>
                                </CardItem>
                                <CardItem title="Accounts" contents=user_count.to_string()>
                                    <IconUserGroup attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>
                                </CardItem>
                                <CardItem title="DKIM Signatures" contents=signature_count>
                                    <IconShieldCheck attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>
                                </CardItem>
                            </Card>

                            <ReportView>

                                <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                                    <div class="sm:col-span-12 pb-4">
                                        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                            DNS Records
                                        </h2>
                                    </div>
                                    <Table headers=vec![
                                        "Type".to_string(),
                                        "Name".to_string(),
                                        "Contents".to_string(),
                                    ]>
                                        {records
                                            .into_iter()
                                            .map(|record| {
                                                view! {
                                                    <TableRow>
                                                        <span>{record.typ}</span>
                                                        <span>{record.name}</span>
                                                        <span>{record.content}</span>

                                                    </TableRow>
                                                }
                                            })
                                            .collect_view()}

                                    </Table>

                                </div>

                                <div class="flex justify-end">

                                    <Button
                                        text="Close"
                                        color=Color::Blue
                                        on_click=move |_| {
                                            use_navigate()(
                                                "/manage/directory/domains",
                                                Default::default(),
                                            );
                                        }
                                    />

                                </div>
                            </ReportView>
                        }
                            .into_view(),
                    )
                }
            }}

        </Transition>
    }
}
