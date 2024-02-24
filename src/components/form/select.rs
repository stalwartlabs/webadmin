use leptos::*;

pub trait SelectOption: Clone + PartialEq + Eq + Default + 'static {
    fn label(&self) -> String;
    fn value(&self) -> String;
}

#[component]
pub fn Select<T: SelectOption>(
    #[prop(optional, into)] value: RwSignal<T>,
    #[prop(optional, into)] options: MaybeSignal<Vec<T>>,
) -> impl IntoView {
    let ev_options = options.clone();

    view! {
        <select
            class="py-2 px-3 pe-9 block w-full border-gray-200 rounded-lg text-sm focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
            on:change=move |ev| {
                let selection = event_target_value(&ev);
                value
                    .set(ev_options.get().iter().find(|o| o.value() == selection).unwrap().clone());
            }
        >

            <For
                each=move || options.get()
                key=move |item| item.value()
                children=move |item| {
                    let option_value = item.value();
                    let label = item.label();
                    let is_selected = value.get() == item;
                    view! {
                        <option selected=is_selected value=option_value>
                            {label}
                        </option>
                    }
                }
            />

        </select>
    }
}

impl SelectOption for String {
    fn label(&self) -> String {
        self.clone()
    }

    fn value(&self) -> String {
        self.clone()
    }
}

impl SelectOption for &'static str {
    fn label(&self) -> String {
        self.to_string()
    }

    fn value(&self) -> String {
        self.to_string()
    }
}
