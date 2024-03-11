use std::{collections::HashSet, sync::Arc};

use leptos::*;
use leptos_router::*;

use crate::{
    components::{
        icon::{IconAdd, IconTrash},
        list::{
            header::ColumnList,
            pagination::Pagination,
            row::SelectItem,
            toolbar::{SearchBox, ToolbarButton},
            Footer, ListItem, ListSection, ListTable, ListTextItem, Toolbar, ZeroResults,
        },
        messages::{
            alert::{use_alerts, Alert},
            modal::{use_modals, Modal},
        },
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
        url::UrlBuilder,
    },
    pages::{config::Schemas, maybe_plural, List},
};

use super::{Schema, Settings, UpdateSettings};

#[component]
pub fn SettingsList(id: &'static str) -> impl IntoView {
    let schema = expect_context::<Arc<Schemas>>().get(id);
    let query = use_query_map();
    let page = create_memo(move |_| {
        query
            .with(|q| q.get("page").and_then(|page| page.parse::<u32>().ok()))
            .filter(|&page| page > 0)
            .unwrap_or(1)
    });
    let filter = create_memo(move |_| {
        query.with(|q| {
            q.get("filter").and_then(|s| {
                let s = s.trim();
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            })
        })
    });
    let schema_ = schema.clone();
    let current_schema = create_memo(move |_| schema_.clone());
    let schema_id = schema.id;
    let name_singular = schema.name_singular;
    let name_plural = schema.name_plural;
    let page_size = schema.list.page_size;

    let auth = use_authorization();
    let alert = use_alerts();
    let modal = use_modals();
    let selected = create_rw_signal::<HashSet<String>>(HashSet::new());
    provide_context(selected);

    let settings = create_resource(
        move || (page(), filter()),
        move |(page, filter)| {
            let auth = auth.get();
            let schema = current_schema.get();

            async move {
                HttpRequest::get("/api/settings")
                    .with_authorization(&auth)
                    .with_parameter("page", page.to_string())
                    .with_parameter("limit", schema.list.page_size.to_string())
                    .with_parameter("prefix", schema.prefix.unwrap())
                    .with_parameter("groupby", format!(".{}", schema.suffix.unwrap_or_default()))
                    .with_optional_parameter("filter", filter)
                    .send::<List<Settings>>()
                    .await
            }
        },
    );

    let delete_action = create_action(move |items: &Arc<HashSet<String>>| {
        let items = items.clone();
        let auth = auth.get();
        let schema = current_schema.get();

        async move {
            let mut updates = Vec::with_capacity(items.len());
            for item in items.iter() {
                if !item.is_empty() {
                    if schema.suffix.is_some() {
                        updates.push(UpdateSettings::Clear {
                            prefix: format!("{}.{}.", schema.prefix.unwrap(), item),
                        });
                    } else {
                        updates.push(UpdateSettings::Delete {
                            keys: vec![format!("{}.{}", schema.prefix.unwrap(), item)],
                        });
                    }
                }
            }

            match HttpRequest::post("/api/settings")
                .with_authorization(&auth)
                .with_body(updates)
                .unwrap()
                .send::<()>()
                .await
            {
                Ok(_) => {
                    settings.refetch();
                    alert.set(Alert::success(format!(
                        "Deleted {}.",
                        maybe_plural(items.len(), name_singular, name_plural,)
                    )));
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    let total_results = create_rw_signal(None::<u32>);
    view! {
        <ListSection>
            <ListTable title=schema.list.title subtitle=schema.list.subtitle>
                <Toolbar slot>
                    <SearchBox
                        value=filter
                        on_search=move |value| {
                            use_navigate()(
                                &UrlBuilder::new(format!("/settings/{}", schema_id))
                                    .with_parameter("filter", value)
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                    <ToolbarButton
                        text=Signal::derive(move || {
                            let ns = selected.get().len();
                            if ns > 0 { format!("Delete ({ns})") } else { "Delete".to_string() }
                        })

                        color=Color::Red
                        on_click=Callback::new(move |_| {
                            let to_delete = selected.get().len();
                            if to_delete > 0 {
                                let text = maybe_plural(to_delete, name_singular, name_plural);
                                modal
                                    .set(
                                        Modal::with_title("Confirm deletion")
                                            .with_message(
                                                format!(
                                                    "Are you sure you want to delete {text}? This action cannot be undone.",
                                                ),
                                            )
                                            .with_button(format!("Delete {text}"))
                                            .with_dangerous_callback(move || {
                                                delete_action
                                                    .dispatch(
                                                        Arc::new(
                                                            selected.try_update(std::mem::take).unwrap_or_default(),
                                                        ),
                                                    );
                                            }),
                                    )
                            }
                        })
                    >

                        <IconTrash/>
                    </ToolbarButton>

                    <ToolbarButton
                        text=format!("Add {}", name_singular)
                        color=Color::Blue
                        on_click=move |_| {
                            use_navigate()(&format!("/settings/{}", schema_id), Default::default());
                        }
                    >

                        <IconAdd size=16 attr:class="flex-shrink-0 size-3"/>
                    </ToolbarButton>

                </Toolbar>

                <Transition fallback=Skeleton>
                    {move || match settings.get() {
                        None => None,
                        Some(Err(http::Error::Unauthorized)) => {
                            use_navigate()("/login", Default::default());
                            Some(view! { <div></div> }.into_view())
                        }
                        Some(Err(err)) => {
                            total_results.set(Some(0));
                            alert.set(Alert::from(err));
                            Some(view! { <Skeleton/> }.into_view())
                        }
                        Some(Ok(settings)) if !settings.items.is_empty() => {
                            total_results.set(Some(settings.total as u32));
                            let schema = current_schema.get();
                            let settings_ = settings.clone();
                            let mut headers = schema
                                .list
                                .fields
                                .iter()
                                .map(|f| f.label_column.to_string())
                                .collect::<Vec<_>>();
                            if schema.can_edit() {
                                headers.push("".to_string());
                            }
                            Some(
                                view! {
                                    <ColumnList
                                        headers=headers

                                        select_all=Callback::new(move |_| {
                                            settings_
                                                .items
                                                .iter()
                                                .filter_map(|p| p.get("_id").map(|id| id.to_string()))
                                                .collect::<Vec<_>>()
                                        })
                                    >

                                        <For
                                            each=move || settings.items.clone()
                                            key=|setting| {
                                                setting
                                                    .get("_id")
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_default()
                                            }

                                            let:settings
                                        >
                                            <SettingsItem settings schema=schema.clone()/>
                                        </For>

                                    </ColumnList>
                                }
                                    .into_view(),
                            )
                        }
                        Some(Ok(_)) => {
                            total_results.set(Some(0));
                            Some(
                                view! {
                                    <ZeroResults
                                        title="No results"
                                        subtitle="Your search did not yield any results."
                                        button_text=format!("Create a new {}", name_singular)

                                        button_action=Callback::new(move |_| {
                                            use_navigate()(
                                                &format!("/settings/{}", schema_id),
                                                Default::default(),
                                            );
                                        })
                                    />
                                }
                                    .into_view(),
                            )
                        }
                    }}

                </Transition>

                <Footer slot>

                    <Pagination
                        current_page=page
                        total_results=total_results.read_only()
                        page_size=page_size
                        on_page_change=move |page: u32| {
                            use_navigate()(
                                &UrlBuilder::new(format!("/settings/{}", schema_id))
                                    .with_parameter("page", page.to_string())
                                    .with_optional_parameter("filter", filter())
                                    .finish(),
                                Default::default(),
                            );
                        }
                    />

                </Footer>
            </ListTable>
        </ListSection>
    }
}

#[component]
fn SettingsItem(settings: Settings, schema: Arc<Schema>) -> impl IntoView {
    let columns = schema
        .list
        .fields
        .iter()
        .map(|field| {
            let value = field.display(&settings);
            view! { <ListTextItem>{value}</ListTextItem> }
        })
        .collect_view();
    let setting_id = settings
        .get("_id")
        .map(|s| s.to_string())
        .unwrap_or_default();
    let edit_link = if schema.can_edit() {
        let edit_url = format!("/settings/{}/{}", schema.id, setting_id);
        Some(view! {
            <ListItem subclass="px-6 py-1.5">
                <a
                    class="inline-flex items-center gap-x-1 text-sm text-blue-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                    href=edit_url
                >
                    Edit
                </a>
            </ListItem>
        })
    } else {
        None
    };

    view! {
        <tr>
            <ListItem>
                <label class="flex">
                    <SelectItem item_id=setting_id/>

                    <span class="sr-only">Checkbox</span>
                </label>
            </ListItem>
            {columns}
            {edit_link}

        </tr>
    }
}
