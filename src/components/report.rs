use leptos::*;

#[component]
pub fn ReportView(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(into, optional)] hide: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class="max-w-[85rem] px-4 py-5 sm:px-6 lg:px-8 lg:py-7 mx-auto"
            class:hidden=move || hide.get()
        >
            <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">

                {children()}

            </div>
        </div>
    }
}

#[component]
pub fn ReportSection(
    #[prop(into)] title: String,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class="grid sm:grid-cols-12 gap-2 sm:gap-4 py-8 first:pt-0 last:pb-0 border-t first:border-transparent border-gray-200 dark:border-gray-700 dark:first:border-transparent"
        >

            <div class="sm:col-span-12">
                <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">{title}</h2>
            </div>

            {children()}
        </div>
    }
}

#[component]
pub fn ReportItem(
    #[prop(into)] label: String,
    #[prop(optional)] hide: bool,
    children: Children,
) -> impl IntoView {
    if !hide {
        Some(view! {
            <div class="sm:col-span-3">
                <label class="inline-block text-sm font-medium text-gray-500 mt-2.5">{label}</label>
            </div>

            <div class="sm:col-span-9">{children()}</div>
        }.into_view())
    } else {
        None
    }
}

#[component]
pub fn ReportTextValue(#[prop(into)] value: MaybeSignal<String>) -> impl IntoView {
    view! {
        <label class="inline-block text-sm font-semibold text-gray-500 mt-2.5">
            {move || value.get()}
        </label>
    }
}
