/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};
use std::ops::Add;

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
    pages::{directory::Principal, List},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnsRecord {
    #[serde(rename = "type")]
    typ: String,
    name: String,
    content: String,
}

fn format_zonefile(records: &Vec<DnsRecord>, domain: &str) -> String {
    let max_len = records.iter().fold([0, 0], |[name, typ], x| {
        [name.max(x.name.len()), typ.max(x.typ.len())]
    });

    records.iter().fold(String::new(), |acc, x| {
        let key = format!(
            "{}{: <width1$} IN {: <width2$}",
            acc,
            x.name,
            x.typ,
            width1 = max_len[0],
            width2 = max_len[1]
        );
        if x.name == "TXT" {
            x.content.as_bytes()
                .chunks(255)
                .fold(key, |acc, x| {
                    format!("{} \"{}\"", acc, String::from_utf8_lossy(x))
                })
                .add("\n")
        } else {
            format!("{} {}\n", key, x.content)
        }
    })
}

#[component]
pub fn DnsDisplay() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let params = use_params_map();
    let domain_details = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get_untracked();

            async move {
                let result = HttpRequest::get(("/api/dns/records", &name))
                    .with_authorization(&auth)
                    .send::<Vec<DnsRecord>>()
                    .await?;
                let user_count = HttpRequest::get("/api/principal")
                    .with_authorization(&auth)
                    .with_parameter("filter", &name)
                    .with_parameter("fields", "name")
                    .with_parameter("count", "1")
                    .with_parameter("types", "individual")
                    .send::<List<Principal>>()
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
                    use_navigate()("/manage/directory/domain", Default::default());
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
                    let zonefile = format_zonefile(
                        &records,
                        &format!(
                            "{}.",
                            params.get().get("id").map(|d| d.as_str()).unwrap_or_default(),
                        ),
                    );
                    log::debug!("zomefile: {}", zonefile);
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
                                    <div class="sm:col-span-12 pb-4 pt-10">
                                        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                            Zonefile
                                        </h2>
                                    </div>
                                    <textarea
                                        class="py-3 px-4 block w-full border-gray-200 rounded-lg text-sm font-mono text-nowrap focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                                        readonly=true
                                        rows=20
                                        prop:value=zonefile
                                    ></textarea>

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
