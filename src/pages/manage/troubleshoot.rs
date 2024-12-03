/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use chrono::Duration;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_router::use_query_map;
use leptos_use::{
    use_event_source_with_options, ReconnectLimit, UseEventSourceOptions, UseEventSourceReturn,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use crate::{
    components::{
        badge::Badge,
        card::{Card, CardItem},
        form::{
            button::Button,
            input::{InputText, TextArea},
            Form, FormButtonBar, FormElement, FormItem, FormSection,
        },
        icon::{
            IconAlertTriangle, IconArrowRightCircle, IconCancel, IconCheckCircle, IconClock,
            IconExclamationTriangle,
        },
        messages::alert::{use_alerts, Alert, Alerts},
        report::{ReportItem, ReportSection, ReportTextValue, ReportView},
        Color,
    },
    core::{
        http::HttpRequest,
        oauth::use_authorization,
        schema::{Builder, Schemas, Transformer, Type, Validator},
        url::UrlBuilder,
    },
};

#[component]
pub fn TroubleshootDelivery() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let query = use_query_map();
    let token: RwSignal<Option<String>> = RwSignal::new(None);
    let data = expect_context::<Arc<Schemas>>()
        .build_form("troubleshoot-delivery")
        .into_signal();
    let start_troubleshoot = create_action(move |_| {
        let auth = auth.get();

        async move {
            match HttpRequest::get("/api/troubleshoot/token")
                .with_authorization(&auth)
                .send::<String>()
                .await
            {
                Ok(auth_token) => {
                    token.set(Some(auth_token));
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });
    create_effect(move |_| {
        if let Some(email) = query.get().get("target") {
            data.update(|data| {
                data.set("email", email);
            });
        }
    });

    view! {
        {move || {
            if let Some(auth_token) = token.get() {
                let auth = auth.get_untracked();
                let email = data.get().value::<String>("email").unwrap_or_default();
                let url_builer = UrlBuilder::new(
                        format!("{}/api/troubleshoot/delivery", auth.base_url),
                    )
                    .with_subpath(&email)
                    .with_parameter("token", auth_token);
                let UseEventSourceReturn { data, error, close, .. } = use_event_source_with_options::<
                    Vec<DeliveryStage>,
                    JsonSerdeCodec,
                >(
                    &url_builer.finish(),
                    UseEventSourceOptions::default()
                        .reconnect_limit(ReconnectLimit::Limited(1))
                        .reconnect_interval(2000)
                        .named_events(vec!["event".to_string()]),
                );
                let span_history = RwSignal::new(Vec::new());
                let close_ = close.clone();
                create_effect(move |_| {
                    span_history
                        .update(|spans| {
                            let new_spans = data.get().unwrap_or_default();
                            for event in new_spans {
                                if !matches!(event, DeliveryStage::Completed) {
                                    spans.push(event);
                                } else {
                                    close_();
                                }
                            }
                        });
                });
                create_effect(move |_| {
                    error
                        .with(|error| {
                            if let Some(err) = error {
                                alert.set(Alert::error(format!("Endpoint error: {}", err)));
                            }
                        });
                });
                view! {
                    <div>
                        <Alerts/>
                        <ReportView>
                            <div class="gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent">
                                <div class="sm:col-span-12 pb-10">
                                    <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                                        Email Delivery Process
                                    </h2>
                                </div>

                                <div>
                                    <For
                                        each=move || {
                                            span_history.get().clone().into_iter().enumerate()
                                        }

                                        key=|(idx, _)| *idx
                                        let:event
                                    >
                                        <StageView event=event.1/>
                                    </For>

                                </div>

                            </div>

                            <div class="flex justify-end">

                                <Button
                                    text="Close"
                                    color=Color::Blue
                                    on_click=move |_| {
                                        close();
                                        token.set(None);
                                    }
                                />

                            </div>
                        </ReportView>
                    </div>
                }
                    .into_view()
            } else {
                view! {
                    <Form
                        title="Delivery Troubleshooting"
                        subtitle="Troubleshoot or test e-mail delivery"
                    >

                        <FormSection>
                            <FormItem
                                label="E-mail or Domain"
                                tooltip="E-mail address or domain name to troubleshoot."
                            >
                                <InputText
                                    placeholder="john@example.org or example.org"
                                    element=FormElement::new("email", data)
                                />
                            </FormItem>

                        </FormSection>

                        <FormButtonBar>

                            <Button
                                text="Start"
                                color=Color::Blue
                                on_click=Callback::new(move |_| {
                                    data.update(|data| {
                                        if data.validate_form() {
                                            start_troubleshoot.dispatch(());
                                        }
                                    });
                                })
                            />

                        </FormButtonBar>

                    </Form>
                }
                    .into_view()
            }
        }}
    }
}

#[component]
pub fn TroubleshootDmarc() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let query = use_query_map();
    let response: RwSignal<Option<DmarcTroubleshootResponse>> = RwSignal::new(None);
    let data = expect_context::<Arc<Schemas>>()
        .build_form("troubleshoot-dmarc")
        .into_signal();
    let in_flight = RwSignal::new(false);
    let send_request = create_action(move |request: &DmarcTroubleshootRequest| {
        let auth = auth.get();
        let request = request.clone();

        async move {
            in_flight.set(true);
            match HttpRequest::post("/api/troubleshoot/dmarc")
                .with_authorization(&auth)
                .with_body(request)
                .unwrap()
                .send::<DmarcTroubleshootResponse>()
                .await
            {
                Ok(dmarc_response) => {
                    response.set(Some(dmarc_response));
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
            in_flight.set(false);
        }
    });
    create_effect(move |_| {
        if let Some(mail_from) = query.get().get("target") {
            data.update(|data| {
                data.set("mail_from", mail_from);
            });
        }
    });

    view! {
        {move || {
            if let Some(dmarc) = response.get() {
                let dkim_result = if dmarc.dkim_pass {
                    AuthResult::Pass
                } else {
                    dmarc.dkim_results.first().cloned().unwrap_or(AuthResult::None)
                };
                let spf_ehlo_error = dmarc.spf_ehlo_result.error();
                let spf_mail_from_error = dmarc.spf_mail_from_result.error();
                let arc_error = dmarc.arc_result.error();
                let dmarc_error = dmarc.dmarc_result.error();
                let dmarc_icon = dmarc.dmarc_result.icon();
                let dmarc_text = dmarc.dmarc_result.text();
                let spf_icon = dmarc.spf_mail_from_result.icon();
                let spf_text = dmarc.spf_mail_from_result.text();
                let arc_icon = dmarc.arc_result.icon();
                let arc_text = dmarc.arc_result.text();
                view! {
                    <div>
                        <Alerts/>
                        <Card>
                            <CardItem title="DMARC" contents=dmarc_text>

                                {dmarc_icon}

                            </CardItem>
                            <CardItem title="DKIM" contents=dkim_result.text()>

                                {dkim_result.icon()}

                            </CardItem>
                            <CardItem title="SPF" contents=spf_text>

                                {spf_icon}

                            </CardItem>
                            <CardItem title="ARC" contents=arc_text>

                                {arc_icon}

                            </CardItem>

                        </Card>
                        <ReportView>
                            <ReportSection title="DMARC Analysis Result">
                                <ReportItem label="SPF EHLO Domain">
                                    <ReportTextValue value=dmarc.spf_ehlo_domain/>
                                </ReportItem>
                                <ReportItem label="SPF EHLO Result">
                                    {dmarc.spf_ehlo_result.into_view()}
                                </ReportItem>
                                {spf_ehlo_error
                                    .map(|error| {
                                        view! {
                                            <ReportItem label="SPF EHLO Error">
                                                <ReportTextValue value=error/>
                                            </ReportItem>
                                        }
                                    })}

                                <ReportItem label="SPF Mail From Domain">
                                    <ReportTextValue value=dmarc.spf_mail_from_domain/>
                                </ReportItem>
                                <ReportItem label="SPF Mail From Result">
                                    {dmarc.spf_mail_from_result.into_view()}
                                </ReportItem>
                                {spf_mail_from_error
                                    .map(|error| {
                                        view! {
                                            <ReportItem label="SPF Mail From Error">
                                                <ReportTextValue value=error/>
                                            </ReportItem>
                                        }
                                    })}

                                <ReportItem label="Reverse IP Validation">
                                    {dmarc.ip_rev_result.into_view()}
                                </ReportItem>
                                <ReportItem label="PTR Records">
                                    <ReportTextValue value=dmarc.ip_rev_ptr.join(", ")/>
                                </ReportItem>
                                {dmarc
                                    .dkim_results
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, dkim)| {
                                        let error = dkim.error();
                                        view! {
                                            <ReportItem label=format!(
                                                "DKIM Result #{}",
                                                idx + 1,
                                            )>{dkim.into_view()}</ReportItem>
                                            {error
                                                .map(|error| {
                                                    view! {
                                                        <ReportItem label=format!("DKIM Error #{}", idx + 1)>
                                                            <ReportTextValue value=error/>
                                                        </ReportItem>
                                                    }
                                                })}
                                        }
                                    })
                                    .collect_view()}
                                <ReportItem label="ARC Result">
                                    {dmarc.arc_result.into_view()}
                                </ReportItem>
                                {arc_error
                                    .map(|error| {
                                        view! {
                                            <ReportItem label="ARC Error">
                                                <ReportTextValue value=error/>
                                            </ReportItem>
                                        }
                                    })}

                                <ReportItem label="DMARC Policy">
                                    {dmarc.dmarc_policy.into_view()}
                                </ReportItem>
                                <ReportItem label="DMARC Result">
                                    {dmarc.dmarc_result.into_view()}
                                </ReportItem>
                                {dmarc_error
                                    .map(|error| {
                                        view! {
                                            <ReportItem label="DMARC Error">
                                                <ReportTextValue value=error/>
                                            </ReportItem>
                                        }
                                    })}

                            </ReportSection>

                            <div class="flex justify-end">

                                <Button
                                    text="Close"
                                    color=Color::Blue
                                    on_click=move |_| {
                                        response.set(None);
                                    }
                                />

                            </div>
                        </ReportView>
                    </div>
                }
                    .into_view()
            } else {
                view! {
                    <Form
                        title="DMARC Troubleshooting"
                        subtitle="Troubleshoot or test DMARC policy enforcement"
                    >

                        <FormSection>
                            <FormItem label="Sender Address" tooltip="The SMTP MAIL FROM address.">
                                <InputText
                                    placeholder="john@example.org"
                                    element=FormElement::new("mail_from", data)
                                />
                            </FormItem>
                            <FormItem
                                label="EHLO Hostname"
                                tooltip=concat!(
                                    "The hostname used in the SMTP EHLO stage. ",
                                    "When testing DMARC for a local domain, ",
                                    "enter here your server's hostname.",
                                )
                            >

                                <InputText
                                    placeholder="mx.example.org"
                                    element=FormElement::new("ehlo_domain", data)
                                />
                            </FormItem>
                            <FormItem
                                label="IP Address"
                                tooltip=concat!(
                                    "The SMTP server's IP address. ",
                                    "When testing DMARC for a local domain, ",
                                    "enter here your server's IP address.",
                                )
                            >

                                <InputText
                                    placeholder="192.168.0.1"
                                    element=FormElement::new("remote_ip", data)
                                />
                            </FormItem>
                            <FormItem
                                label="Headers"
                                tooltip=concat!(
                                    "Message headers including DKIM signatures. ",
                                    "Leave blank to test SPF only.",
                                )
                            >

                                <TextArea element=FormElement::new("headers", data)/>
                            </FormItem>

                        </FormSection>

                        <FormButtonBar>

                            <Button
                                text="Start"
                                color=Color::Blue
                                disabled=in_flight
                                on_click=Callback::new(move |_| {
                                    data.update(|data| {
                                        if data.validate_form() {
                                            send_request
                                                .dispatch(DmarcTroubleshootRequest {
                                                    remote_ip: data
                                                        .get("remote_ip")
                                                        .unwrap_or_default()
                                                        .parse()
                                                        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                                                    ehlo_domain: data
                                                        .get("ehlo_domain")
                                                        .unwrap_or_default()
                                                        .to_string(),
                                                    mail_from: data
                                                        .get("mail_from")
                                                        .unwrap_or_default()
                                                        .to_string(),
                                                    headers: data
                                                        .get("headers")
                                                        .and_then(|h| {
                                                            if h.trim().is_empty() { None } else { Some(h.to_string()) }
                                                        }),
                                                });
                                        }
                                    });
                                })
                            />

                        </FormButtonBar>

                    </Form>
                }
                    .into_view()
            }
        }}
    }
}

#[component]
fn StageView(event: DeliveryStage) -> impl IntoView {
    let icon = event.icon();
    let details = event.details();

    view! {
        <div class="group relative flex gap-x-5">

            <div class="relative group-last:after:hidden after:absolute after:top-8 after:bottom-2 after:start-3 after:w-px after:-translate-x-[0.5px] after:bg-gray-200 dark:after:bg-neutral-700">
                <div class="relative z-10 size-6 flex justify-center items-center">{icon}</div>
            </div>

            <div class="grow pb-8 group-last:pb-0">

                {details
                    .elapsed
                    .map(|elapsed| {
                        view! {
                            <h3 class="mb-1 text-xs text-gray-600 dark:text-neutral-400">

                                {format!(
                                    "Completed in {}",
                                    HumanTime::from(elapsed)
                                        .to_text_en(Accuracy::Precise, Tense::Present),
                                )}

                            </h3>
                        }
                    })}
                <p class="font-semibold text-sm text-gray-800 dark:text-neutral-200">
                    {details.title}
                </p>
                <p class="mt-1 text-sm text-gray-600 dark:text-neutral-400">{details.subtitle}</p>
                <ul class="list-disc ms-6 mt-3 space-y-1.5">

                    {details
                        .items
                        .into_iter()
                        .map(|item| {
                            view! {
                                <li class="ps-1 text-sm text-gray-600 dark:text-neutral-400">
                                    {item}
                                </li>
                            }
                        })
                        .collect_view()}

                </ul>
            </div>

        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum DeliveryStage {
    MxLookupStart {
        domain: String,
    },
    MxLookupSuccess {
        mxs: Vec<MX>,
        elapsed: u64,
    },
    MxLookupError {
        reason: String,
        elapsed: u64,
    },
    MtaStsFetchStart,
    MtaStsFetchSuccess {
        policy: Policy,
        elapsed: u64,
    },
    MtaStsFetchError {
        reason: String,
        elapsed: u64,
    },
    MtaStsNotFound {
        elapsed: u64,
    },
    TlsRptLookupStart,
    TlsRptLookupSuccess {
        rua: Vec<ReportUri>,
        elapsed: u64,
    },
    TlsRptLookupError {
        reason: String,
        elapsed: u64,
    },
    TlsRptNotFound {
        elapsed: u64,
    },
    DeliveryAttemptStart {
        hostname: String,
    },
    MtaStsVerifySuccess,
    MtaStsVerifyError {
        reason: String,
    },
    TlsaLookupStart,
    TlsaLookupSuccess {
        record: Tlsa,
        elapsed: u64,
    },
    TlsaNotFound {
        elapsed: u64,
        reason: String,
    },
    TlsaLookupError {
        elapsed: u64,
        reason: String,
    },
    IpLookupStart,
    IpLookupSuccess {
        remote_ips: Vec<IpAddr>,
        elapsed: u64,
    },
    IpLookupError {
        reason: String,
        elapsed: u64,
    },
    ConnectionStart {
        remote_ip: IpAddr,
    },
    ConnectionSuccess {
        elapsed: u64,
    },
    ConnectionError {
        elapsed: u64,
        reason: String,
    },
    ReadGreetingStart,
    ReadGreetingSuccess {
        elapsed: u64,
    },
    ReadGreetingError {
        elapsed: u64,
        reason: String,
    },
    EhloStart,
    EhloSuccess {
        elapsed: u64,
    },
    EhloError {
        elapsed: u64,
        reason: String,
    },
    StartTlsStart,
    StartTlsSuccess {
        elapsed: u64,
    },
    StartTlsError {
        elapsed: u64,
        reason: String,
    },
    DaneVerifySuccess,
    DaneVerifyError {
        reason: String,
    },
    MailFromStart,
    MailFromSuccess {
        elapsed: u64,
    },
    MailFromError {
        reason: String,
        elapsed: u64,
    },
    RcptToStart,
    RcptToSuccess {
        elapsed: u64,
    },
    RcptToError {
        reason: String,
        elapsed: u64,
    },
    QuitStart,
    QuitCompleted {
        elapsed: u64,
    },
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct MX {
    pub exchanges: Vec<String>,
    pub preference: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ReportUri {
    Mail { email: String },
    Http { url: String },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsaEntry {
    pub is_end_entity: bool,
    pub is_sha256: bool,
    pub is_spki: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tlsa {
    pub entries: Vec<TlsaEntry>,
    pub has_end_entities: bool,
    pub has_intermediates: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Enforce,
    Testing,
    #[default]
    None,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MxPattern {
    Equals(String),
    StartsWith(String),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub mode: Mode,
    pub mx: Vec<MxPattern>,
    pub max_age: u64,
}

#[derive(Default)]
struct Details {
    title: String,
    subtitle: String,
    items: Vec<String>,
    elapsed: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DmarcTroubleshootRequest {
    #[serde(rename = "remoteIp")]
    remote_ip: IpAddr,
    #[serde(rename = "ehloDomain")]
    ehlo_domain: String,
    #[serde(rename = "mailFrom")]
    mail_from: String,
    headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DmarcTroubleshootResponse {
    #[serde(rename = "spfEhloDomain")]
    spf_ehlo_domain: String,
    #[serde(rename = "spfEhloResult")]
    spf_ehlo_result: AuthResult,
    #[serde(rename = "spfMailFromDomain")]
    spf_mail_from_domain: String,
    #[serde(rename = "spfMailFromResult")]
    spf_mail_from_result: AuthResult,
    #[serde(rename = "ipRevResult")]
    ip_rev_result: AuthResult,
    #[serde(rename = "ipRevPtr")]
    ip_rev_ptr: Vec<String>,
    #[serde(rename = "dkimResults")]
    dkim_results: Vec<AuthResult>,
    #[serde(rename = "dkimPass")]
    dkim_pass: bool,
    #[serde(rename = "arcResult")]
    arc_result: AuthResult,
    #[serde(rename = "dmarcResult")]
    dmarc_result: AuthResult,
    #[serde(rename = "dmarcPass")]
    dmarc_pass: bool,
    #[serde(rename = "dmarcPolicy")]
    dmarc_policy: DmarcPolicy,
    elapsed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum AuthResult {
    Pass,
    Fail { details: Option<String> },
    SoftFail { details: Option<String> },
    TempError { details: Option<String> },
    PermError { details: Option<String> },
    Neutral { details: Option<String> },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DmarcPolicy {
    None,
    Quarantine,
    Reject,
    Unspecified,
}

impl DeliveryStage {
    pub fn details(self) -> Details {
        match self {
            DeliveryStage::MxLookupStart { domain } => Details::new(
                format!("MX Lookup for {domain}"),
                format!("Querying MX records for domain {domain}.."),
            ),
            DeliveryStage::MxLookupSuccess { mxs, elapsed } => Details::new(
                "MX Lookup Successful",
                "Successfully fetched MX records for domain.",
            )
            .elapsed(elapsed)
            .items(mxs.into_iter().map(|mx| {
                format!(
                    "{} with preference {}",
                    mx.exchanges.join(", "),
                    mx.preference
                )
            })),
            DeliveryStage::MxLookupError { reason, elapsed } => {
                Details::new("MX Lookup Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::MtaStsFetchStart => Details::new(
                "MTA-STS Policy Fetch",
                "Fetching MTA-STS policy for domain...",
            ),
            DeliveryStage::MtaStsFetchSuccess { policy, elapsed } => Details::new(
                "MTA-STS Policy Fetched Successfully",
                "Successfully fetched MTA-STS policy for domain",
            )
            .elapsed(elapsed)
            .with_items(policy.items()),
            DeliveryStage::MtaStsFetchError { reason, elapsed } => {
                Details::new("MTA-STS Policy Fetch Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::MtaStsNotFound { elapsed } => Details::new(
                "MTA-STS Policy Unavailable",
                concat!(
                    "The domain does not publish an MTA-STS policy. ",
                    "Publishing an MTA-STS policy is recommended but not required ",
                    "for a successful message delivery."
                ),
            )
            .elapsed(elapsed),
            DeliveryStage::TlsRptLookupStart => Details::new(
                "TLS-RPT Record Fetch",
                "Fetching TLS Reporting record for host...",
            ),
            DeliveryStage::TlsRptLookupSuccess { rua, elapsed } => Details::new(
                "TLS-RPT Record Fetched Successfully",
                "TLS Reporting record for host fetched successfully.",
            )
            .elapsed(elapsed)
            .items(rua.into_iter().map(|uri| match uri {
                ReportUri::Mail { email } => format!("Send TLS report to e-mail {email}"),
                ReportUri::Http { url } => format!("Submit TLS report to URL {url}"),
            })),
            DeliveryStage::TlsRptLookupError { reason, elapsed } => {
                Details::new("TLS-RPT Record Fetch Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::TlsRptNotFound { elapsed } => Details::new(
                "TLS-RPT Record Not Found",
                concat!(
                    "A TLS Reporting record was not found for this domain. ",
                    "TLS-RPT Records are optional and not required for a ",
                    "successful message delivery."
                ),
            )
            .elapsed(elapsed),
            DeliveryStage::DeliveryAttemptStart { hostname } => Details::new(
                format!("Delivery attempt to host {hostname}"),
                format!("Attempting to deliver message to host {hostname}..."),
            ),
            DeliveryStage::MtaStsVerifySuccess => Details::new(
                "MTA-STS Verification Successful",
                "This host is authorized by the published MTA-STS policy.",
            ),
            DeliveryStage::MtaStsVerifyError { reason } => {
                Details::new("MTA-STS Verification Failed", reason)
            }
            DeliveryStage::TlsaLookupStart => {
                Details::new("TLSA Record Lookup", "Looking up TLSA records for host...")
            }
            DeliveryStage::TlsaLookupSuccess { record, elapsed } => Details::new(
                "TLSA Record Lookup Successful",
                "TLSA record for host fetched successfully.",
            )
            .with_items(record.items())
            .elapsed(elapsed),
            DeliveryStage::TlsaNotFound { elapsed, reason } => {
                Details::new("TLSA Record Not Found", reason).elapsed(elapsed)
            }
            DeliveryStage::TlsaLookupError { elapsed, reason } => {
                Details::new("TLSA Record Lookup Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::IpLookupStart => Details::new(
                "IP Address Lookup",
                "Looking up A and AAAA records for host...",
            ),
            DeliveryStage::IpLookupSuccess {
                remote_ips,
                elapsed,
            } => Details::new(
                "IP Address Lookup Successful",
                "Successfully fetched A/AAAA records for host.",
            )
            .elapsed(elapsed)
            .items(remote_ips.into_iter().map(|ip| ip.to_string())),
            DeliveryStage::IpLookupError { reason, elapsed } => {
                Details::new("IP Address Lookup Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::ConnectionStart { remote_ip } => Details::new(
                format!("Connecting to {remote_ip}"),
                format!("Attempting to establish TCP connection to {remote_ip} on port 25..."),
            ),
            DeliveryStage::ConnectionSuccess { elapsed } => Details::new(
                "Connection Established",
                "Successfully connected to remote SMTP server.",
            )
            .elapsed(elapsed),
            DeliveryStage::ConnectionError { elapsed, reason } => {
                Details::new("Connection Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::ReadGreetingStart => Details::new(
                "SMTP Greeting Read",
                "Reading SMTP greeting from remote host...",
            ),
            DeliveryStage::ReadGreetingSuccess { elapsed } => Details::new(
                "SMTP Greeting Read Successfully",
                "Successfully read SMTP greeting.",
            )
            .elapsed(elapsed),
            DeliveryStage::ReadGreetingError { elapsed, reason } => {
                Details::new("SMTP Greeting Read Error", reason).elapsed(elapsed)
            }
            DeliveryStage::EhloStart => {
                Details::new("EHLO Stage", "Sending EHLO command to remote host...")
            }
            DeliveryStage::EhloSuccess { elapsed } => Details::new(
                "EHLO Command Accepted",
                "EHLO command accepted by remote host.",
            )
            .elapsed(elapsed),
            DeliveryStage::EhloError { elapsed, reason } => {
                Details::new("EHLO Command Rejected", reason).elapsed(elapsed)
            }
            DeliveryStage::StartTlsStart => Details::new(
                "Starting TLS",
                "Attempting to upgrade clear-text connection to TLS...",
            ),
            DeliveryStage::StartTlsSuccess { elapsed } => Details::new(
                "TLS Handshake Successful",
                "Successfully upgraded the connection to TLS.",
            )
            .elapsed(elapsed),
            DeliveryStage::StartTlsError { elapsed, reason } => {
                Details::new("STARTTLS Command Failed", reason).elapsed(elapsed)
            }
            DeliveryStage::DaneVerifySuccess => Details::new(
                "DANE Verification Successful",
                "Matching TLSA record found for the provided TLS certificate.",
            ),
            DeliveryStage::DaneVerifyError { reason } => {
                Details::new("DANE Verification Failed", reason)
            }
            DeliveryStage::MailFromStart => Details::new(
                "MAIL FROM Stage",
                "Sending MAIL FROM command to remote host...",
            ),
            DeliveryStage::MailFromSuccess { elapsed } => Details::new(
                "MAIL FROM Command Accepted",
                "Message sender accepted by remote host.",
            )
            .elapsed(elapsed),
            DeliveryStage::MailFromError { reason, elapsed } => {
                Details::new("MAIL FROM Command Rejected", reason).elapsed(elapsed)
            }
            DeliveryStage::RcptToStart => {
                Details::new("RCPT TO Stage", "Sending RCPT TO command to remote host...")
            }
            DeliveryStage::RcptToSuccess { elapsed } => Details::new(
                "RCPT TO Accepted",
                "Message recipient accepted by remote host.",
            )
            .elapsed(elapsed),
            DeliveryStage::RcptToError { reason, elapsed } => {
                Details::new("RCPT TO Command Rejected", reason).elapsed(elapsed)
            }
            DeliveryStage::QuitStart => Details::new(
                "Close Connection",
                "Sending QUIT command and closing connection...",
            ),
            DeliveryStage::QuitCompleted { elapsed } => {
                Details::new("Connection Closed", "SMTP Transaction finished.").elapsed(elapsed)
            }
            DeliveryStage::Completed => unreachable!(),
        }
    }

    pub fn icon(&self) -> View {
        const CLASS: &str = "shrink-0 size-6 text-gray-600 dark:text-neutral-400";
        const SIZE: usize = 24;

        match self {
            DeliveryStage::MxLookupStart { .. }
            | DeliveryStage::MtaStsFetchStart
            | DeliveryStage::TlsRptLookupStart
            | DeliveryStage::DeliveryAttemptStart { .. }
            | DeliveryStage::TlsaLookupStart
            | DeliveryStage::IpLookupStart
            | DeliveryStage::ConnectionStart { .. }
            | DeliveryStage::ReadGreetingStart
            | DeliveryStage::EhloStart
            | DeliveryStage::StartTlsStart
            | DeliveryStage::MailFromStart
            | DeliveryStage::RcptToStart
            | DeliveryStage::QuitStart => {
                view! { <IconClock size=SIZE attr:class=CLASS/> }
            }

            DeliveryStage::MxLookupSuccess { .. }
            | DeliveryStage::MtaStsFetchSuccess { .. }
            | DeliveryStage::TlsRptLookupSuccess { .. }
            | DeliveryStage::MtaStsVerifySuccess
            | DeliveryStage::TlsaLookupSuccess { .. }
            | DeliveryStage::IpLookupSuccess { .. }
            | DeliveryStage::ConnectionSuccess { .. }
            | DeliveryStage::ReadGreetingSuccess { .. }
            | DeliveryStage::EhloSuccess { .. }
            | DeliveryStage::StartTlsSuccess { .. }
            | DeliveryStage::DaneVerifySuccess
            | DeliveryStage::MailFromSuccess { .. }
            | DeliveryStage::RcptToSuccess { .. }
            | DeliveryStage::QuitCompleted { .. } => {
                view! { <IconCheckCircle size=SIZE attr:class=CLASS/> }
            }

            DeliveryStage::MtaStsNotFound { .. }
            | DeliveryStage::TlsRptNotFound { .. }
            | DeliveryStage::TlsaNotFound { .. } => {
                view! { <IconExclamationTriangle size=SIZE attr:class=CLASS/> }
            }

            DeliveryStage::MxLookupError { .. }
            | DeliveryStage::MtaStsFetchError { .. }
            | DeliveryStage::TlsRptLookupError { .. }
            | DeliveryStage::MtaStsVerifyError { .. }
            | DeliveryStage::TlsaLookupError { .. }
            | DeliveryStage::IpLookupError { .. }
            | DeliveryStage::ConnectionError { .. }
            | DeliveryStage::ReadGreetingError { .. }
            | DeliveryStage::EhloError { .. }
            | DeliveryStage::StartTlsError { .. }
            | DeliveryStage::DaneVerifyError { .. }
            | DeliveryStage::MailFromError { .. }
            | DeliveryStage::RcptToError { .. } => {
                view! { <IconCancel size=SIZE attr:class=CLASS/> }
            }
            DeliveryStage::Completed => unreachable!(),
        }
    }
}

impl Details {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Details {
            title: title.into(),
            subtitle: subtitle.into(),
            ..Default::default()
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = String>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn elapsed(mut self, elapsed: u64) -> Self {
        self.elapsed = Some(Duration::from_std(std::time::Duration::from_millis(elapsed)).unwrap());
        self
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }
}

impl Policy {
    fn items(self) -> Vec<String> {
        let mut items = Vec::with_capacity(3 + self.mx.len());
        items.push(
            match self.mode {
                Mode::Enforce => "Enforce policy",
                Mode::Testing => "Testing policy",
                Mode::None => "Other or unknown policy",
            }
            .to_string(),
        );
        for mx in self.mx {
            items.push(match mx {
                MxPattern::Equals(host) => format!("Policy authorizes MX {host}"),
                MxPattern::StartsWith(host) => format!("Policy authorizes MXs *.{host}"),
            });
        }
        items.push(format!("Policy ID is {}", self.id));
        items.push(format!("Policy max-age is {}", self.max_age));
        items
    }
}

impl Tlsa {
    fn items(self) -> Vec<String> {
        let mut items = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            let mut item = format!(
                "{} with {} hash ",
                if entry.is_end_entity {
                    "End entity"
                } else {
                    "Intermediate entity"
                },
                if entry.is_sha256 {
                    "SHA-256"
                } else {
                    "SHA-512"
                },
            );
            for byte in entry.data {
                item.push_str(&format!("{:x}", byte));
            }
            items.push(item);
        }
        items
    }
}

impl AuthResult {
    pub fn text(&self) -> &'static str {
        match self {
            AuthResult::Pass => "Pass",
            AuthResult::Fail { .. } => "Fail",
            AuthResult::SoftFail { .. } => "SoftFail",
            AuthResult::TempError { .. } => "TempError",
            AuthResult::PermError { .. } => "PermError",
            AuthResult::Neutral { .. } => "Neutral",
            AuthResult::None => "None",
        }
    }

    pub fn error(&self) -> Option<String> {
        match self {
            AuthResult::Fail { details } => details.clone(),
            AuthResult::SoftFail { details } => details.clone(),
            AuthResult::TempError { details } => details.clone(),
            AuthResult::PermError { details } => details.clone(),
            AuthResult::Neutral { details } => details.clone(),
            AuthResult::Pass | AuthResult::None => None,
        }
    }

    pub fn icon(&self) -> View {
        const CLASS: &str = "flex-shrink-0 size-5 text-gray-400 dark:text-gray-600";

        match self {
            AuthResult::Pass => view! { <IconCheckCircle attr:class=CLASS/> },
            AuthResult::Fail { .. }
            | AuthResult::SoftFail { .. }
            | AuthResult::PermError { .. } => view! { <IconCancel attr:class=CLASS/> },
            AuthResult::TempError { .. } | AuthResult::Neutral { .. } | AuthResult::None => {
                view! { <IconExclamationTriangle attr:class=CLASS/> }
            }
        }
    }
}

impl IntoView for AuthResult {
    fn into_view(self) -> View {
        match self {
            AuthResult::Pass => view! {
                <Badge color=Color::Green>
                    <IconCheckCircle attr:class="flex-shrink-0 size-3"/>

                    {self.text().to_string()}

                </Badge>
            }
            .into_view(),
            AuthResult::Fail { .. } => view! {
                <Badge color=Color::Red>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    {self.text().to_string()}

                </Badge>
            }
            .into_view(),
            AuthResult::TempError { .. } | AuthResult::SoftFail { .. } => view! {
                <Badge color=Color::Yellow>
                    <IconClock attr:class="flex-shrink-0 size-3"/>

                    {self.text().to_string()}

                </Badge>
            }
            .into_view(),
            AuthResult::PermError { .. } => view! {
                <Badge color=Color::Red>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.text().to_string()}

                </Badge>
            }
            .into_view(),
            AuthResult::None | AuthResult::Neutral { .. } => view! {
                <Badge color=Color::Blue>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    {self.text().to_string()}

                </Badge>
            }
            .into_view(),
        }
    }
}

impl IntoView for DmarcPolicy {
    fn into_view(self) -> View {
        match self {
            DmarcPolicy::Quarantine => view! {
                <Badge color=Color::Yellow>
                    <IconAlertTriangle attr:class="flex-shrink-0 size-3"/>

                    Quarantine

                </Badge>
            }
            .into_view(),
            DmarcPolicy::Reject => view! {
                <Badge color=Color::Red>
                    <IconCancel attr:class="flex-shrink-0 size-3"/>

                    Reject

                </Badge>
            }
            .into_view(),
            DmarcPolicy::Unspecified | DmarcPolicy::None => view! {
                <Badge color=Color::Blue>
                    <IconArrowRightCircle attr:class="flex-shrink-0 size-3"/>

                    None

                </Badge>
            }
            .into_view(),
        }
    }
}

impl Builder<Schemas, ()> {
    pub fn build_troubleshoot(self) -> Self {
        self.new_schema("troubleshoot-delivery")
            .new_field("email")
            .input_check(
                [Transformer::Lowercase, Transformer::Trim],
                [Validator::Required],
            )
            .typ(Type::Input)
            .build()
            .build()
            .new_schema("troubleshoot-dmarc")
            .new_field("mail_from")
            .input_check(
                [Transformer::Lowercase, Transformer::Trim],
                [Validator::Required, Validator::IsEmail],
            )
            .typ(Type::Input)
            .build()
            .new_field("remote_ip")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsIpOrMask],
            )
            .typ(Type::Input)
            .build()
            .new_field("ehlo_domain")
            .input_check([Transformer::Trim], [Validator::Required])
            .typ(Type::Input)
            .build()
            .new_field("headers")
            .typ(Type::Input)
            .build()
            .build()
    }
}
