use leptos::*;

#[component]
pub fn Table(#[prop(into)] headers: MaybeSignal<Vec<String>>, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-white">
            <div class="-m-1.5 overflow-x-auto">
                <div class="p-1.5 min-w-full inline-block align-middle">
                    <div class="border rounded-lg shadow overflow-hidden dark:border-gray-700 dark:shadow-gray-900">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            <thead>
                                <tr>
                                    <For
                                        each=move || {
                                            headers.get().into_iter().collect::<Vec<_>>()
                                        }

                                        key=|header| header.clone()
                                        children=move |header| {
                                            view! {
                                                <th
                                                    scope="col"
                                                    class="px-6 py-3 text-start text-xs font-medium text-gray-500 uppercase"
                                                >
                                                    {header}
                                                </th>
                                            }
                                        }
                                    />

                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                {children()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TableRow(children: Children) -> impl IntoView {
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            view! {
                <td class="px-6 py-2 whitespace-nowrap text-sm text-gray-800 dark:text-gray-200">
                    {child}
                </td>
            }
        })
        .collect_view();

    view! { <tr>{children}</tr> }
}
