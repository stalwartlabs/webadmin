/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    vec,
};

use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::{
        badge::Badge,
        card::{Card, CardItem},
        form::button::Button,
        icon::{IconCancel, IconCheckCircle, IconClock},
        list::{
            header::ColumnList, pagination::Pagination, toolbar::SearchBox, Footer, ListItem,
            ListTable, ListTextItem, Toolbar,
        },
        report::{ReportItem, ReportSection, ReportTextValue, ReportView},
        Color,
    },
    pages::{
        queue::reports::{display::PAGE_SIZE, Policy},
        FormatDateTime,
    },
};

use super::{FailureDetails, PolicyType, TlsReport};

#[derive(Clone, Debug, PartialEq, Copy)]
enum CurrentView {
    Main,
    Policy,
    Failure,
}

#[component]
#[allow(unused_parens)]
pub fn TlsReportDisplay(
    report: TlsReport,
    extra: Vec<(String, String)>,
    back_url: String,
) -> impl IntoView {
    let report_start_date = report.date_range.start_datetime.format_date_time();
    let report_start_time = report.date_range.start_datetime.format_time();
    let report_end_date = report.date_range.end_datetime.format_date_time();
    let report_end_time = report.date_range.end_datetime.format_time();

    let mut total_success = 0;
    let mut total_fail = 0;

    for policy in &report.policies {
        total_success += policy.summary.total_success;
        total_fail += policy.summary.total_failure;
    }

    let current_view = create_rw_signal(CurrentView::Main);
    let total_policies = report.policies.len() as u32;
    let policy_page = create_rw_signal(1u32);
    let policy_filter = create_rw_signal(None::<String>);
    let failure_page = create_rw_signal(1u32);
    let failure_filter = create_rw_signal(None::<String>);
    let selected_policy = create_rw_signal(None::<Policy>);
    let selected_failure = create_rw_signal(None::<FailureDetails>);
    let fetch_policies = create_memo(move |_| {
        let mut policies = Vec::with_capacity(PAGE_SIZE as usize);
        let mut offset = (policy_page.get().saturating_sub(1)) * PAGE_SIZE;
        let policy_filter = policy_filter
            .get()
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();

        for policy in &report.policies {
            if policy.contains_string(&policy_filter) {
                if offset == 0 {
                    policies.push(policy.clone());

                    if policies.len() >= PAGE_SIZE as usize {
                        break;
                    }
                } else {
                    offset -= 1;
                }
            }
        }

        selected_policy.set(None);
        selected_failure.set(None);
        failure_page.set(1);
        failure_filter.set(None);
        policies
    });

    let extra = extra
        .into_iter()
        .filter_map(|(k, v)| {
            if !v.is_empty() {
                Some(view! {
                    <ReportItem label=k>
                        <ReportTextValue value=v/>
                    </ReportItem>
                })
            } else {
                None
            }
        })
        .collect_view();

    view! {
        <Card>
            <CardItem title="Report Start" contents=report_start_date subcontents=report_start_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Report End" contents=report_end_date subcontents=report_end_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Successes" contents=total_success.to_string()>

                <IconCheckCircle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Failures" contents=total_fail.to_string()>

                <IconCancel attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

        </Card>

        <ReportView hide=Signal::derive(move || current_view.get() != CurrentView::Main)>
            <ReportSection title="Report Details">
                <ReportItem label="Report ID">
                    <ReportTextValue value=report.report_id/>
                </ReportItem>
                <ReportItem label="Organization Name" hide=report.organization_name.is_none()>
                    <ReportTextValue value=report.organization_name.unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Contact" hide=report.contact_info.is_none()>
                    <ReportTextValue value=report.contact_info.unwrap_or_default()/>
                </ReportItem>
                {extra}
            </ReportSection>
            {if total_policies > 0 {
                Some(
                    view! {
                        <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                            <div class="sm:col-span-12 pb-4">
                                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                    Policies
                                </h2>
                            </div>
                            <ListTable>
                                <Toolbar slot>
                                    <SearchBox
                                        value=policy_filter
                                        on_search=move |value| {
                                            policy_filter.set(Some(value));
                                        }
                                    />

                                </Toolbar>
                                <ColumnList headers=vec![
                                    "Domain".to_string(),
                                    "Type".to_string(),
                                    "Successes".to_string(),
                                    "Failures".to_string(),
                                    "".to_string(),
                                ]>

                                    <For
                                        each=move || fetch_policies.get()
                                        key=|policy| policy.id()
                                        children=move |policy| {
                                            let domain = policy.policy.policy_domain.clone();
                                            view! {
                                                <tr>
                                                    <ListTextItem>{domain}</ListTextItem>
                                                    <ListTextItem>{policy.policy.policy_type}</ListTextItem>
                                                    <ListTextItem>{policy.summary.total_success}</ListTextItem>
                                                    <ListTextItem>{policy.summary.total_failure}</ListTextItem>
                                                    <ListItem subclass="px-6 py-1.5">
                                                        <button
                                                            class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                                            on:click=move |_| {
                                                                current_view.set(CurrentView::Policy);
                                                                selected_policy.set(Some(policy.clone()));
                                                            }
                                                        >

                                                            Show
                                                        </button>
                                                    </ListItem>

                                                </tr>
                                            }
                                        }
                                    />

                                </ColumnList>

                                <Footer slot>

                                    <Pagination
                                        current_page=policy_page
                                        total_results=Some(total_policies)
                                        page_size=PAGE_SIZE
                                        on_page_change=move |new_page: u32| {
                                            policy_page.set(new_page);
                                        }
                                    />

                                </Footer>

                            </ListTable>

                        </div>
                    },
                )
            } else {
                None
            }}

            <div class="flex justify-end">

                <Button
                    text="Close"
                    color=Color::Blue
                    on_click=move |_| {
                        use_navigate()(&back_url, Default::default());
                    }
                />

            </div>
        </ReportView>

        {move || {
            selected_policy
                .get()
                .map(|policy| {
                    view! {
                        <ReportView hide=Signal::derive(move || {
                            current_view.get() != CurrentView::Policy
                        })>
                            <ReportSection title="Policy">
                                <ReportItem label="Type">{policy.policy.policy_type}</ReportItem>
                                <ReportItem label="Domain">
                                    <ReportTextValue value=policy.policy.policy_domain/>
                                </ReportItem>
                                <ReportItem label="MX Host" hide=policy.policy.mx_host.is_empty()>
                                    <ReportTextValue value=policy.policy.mx_host.join(", ")/>
                                </ReportItem>
                                <ReportItem label="Total successes">
                                    <ReportTextValue value=policy
                                        .summary
                                        .total_success
                                        .to_string()/>
                                </ReportItem>
                                <ReportItem label="Total failures">
                                    <ReportTextValue value=policy
                                        .summary
                                        .total_failure
                                        .to_string()/>
                                </ReportItem>
                            </ReportSection>
                            {if !policy.failure_details.is_empty() {
                                let total_failures = policy.failure_details.len() as u32;
                                let fetch_failures = create_memo(move |_| {
                                    let mut failures = Vec::with_capacity(PAGE_SIZE as usize);
                                    let mut offset = (failure_page.get().saturating_sub(1))
                                        * PAGE_SIZE;
                                    let failure_filter = failure_filter
                                        .get()
                                        .map(|s| s.trim().to_lowercase())
                                        .unwrap_or_default();
                                    for failure in &policy.failure_details {
                                        if failure.contains_string(&failure_filter) {
                                            if offset == 0 {
                                                failures.push(failure.clone());
                                                if failures.len() >= PAGE_SIZE as usize {
                                                    break;
                                                }
                                            } else {
                                                offset -= 1;
                                            }
                                        }
                                    }
                                    selected_failure.set(None);
                                    failures
                                });
                                Some(
                                    view! {
                                        <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                                            <div class="sm:col-span-12 pb-4">
                                                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                                    Failures
                                                </h2>
                                            </div>
                                            <ListTable>
                                                <Toolbar slot>
                                                    <SearchBox
                                                        value=failure_filter
                                                        on_search=move |value| {
                                                            failure_filter.set(Some(value));
                                                        }
                                                    />

                                                </Toolbar>
                                                <ColumnList headers=vec![
                                                    "Result".to_string(),
                                                    "Sender".to_string(),
                                                    "Receiver".to_string(),
                                                    "Failures".to_string(),
                                                    "".to_string(),
                                                ]>

                                                    <For
                                                        each=move || fetch_failures.get()
                                                        key=|failure| failure.id()
                                                        children=move |failure| {
                                                            let failure_ = failure.clone();
                                                            view! {
                                                                <tr>
                                                                    <ListTextItem>
                                                                        {failure.result_type.to_string()}
                                                                    </ListTextItem>
                                                                    <ListTextItem>
                                                                        {failure
                                                                            .sending_mta_ip
                                                                            .map(|ip| ip.to_string())
                                                                            .unwrap_or_default()}
                                                                    </ListTextItem>
                                                                    <ListTextItem>
                                                                        {failure
                                                                            .receiving_ip
                                                                            .map(|ip| ip.to_string())
                                                                            .or(failure.receiving_mx_hostname)
                                                                            .or(failure.receiving_mx_helo)}
                                                                    </ListTextItem>
                                                                    <ListTextItem>{failure.failed_session_count}</ListTextItem>
                                                                    <ListItem subclass="px-6 py-1.5">
                                                                        <button
                                                                            class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                                                            on:click=move |_| {
                                                                                current_view.set(CurrentView::Failure);
                                                                                selected_failure.set(Some(failure_.clone()));
                                                                            }
                                                                        >

                                                                            Show
                                                                        </button>
                                                                    </ListItem>

                                                                </tr>
                                                            }
                                                        }
                                                    />

                                                </ColumnList>

                                                <Footer slot>

                                                    <Pagination
                                                        current_page=failure_page
                                                        total_results=Some(total_failures)
                                                        page_size=PAGE_SIZE
                                                        on_page_change=move |new_page: u32| {
                                                            failure_page.set(new_page);
                                                        }
                                                    />

                                                </Footer>

                                            </ListTable>

                                        </div>
                                    },
                                )
                            } else {
                                None
                            }}

                            <div class="mt-5 flex justify-end gap-x-2">
                                <Button
                                    text="Close"
                                    color=Color::Blue
                                    on_click=move |_| {
                                        current_view.set(CurrentView::Main);
                                    }
                                />

                            </div>

                        </ReportView>
                    }
                })
        }}

        {move || {
            selected_failure
                .get()
                .map(|failure| {
                    view! {
                        <ReportView hide=Signal::derive(move || {
                            current_view.get() != CurrentView::Failure
                        })>
                            <ReportSection title="Failure Details">
                                <ReportItem label="Type">
                                    <ReportTextValue value=failure.result_type.to_string()/>
                                </ReportItem>
                                <ReportItem
                                    label="Sending MTA IP"
                                    hide=failure.sending_mta_ip.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .sending_mta_ip
                                        .map(|ip| ip.to_string())
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem
                                    label="Receiving IP"
                                    hide=failure.receiving_ip.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .receiving_ip
                                        .map(|ip| ip.to_string())
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem
                                    label="Receiving MX Host"
                                    hide=failure.receiving_mx_hostname.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .receiving_mx_hostname
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem
                                    label="Receiving MX HELO"
                                    hide=failure.receiving_mx_helo.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .receiving_mx_helo
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem label="Failed Sessions">
                                    <ReportTextValue value=failure
                                        .failed_session_count
                                        .to_string()/>
                                </ReportItem>
                                <ReportItem
                                    label="Failure Code"
                                    hide=failure.failure_reason_code.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .failure_reason_code
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem
                                    label="Additional Info"
                                    hide=failure.additional_information.is_none()
                                >
                                    <ReportTextValue value=failure
                                        .additional_information
                                        .unwrap_or_default()/>
                                </ReportItem>
                            </ReportSection>
                            <div class="mt-5 flex justify-end gap-x-2">
                                <Button
                                    text="Close"
                                    color=Color::Blue
                                    on_click=move |_| {
                                        current_view.set(CurrentView::Policy);
                                    }
                                />

                            </div>

                        </ReportView>
                    }
                })
        }}
    }
}

impl Policy {
    pub fn contains_string(&self, filter: &str) -> bool {
        filter.is_empty()
            || self.policy.policy_domain.contains(filter)
            || self
                .policy
                .policy_type
                .to_string()
                .to_lowercase()
                .contains(filter)
            || self
                .policy
                .policy_string
                .iter()
                .any(|s| s.to_lowercase().contains(filter))
            || self
                .policy
                .mx_host
                .iter()
                .any(|s| s.to_lowercase().contains(filter))
            || self
                .failure_details
                .iter()
                .any(|f| f.contains_string(filter))
    }

    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl FailureDetails {
    pub fn contains_string(&self, filter: &str) -> bool {
        filter.is_empty()
            || self
                .sending_mta_ip
                .map_or(false, |s| s.to_string().contains(filter))
            || self
                .receiving_ip
                .map_or(false, |s| s.to_string().contains(filter))
            || self
                .receiving_mx_hostname
                .as_ref()
                .map_or(false, |s| s.contains(filter))
            || self
                .receiving_mx_helo
                .as_ref()
                .map_or(false, |s| s.contains(filter))
            || self
                .additional_information
                .as_ref()
                .map_or(false, |s| s.contains(filter))
            || self
                .failure_reason_code
                .as_ref()
                .map_or(false, |s| s.contains(filter))
    }

    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl IntoView for PolicyType {
    fn into_view(self) -> View {
        let color = match self {
            PolicyType::Tlsa => Color::Green,
            PolicyType::Sts => Color::Blue,
            PolicyType::NoPolicyFound | PolicyType::Other => Color::Red,
        };
        view! {
            <Badge color=color>

                {self.to_string()}
            </Badge>
        }
    }
}
