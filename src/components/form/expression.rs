use leptos::*;

use crate::{
    components::icon::{IconArrowRightCircle, IconPlus, IconVariable, IconXMark},
    core::{
        form::{ExpressionError, FormErrorType},
        schema::Validator,
    },
};

use super::FormElement;

#[component]
pub fn InputExpression(
    element: FormElement,
    #[prop(optional, into)] placeholder: Option<MaybeSignal<String>>,
) -> impl IntoView {
    let if_thens = create_memo(move |_| {
        let data = element.data.get();
        let error = data.error(element.id);

        data.expr_if_thens(element.id)
            .enumerate()
            .map(|(idx, expr)| {
                (
                    idx,
                    expr.clone(),
                    error.as_ref().and_then(|e| match e.id {
                        FormErrorType::Expression(ExpressionError::If(pos)) if pos == idx => {
                            Some(ExpressionError::If(e.error.clone()))
                        }
                        FormErrorType::Expression(ExpressionError::Then(pos)) if pos == idx => {
                            Some(ExpressionError::Then(e.error.clone()))
                        }
                        _ => None,
                    }),
                )
            })
            .collect::<Vec<_>>()
    });
    let else_value = create_memo(move |_| {
        element
            .data
            .get()
            .expr_else(element.id)
            .unwrap_or_default()
            .to_string()
    });
    let else_err = create_memo(move |_| {
        element
            .data
            .get()
            .error(element.id)
            .as_ref()
            .and_then(|e| match e.id {
                FormErrorType::Expression(ExpressionError::Else) => Some(e.error.clone()),
                _ => None,
            })
    });
    let disable_add = element
        .data
        .get_untracked()
        .schema
        .fields
        .get(element.id)
        .unwrap()
        .checks
        .default
        .as_ref()
        .map_or(false, |checks| {
            checks.validators.contains(&Validator::MaxItems(1))
        });

    view! {
        <div class="space-y-3">

            <For
                each=move || { if_thens.get().into_iter() }
                key=move |(idx, if_then, error)| {
                    format!("{idx}_{}_{}", if_then.hash(), error.is_some())
                }

                children=move |(idx, if_then, error)| {
                    let (is_if_err, is_then_err, error) = match error {
                        Some(ExpressionError::If(err)) => (true, false, err.into()),
                        Some(ExpressionError::Then(err)) => (false, true, err.into()),
                        _ => (false, false, None),
                    };
                    let ok_class = "py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm -mt-px -ms-px first:rounded-t-lg last:rounded-b-lg sm:first:rounded-s-lg sm:mt-0 sm:first:ms-0 sm:first:rounded-se-none sm:last:rounded-es-none sm:last:rounded-e-lg text-sm relative focus:z-10 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600";
                    let err_class = "py-2 px-3 pe-11 block w-full border-red-500 shadow-sm -mt-px -ms-px first:rounded-t-lg last:rounded-b-lg sm:first:rounded-s-lg sm:mt-0 sm:first:ms-0 sm:first:rounded-se-none sm:last:rounded-es-none sm:last:rounded-e-lg text-sm relative focus:z-10 focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600";
                    view! {
                        <div class="space-y-3">
                            <div class="relative">
                                <div class="sm:flex rounded-lg shadow-sm">
                                    <input
                                        type="text"
                                        placeholder="if"
                                        prop:value=if_then.if_
                                        class=move || {
                                            if !is_if_err { ok_class } else { err_class }
                                        }

                                        on:change=move |ev| {
                                            element
                                                .data
                                                .update(|data| {
                                                    data.expr_update_if(
                                                        element.id,
                                                        idx,
                                                        event_target_value(&ev),
                                                    );
                                                });
                                        }
                                    />

                                    <span class="py-2 px-3 inline-flex items-center min-w-fit w-full border border-gray-200 bg-gray-50 text-sm text-gray-500 -mt-px -ms-px first:rounded-t-lg last:rounded-b-lg sm:w-auto sm:first:rounded-s-lg sm:mt-0 sm:first:ms-0 sm:first:rounded-se-none sm:last:rounded-es-none sm:last:rounded-e-lg dark:bg-gray-700 dark:border-gray-700 dark:text-gray-400">
                                        <IconArrowRightCircle attr:class="mx-auto size-4 text-gray-400"/>
                                    </span>
                                    <input
                                        type="text"
                                        placeholder="then"
                                        class=move || {
                                            if !is_then_err { ok_class } else { err_class }
                                        }

                                        prop:value=if_then.then_

                                        on:change=move |ev| {
                                            element
                                                .data
                                                .update(|data| {
                                                    data.expr_update_then(
                                                        element.id,
                                                        idx,
                                                        event_target_value(&ev),
                                                    );
                                                });
                                        }
                                    />

                                    <button
                                        type="button"
                                        class="absolute top-0 end-0 p-2.5 rounded-e-md dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                                        on:click=move |_| {
                                            element
                                                .data
                                                .update(|data| {
                                                    data.expr_delete_if_then(element.id, idx);
                                                });
                                        }
                                    >

                                        <IconXMark/>
                                    </button>
                                </div>

                            </div>
                            {error
                                .map(|error| {
                                    view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                                })}

                        </div>
                    }
                }
            />

            <div class="space-y-3">
                <div class="relative">
                    <input
                        type="text"
                        class=move || {
                            if else_err.get().is_none() {
                                "py-2 px-3 pe-11 block w-full border-gray-200 shadow-sm text-sm rounded-lg focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            } else {
                                "py-2 px-3 pe-11 block w-full border-red-500 shadow-sm text-sm rounded-lg focus:border-red-500 focus:ring-red-500 disabled:opacity-50 disabled:pointer-events-none dark:bg-slate-900 dark:border-gray-700 dark:text-gray-400 dark:focus:ring-gray-600"
                            }
                        }

                        placeholder=placeholder.clone().map(|p| move || p.get())
                        prop:value=else_value

                        on:change=move |ev| {
                            element
                                .data
                                .update(|data| {
                                    data.expr_update_else(element.id, event_target_value(&ev));
                                });
                        }
                    />

                    <div class="absolute inset-y-0 end-0 flex items-center pointer-events-none z-20 pe-4">
                        <IconVariable attr:class="flex-shrink-0 size-4 text-gray-400"/>
                    </div>
                </div>

                {move || {
                    else_err
                        .get()
                        .map(|error| {
                            view! { <p class="text-xs text-red-600 mt-2">{error}</p> }
                        })
                }}

            </div>
        </div>

        <p class="mt-3 text-end" class:hidden=disable_add>
            <button
                type="button"
                class="py-1.5 px-2 inline-flex items-center gap-x-1 text-xs font-medium rounded-full border border-dashed border-gray-200 bg-white text-gray-800 hover:bg-gray-50 disabled:opacity-50 disabled:pointer-events-none dark:bg-gray-800 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                on:click=move |_| {
                    if if_thens
                        .get()
                        .last()
                        .map_or(true, |(_, v, _)| !v.if_.is_empty() && !v.then_.is_empty())
                    {
                        element
                            .data
                            .update(|data| {
                                data.expr_push_if_then(element.id, "", "");
                            });
                    }
                }
            >

                <IconPlus attr:class="flex-shrink-0 size-3.5"/>
                Add Condition
            </button>
        </p>
    }
}
