use leptos::*;

use crate::core::schema::{Source, Type};

use super::FormElement;

#[component]
pub fn Select(element: FormElement) -> impl IntoView {
    let options = match &element
        .data
        .get_untracked()
        .schema
        .fields
        .get(element.id)
        .unwrap()
        .typ_
    {
        Type::Select(Source::Static(options)) => options
            .into_iter()
            .map(|(value, label)| (value.to_string(), label.to_string()))
            .collect::<Vec<_>>(),
        Type::Select(Source::Dynamic { schema, field }) => {
            todo!()
        }
        _ => panic!("Invalid schema type for select"),
    };
    let value = create_memo(move |_| {
        element
            .data
            .get()
            .value::<String>(element.id)
            .unwrap_or_default()
    });

    view! {
        <select
            class="py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
            on:change=move |ev| {
                element
                    .data
                    .update(|data| {
                        data.set(element.id, event_target_value(&ev));
                    });
            }
        >

            <For
                each=move || options.clone()
                key=move |(id, _)| id.clone()
                children=move |(id, label)| {
                    let id_ = id.clone();
                    view! {
                        <option selected=move || value.get() == id value=id_>
                            {label}
                        </option>
                    }
                }
            />

        </select>
    }
}
