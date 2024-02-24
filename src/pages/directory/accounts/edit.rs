use std::vec;

use humansize::{format_size, DECIMAL};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use pwhash::sha512_crypt;

use crate::{
    components::{
        form::{
            button::Button,
            input::{InputPassword, InputSize, InputText},
            select::Select,
            stacked_badge::StackedBadge,
            stacked_input::StackedInput,
            value_is_email, value_is_not_empty, value_lowercase, value_remove_spaces, value_trim,
            ButtonBar, Form, FormItem, FormListValidator, FormValidator, StringValidateFn,
            ValidateCb,
        },
        messages::alert::{use_alerts, Alert},
        skeleton::Skeleton,
        Color,
    },
    core::{
        http::{self, HttpRequest},
        oauth::use_authorization,
    },
    pages::directory::{Principal, Type},
};

#[component]
pub fn AccountEdit() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();
    let params = use_params_map();
    let fetch_principal = create_resource(
        move || params().get("id").cloned().unwrap_or_default(),
        move |name| {
            let auth = auth.get();

            async move {
                if !name.is_empty() {
                    HttpRequest::get(format!("https://127.0.0.1/api/principal/{name}"))
                        .with_authorization(&auth)
                        .send::<Principal>()
                        .await
                } else {
                    Ok(Principal::default())
                }
            }
        },
    );
    let (pending, set_pending) = create_signal(false);

    let current_principal = create_rw_signal(Principal::default());
    let login = FormValidator::new(String::new());
    let name = FormValidator::new(String::new());
    let typ: RwSignal<Type> = create_rw_signal(Type::Individual);
    let password = FormValidator::new(String::new());
    let quota = create_rw_signal(0u32);
    let member_of = create_rw_signal(Vec::<String>::new());
    let email = FormValidator::new(String::new());
    let aliases = FormListValidator::new(Vec::<String>::new());
    let default_type = Type::Individual;

    let validate = move || {
        let aliases = aliases.validate([value_trim, value_lowercase], [value_is_email])?;

        Some(Principal {
            typ: typ.get().into(),
            quota: quota.get().into(),
            name: login
                .validate([value_remove_spaces, value_lowercase], [value_is_not_empty])?
                .into(),
            secrets: {
                let password = password.get().ok()?;
                if !password.is_empty() {
                    vec![sha512_crypt::hash(password).unwrap()]
                } else {
                    vec![]
                }
            },
            emails: [email.validate([value_trim, value_lowercase], [value_is_email])?]
                .into_iter()
                .chain(aliases)
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>(),
            member_of: member_of.get(),
            members: vec![],
            description: name
                .validate::<_, [_; 0], _, StringValidateFn>([value_trim], [])?
                .into(),
            ..Default::default()
        })
    };
    let principal_is_valid = create_action(move |(name, cb): &(String, ValidateCb)| {
        let name = name.clone();
        let auth = auth.get();
        let cb = *cb;

        async move {
            let result = match HttpRequest::get(format!("https://127.0.0.1/api/principal/{name}"))
                .with_authorization(&auth)
                .send::<Principal>()
                .await
            {
                Ok(principal) if principal.typ == Some(Type::Group) => Ok(name),
                Ok(_) => Err("Principal is not a group".to_string()),
                Err(http::Error::NotFound) => Err("Group does not exist".to_string()),
                Err(http::Error::Unauthorized) => {
                    use_navigate()("/login", Default::default());
                    Err("Unauthorized".to_string())
                }
                Err(err) => Err(format!("Request failed: {err:?}")),
            };

            cb.call(result);
        }
    });
    let save_changes = create_action(move |changes: &Principal| {
        let current = current_principal.get();
        let changes = changes.clone();
        let auth = auth.get();

        async move {
            set_pending.set(true);
            let result = if !current.is_blank() {
                let name = current.name.clone().unwrap_or_default();
                let updates = current.into_updates(changes);

                if !updates.is_empty() {
                    log::debug!("Updating principal: {updates:?}");
                    HttpRequest::patch(format!("https://127.0.0.1/api/principal/{name}"))
                        .with_authorization(&auth)
                        .with_body(updates)
                        .unwrap()
                        .send::<()>()
                        .await
                } else {
                    Ok(())
                }
            } else {
                HttpRequest::post("https://127.0.0.1/api/principal")
                    .with_authorization(&auth)
                    .with_body(changes)
                    .unwrap()
                    .send::<u32>()
                    .await
                    .map(|_| ())
            };
            set_pending.set(false);
            log::debug!("Result: {result:?}");

            match result {
                Ok(_) => {
                    use_navigate()("/manage/accounts", Default::default());
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="Account" subtitle="Manage your name, password and account settings.">

            <Transition fallback=Skeleton set_pending>

                {move || match fetch_principal.get() {
                    None => None,
                    Some(Err(http::Error::Unauthorized)) => {
                        use_navigate()("/login", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(http::Error::NotFound)) => {
                        use_navigate()("/manage/accounts", Default::default());
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Err(err)) => {
                        alert.set(Alert::from(err));
                        Some(view! { <div></div> }.into_view())
                    }
                    Some(Ok(principal)) => {
                        login.update(principal.name.clone().unwrap_or_default());
                        typ.set(principal.typ.unwrap_or(default_type));
                        name.update(principal.description.clone().unwrap_or_default());
                        password.update(String::new());
                        quota.set(principal.quota.unwrap_or_default());
                        member_of.set(principal.member_of.clone());
                        email.update(principal.emails.first().cloned().unwrap_or_default());
                        aliases
                            .update(principal.emails.iter().skip(1).cloned().collect::<Vec<_>>());
                        let used_quota = principal.used_quota.unwrap_or_default();
                        let total_quota = principal.quota.unwrap_or_default();
                        current_principal.set(principal);
                        Some(
                            view! {
                                <FormItem label="Login">
                                    <InputText placeholder="Login name" value=login/>
                                </FormItem>

                                <FormItem label="Name">
                                    <InputText placeholder="Full name" value=name/>
                                </FormItem>

                                <FormItem label="Type">
                                    <Select
                                        value=typ
                                        options=vec![Type::Individual, Type::Superuser]
                                    />

                                </FormItem>

                                <FormItem label="Password">
                                    <InputPassword value=password/>
                                </FormItem>

                                <FormItem label="Email">
                                    <InputText placeholder="user@example.org" value=email/>
                                </FormItem>

                                <FormItem label="Aliases">
                                    <StackedInput
                                        values=aliases
                                        placeholder="Email"
                                        add_button_text="Add Email".to_string()
                                    />
                                </FormItem>

                                <FormItem label="Disk quota">
                                    <div class="relative">
                                        <InputSize value=quota/>
                                        <Show when=move || { total_quota > 0 }>
                                            <p class="mt-3">
                                                <label class="inline-flex items-center gap-x-1 text-xs text-black-600 decoration-2 hover:underline font-medium dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600">

                                                    {format!(
                                                        "{} used ({:.1}%)",
                                                        format_size(used_quota, DECIMAL),
                                                        (used_quota as f64 / total_quota as f64) * 100.0,
                                                    )}

                                                </label>
                                            </p>

                                        </Show>

                                    </div>
                                </FormItem>

                                <FormItem label="Member of">
                                    <StackedBadge
                                        values=member_of

                                        add_button_text="Add to group".to_string()
                                        validate_item=Callback::new(move |value| {
                                            principal_is_valid.dispatch(value);
                                        })
                                    />

                                </FormItem>
                            }
                                .into_view(),
                        )
                    }
                }}

            </Transition>

            <ButtonBar slot>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=|_| {
                        use_navigate()("/manage/accounts", Default::default());
                    }
                />

                <Button
                    text="Save changes"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        if let Some(changes) = validate() {
                            save_changes.dispatch(changes);
                        }
                    })

                    disabled=pending
                />
            </ButtonBar>

        </Form>
    }
}
