use std::{
    hash::{DefaultHasher, Hash, Hasher},
    vec,
};

use chrono::{DateTime, Utc};
use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::{
        badge::Badge,
        card::{Card, CardItem},
        form::button::Button,
        icon::{
            IconAlertTriangle, IconArrowRightCircle, IconCancel, IconCheckCircle, IconClock,
            IconEnvelope, IconId,
        },
        list::{
            header::ColumnList,
            pagination::{ItemPagination, Pagination},
            table::{Table, TableRow},
            toolbar::SearchBox,
            Footer, ListItem, ListTable, ListTextItem, Toolbar,
        },
        report::{ReportItem, ReportSection, ReportTextValue, ReportView},
        Color,
    },
    pages::{
        queue::reports::{display::PAGE_SIZE, ActionDisposition, Report},
        FormatDateTime,
    },
};

use super::{DkimResult, DmarcResult, Record, SpfResult};

#[component]
#[allow(unused_parens)]
pub fn DmarcReportDisplay(
    report: Report,
    extra: Vec<(String, String)>,
    back_url: String,
) -> impl IntoView {
    let report_start_date =
        DateTime::<Utc>::from_timestamp(report.report_metadata.date_range.begin as i64, 0)
            .map(|dt| dt.format_date_time())
            .unwrap_or_else(|| "N/A".to_string());
    let report_start_time =
        DateTime::<Utc>::from_timestamp(report.report_metadata.date_range.begin as i64, 0)
            .map(|dt| dt.format_time())
            .unwrap_or_else(|| "N/A".to_string());
    let report_end_date =
        DateTime::<Utc>::from_timestamp(report.report_metadata.date_range.end as i64, 0)
            .map(|dt| dt.format_date_time())
            .unwrap_or_else(|| "N/A".to_string());
    let report_end_time =
        DateTime::<Utc>::from_timestamp(report.report_metadata.date_range.end as i64, 0)
            .map(|dt| dt.format_time())
            .unwrap_or_else(|| "N/A".to_string());
    let domain = report.policy_published.domain.clone();

    let mut total_pass = 0;
    let mut total_quarantine = 0;
    let mut total_reject = 0;
    let mut total_none = 0;

    for record in &report.record {
        match &record.row.policy_evaluated.disposition {
            ActionDisposition::Pass => total_pass += 1,
            ActionDisposition::Quarantine => total_quarantine += 1,
            ActionDisposition::Reject => total_reject += 1,
            ActionDisposition::Unspecified | ActionDisposition::None => total_none += 1,
        }
    }
    let from = if !report.report_metadata.org_name.is_empty() {
        report.report_metadata.org_name.clone()
    } else {
        "N/A".to_string()
    };
    let email = report.report_metadata.email.clone();
    let total_results = report.record.len() as u32;
    let page = create_rw_signal(1u32);
    let filter = create_rw_signal(None::<String>);
    let selected_record = create_rw_signal(0u32);
    let fetch_records = create_memo(move |_| {
        let mut records = Vec::with_capacity(PAGE_SIZE as usize);
        let mut offset = (page.get().saturating_sub(1)) * PAGE_SIZE;
        let filter = filter
            .get()
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();

        let mut record_id: u32 = 0;
        for record in &report.record {
            if record.contains_string(&filter) {
                if offset == 0 {
                    records.push((record_id, record.clone()));
                    record_id += 1;

                    if records.len() >= PAGE_SIZE as usize {
                        break;
                    }
                } else {
                    offset -= 1;
                }
            }
        }

        selected_record.set(0u32);
        records
    });
    let display_record = create_rw_signal(false);
    let fetch_selected_record = create_memo(move |_| {
        if display_record.get() {
            fetch_records
                .get()
                .into_iter()
                .nth(selected_record.get() as usize)
        } else {
            None
        }
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
            <CardItem title="Domain" contents=domain>

                <IconEnvelope attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Report Start" contents=report_start_date subcontents=report_start_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Report End" contents=report_end_date subcontents=report_end_time>

                <IconClock attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Sender" contents=from subcontents=email>

                <IconId attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

        </Card>
        <Card>
            <CardItem title="Passed" contents=total_pass.to_string()>

                <IconCheckCircle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Rejected" contents=total_reject.to_string()>

                <IconCancel attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="Quarantines" contents=total_quarantine.to_string()>

                <IconAlertTriangle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>
            <CardItem title="No Action" contents=total_none.to_string()>

                <IconArrowRightCircle attr:class="flex-shrink-0 size-5 text-gray-400 dark:text-gray-600"/>

            </CardItem>

        </Card>

        <ReportView hide=display_record>
            <ReportSection title="Report Details">
                <ReportItem label="Report ID">
                    <ReportTextValue value=report.report_metadata.report_id/>
                </ReportItem>
                <ReportItem
                    label="Organization Name"
                    hide=report.report_metadata.org_name.is_empty()
                >
                    <ReportTextValue value=report.report_metadata.org_name/>
                </ReportItem>
                <ReportItem label="E-mail" hide=report.report_metadata.email.is_empty()>
                    <ReportTextValue value=report.report_metadata.email/>
                </ReportItem>
                <ReportItem
                    label="Extra Contact Info"
                    hide=report.report_metadata.extra_contact_info.is_none()
                >
                    <ReportTextValue value=report
                        .report_metadata
                        .extra_contact_info
                        .unwrap_or_default()/>
                </ReportItem>
                <ReportItem label="Errors" hide=report.report_metadata.error.is_empty()>
                    <ReportTextValue value=report.report_metadata.error.join(",")/>
                </ReportItem>
                {extra}
            </ReportSection>
            <ReportSection title="Published Policy">
                <ReportItem label="Domain">
                    <ReportTextValue value=report.policy_published.domain/>
                </ReportItem>

                <ReportItem label="DKIM Alignment">
                    <ReportTextValue value=report.policy_published.adkim.to_string()/>
                </ReportItem>
                <ReportItem label="SPF Alignment">
                    <ReportTextValue value=report.policy_published.aspf.to_string()/>
                </ReportItem>
                <ReportItem label="Domain Policy">
                    <ReportTextValue value=report.policy_published.p.to_string()/>
                </ReportItem>
                <ReportItem label="Subdomain Policy">
                    <ReportTextValue value=report.policy_published.sp.to_string()/>
                </ReportItem>
                <ReportItem label="Testing" hide=!report.policy_published.testing>
                    <ReportTextValue value="Yes"/>
                </ReportItem>
            </ReportSection>
            <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                <div class="sm:col-span-12 pb-4">
                    <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">Records</h2>
                </div>
                <ListTable>
                    <Toolbar slot>
                        <SearchBox
                            value=filter
                            on_search=move |value| {
                                filter.set(Some(value));
                            }
                        />

                    </Toolbar>
                    <ColumnList headers=vec![
                        "From".to_string(),
                        "To".to_string(),
                        "Source IP".to_string(),
                        "Disposition".to_string(),
                        "".to_string(),
                    ]>

                        <For
                            each=move || fetch_records.get()
                            key=|(_, record)| record.id()
                            children=move |(record_id, record)| {
                                view! {
                                    <tr>
                                        <ListTextItem>
                                            {record.identifiers.header_from.clone()}
                                        </ListTextItem>
                                        <ListTextItem>
                                            {record.identifiers.envelope_to.clone().unwrap_or_default()}
                                        </ListTextItem>
                                        <ListTextItem>
                                            {record
                                                .row
                                                .source_ip
                                                .map(|ip| ip.to_string())
                                                .unwrap_or_default()}
                                        </ListTextItem>
                                        <ListTextItem>
                                            {record.row.policy_evaluated.disposition}
                                        </ListTextItem>
                                        <ListItem subclass="px-6 py-1.5">
                                            <button
                                                class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                                on:click=move |_| {
                                                    selected_record.set(record_id);
                                                    display_record.set(true);
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
                            current_page=page
                            total_results=Some(total_results)
                            page_size=PAGE_SIZE
                            on_page_change=move |new_page: u32| {
                                page.set(new_page);
                            }
                        />

                    </Footer>

                </ListTable>

            </div>

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
            fetch_selected_record
                .get()
                .map(|(record_id, record)| {
                    view! {
                        <ReportView>
                            <ReportSection title="Record">
                                <ReportItem label="Disposition">
                                    {record.row.policy_evaluated.disposition}
                                </ReportItem>
                                <ReportItem label="DKIM Result">
                                    {record.row.policy_evaluated.dkim}
                                </ReportItem>
                                <ReportItem label="SPF Result">
                                    {record.row.policy_evaluated.spf}
                                </ReportItem>
                                <ReportItem
                                    label="Envelope From"
                                    hide=record.identifiers.envelope_from.is_empty()
                                >
                                    <ReportTextValue value=record.identifiers.envelope_from/>
                                </ReportItem>
                                <ReportItem
                                    label="Header From"
                                    hide=record.identifiers.header_from.is_empty()
                                >
                                    <ReportTextValue value=record.identifiers.header_from/>
                                </ReportItem>
                                <ReportItem
                                    label="Envelope To"
                                    hide=record.identifiers.envelope_to.is_none()
                                >
                                    <ReportTextValue value=record
                                        .identifiers
                                        .envelope_to
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem label="Source IP" hide=record.row.source_ip.is_none()>
                                    <ReportTextValue value=record
                                        .row
                                        .source_ip
                                        .map(|ip| ip.to_string())
                                        .unwrap_or_default()/>
                                </ReportItem>
                                <ReportItem label="Count" hide=record.row.count == 0>
                                    <ReportTextValue value=record.row.count.to_string()/>
                                </ReportItem>
                                <ReportItem
                                    label="Override Reasons"
                                    hide=record.row.policy_evaluated.reason.is_empty()
                                >
                                    <ReportTextValue value=(record
                                        .row
                                        .policy_evaluated
                                        .reason
                                        .into_iter()
                                        .map(|reason| reason.to_string())
                                        .collect::<Vec<String>>()
                                        .join(", "))/>

                                </ReportItem>
                            </ReportSection>

                            {if !record.auth_results.dkim.is_empty() {
                                Some(
                                    view! {
                                        <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                                            <div class="sm:col-span-12 pb-4">
                                                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                                    DKIM Results
                                                </h2>
                                            </div>
                                            <Table headers=vec![
                                                "Domain".to_string(),
                                                "Selector".to_string(),
                                                "Result".to_string(),
                                                "Details".to_string(),
                                            ]>
                                                {record
                                                    .auth_results
                                                    .dkim
                                                    .into_iter()
                                                    .map(|dkim| {
                                                        view! {
                                                            <TableRow>
                                                                <span>{dkim.domain}</span>
                                                                <span>{dkim.selector}</span>
                                                                <span>{dkim.result}</span>
                                                                <span>{dkim.human_result.unwrap_or_default()}</span>

                                                            </TableRow>
                                                        }
                                                    })
                                                    .collect_view()}

                                            </Table>

                                        </div>
                                    },
                                )
                            } else {
                                None
                            }}

                            {if !record.auth_results.spf.is_empty() {
                                Some(
                                    view! {
                                        <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                                            <div class="sm:col-span-12 pb-4">
                                                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                                    SPF Results
                                                </h2>
                                            </div>
                                            <Table headers=vec![
                                                "Domain".to_string(),
                                                "Scope".to_string(),
                                                "Result".to_string(),
                                                "Details".to_string(),
                                            ]>
                                                {record
                                                    .auth_results
                                                    .spf
                                                    .into_iter()
                                                    .map(|spf| {
                                                        view! {
                                                            <TableRow>
                                                                <span>{spf.domain}</span>
                                                                <span>{spf.scope.to_string()}</span>
                                                                <span>{spf.result}</span>
                                                                <span>{spf.human_result.unwrap_or_default()}</span>

                                                            </TableRow>
                                                        }
                                                    })
                                                    .collect_view()}

                                            </Table>

                                        </div>
                                    },
                                )
                            } else {
                                None
                            }}

                            <div class="grid justify-center sm:flex sm:justify-between sm:items-center gap-1">
                                <div>
                                    <ItemPagination
                                        total_items=fetch_records.get_untracked().len() as u32
                                        current_item=record_id + 1
                                        on_item_change=move |new_record_id| {
                                            selected_record.set(new_record_id - 1);
                                        }
                                    />

                                </div>

                                <Button
                                    text="Close"
                                    color=Color::Blue
                                    on_click=move |_| {
                                        display_record.set(false);
                                    }
                                />

                            </div>
                        </ReportView>
                    }
                })
        }}
    }
}

impl IntoView for ActionDisposition {
    fn into_view(self) -> View {
        match self {
            ActionDisposition::Pass => view! {
                <Badge color=Color::Green>
                    <IconCheckCircle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            ActionDisposition::Quarantine => view! {
                <Badge color=Color::Yellow>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            ActionDisposition::Reject => view! {
                <Badge color=Color::Red>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            ActionDisposition::Unspecified | ActionDisposition::None => view! {
                <Badge color=Color::Blue>
                    <IconArrowRightCircle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
        }
    }
}

impl IntoView for DmarcResult {
    fn into_view(self) -> View {
        match self {
            DmarcResult::Pass => view! {
                <Badge color=Color::Green>
                    <IconCheckCircle attr:class="flex-shrink-0 size-3"/>

                    Pass

                </Badge>
            }
            .into_view(),
            DmarcResult::Fail => view! {
                <Badge color=Color::Red>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    Fail

                </Badge>
            }
            .into_view(),
            DmarcResult::Unspecified => view! {
                <Badge color=Color::Blue>
                    <IconArrowRightCircle attr:class="flex-shrink-0 size-3"/>

                    Unspecified

                </Badge>
            }
            .into_view(),
        }
    }
}

impl IntoView for DkimResult {
    fn into_view(self) -> View {
        match self {
            DkimResult::Pass => view! {
                <Badge color=Color::Green>
                    <IconCheckCircle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            DkimResult::Fail => view! {
                <Badge color=Color::Red>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            DkimResult::TempError => view! {
                <Badge color=Color::Yellow>
                    <IconClock attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            DkimResult::PermError => view! {
                <Badge color=Color::Red>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            DkimResult::None | DkimResult::Policy | DkimResult::Neutral => view! {
                <Badge color=Color::Blue>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
        }
    }
}

impl IntoView for SpfResult {
    fn into_view(self) -> View {
        match self {
            SpfResult::Pass => view! {
                <Badge color=Color::Green>
                    <IconCheckCircle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            SpfResult::Fail | SpfResult::SoftFail => view! {
                <Badge color=Color::Red>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            SpfResult::TempError => view! {
                <Badge color=Color::Yellow>
                    <IconClock attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            SpfResult::PermError => view! {
                <Badge color=Color::Red>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
            SpfResult::None | SpfResult::Neutral => view! {
                <Badge color=Color::Blue>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.to_string()}

                </Badge>
            }
            .into_view(),
        }
    }
}

impl Record {
    pub fn contains_string(&self, filter: &str) -> bool {
        filter.is_empty()
            || self.identifiers.envelope_from.contains(filter)
            || self.identifiers.header_from.contains(filter)
            || self
                .identifiers
                .envelope_to
                .as_ref()
                .map_or(false, |to| to.contains(filter))
            || self.auth_results.dkim.iter().any(|dkim| {
                dkim.domain.contains(filter)
                    || dkim.selector.contains(filter)
                    || dkim
                        .human_result
                        .as_ref()
                        .map_or(false, |r| r.contains(filter))
            })
            || self.auth_results.spf.iter().any(|spf| {
                spf.domain.contains(filter)
                    || spf
                        .human_result
                        .as_ref()
                        .map_or(false, |r| r.contains(filter))
            })
            || self
                .row
                .source_ip
                .as_ref()
                .map_or(false, |ip| ip.to_string().contains(filter))
    }

    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
