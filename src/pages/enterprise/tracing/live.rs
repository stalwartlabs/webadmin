/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: LicenseRef-SEL
 *
 * This file is subject to the Stalwart Enterprise License Agreement (SEL) and
 * is not open source software. It must not be modified or distributed without
 * explicit permission from Stalwart Labs Ltd.
 * Unauthorized use, modification, or distribution is strictly prohibited.
 */

use codee::string::JsonSerdeCodec;
use leptos::{
    component, create_action, create_effect, expect_context, view, Callback, For, IntoView,
    RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWith,
};
use leptos_use::{
    use_event_source_with_options, ReconnectLimit, UseEventSourceOptions, UseEventSourceReturn,
};
use std::sync::Arc;

use crate::{
    components::{
        form::{
            button::Button, input::InputText, Form, FormButtonBar, FormElement, FormItem,
            FormSection,
        },
        messages::alert::{use_alerts, Alert, Alerts},
        report::ReportView,
        Color,
    },
    core::{
        http::HttpRequest,
        oauth::use_authorization,
        schema::{Builder, Schemas, Transformer, Type},
        url::UrlBuilder,
    },
    pages::enterprise::tracing::{display::EventView, event::Event},
};

#[component]
pub fn LiveTracing() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let token: RwSignal<Option<String>> = RwSignal::new(None);
    let data = expect_context::<Arc<Schemas>>()
        .build_form("live-tracing")
        .into_signal();
    let start_live_telemetry = create_action(move |_| {
        let auth = auth.get();

        async move {
            match HttpRequest::get("/api/telemetry/live/token")
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

    view! {
        {move || {
            if let Some(auth_token) = token.get() {
                let auth = auth.get_untracked();
                let filter = data.get().value::<String>("filter").unwrap_or_default();
                let mut url_builer = UrlBuilder::new(
                    format!("{}/api/telemetry/traces/live/{}", auth.base_url, auth_token),
                );
                if !filter.is_empty() {
                    for keyword in filter.split_ascii_whitespace() {
                        if let Some((key, value)) = keyword.split_once(':') {
                            url_builer = url_builer
                                .with_parameter(key.trim().to_string(), value.trim());
                        } else {
                            url_builer = url_builer.with_parameter("filter", keyword);
                        }
                    }
                }
                let UseEventSourceReturn { data, error, close, .. } = use_event_source_with_options::<
                    Vec<Event>,
                    JsonSerdeCodec,
                >(
                    &url_builer.finish(),
                    UseEventSourceOptions::default()
                        .reconnect_limit(ReconnectLimit::Limited(5))
                        .reconnect_interval(2000)
                        .named_events(vec!["trace".to_string()]),
                );
                let span_history = RwSignal::new(Vec::new());
                create_effect(move |_| {
                    span_history
                        .update(|spans| {
                            let new_spans = data.get().unwrap_or_default();
                            if spans.len() + new_spans.len() > 1000 {
                                spans.drain(0..(spans.len() + new_spans.len() - 1000));
                            }
                            for event in new_spans {
                                spans.push(event);
                            }
                        });
                });
                create_effect(move |_| {
                    error
                        .with(|error| {
                            if let Some(err) = error {
                                alert.set(Alert::error(format!("Live tracing error: {}", err)));
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
                                        Live Tracing
                                    </h2>
                                </div>

                                <div>
                                    <For
                                        each=move || span_history.get().clone()
                                        key=|event| event.id()
                                        let:event
                                    >
                                        <EventView event=event span_start=None/>
                                    </For>

                                </div>

                            </div>

                            <div class="flex justify-end">

                                <Button
                                    text="Stop"
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
                    <Form title="Live Tracing" subtitle="Real-time tracing of server events">

                        <FormSection>
                            <FormItem
                                label="Filter"
                                tooltip="An optional filter to limit the events displayed."
                            >
                                <InputText
                                    placeholder="remote-ip:192.168.0.1 or domain:example.com or keyword"
                                    element=FormElement::new("filter", data)
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
                                            start_live_telemetry.dispatch(());
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

impl Builder<Schemas, ()> {
    pub fn build_live_tracing(self) -> Self {
        self.new_schema("live-tracing")
            .new_field("filter")
            .input_check([Transformer::Lowercase], [])
            .typ(Type::Input)
            .build()
            .build()
    }
}
