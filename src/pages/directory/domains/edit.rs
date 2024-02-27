use leptos::*;
use leptos_router::use_navigate;

use crate::{
    components::{
        form::{
            button::Button, input::InputText, value_lowercase, value_trim, ButtonBar, Form,
            FormItem, FormValidator,
        },
        messages::alert::{use_alerts, Alert},
        Color,
    },
    core::{http::HttpRequest, oauth::use_authorization},
};

#[component]
pub fn DomainCreate() -> impl IntoView {
    let auth = use_authorization();
    let alert = use_alerts();

    let (pending, set_pending) = create_signal(false);

    let name = FormValidator::new(String::new());

    let save_changes = create_action(move |name: &String| {
        let auth = auth.get();
        let name = name.clone();

        async move {
            set_pending.set(true);
            let result = HttpRequest::post(format!("https://127.0.0.1/api/domain/{name}"))
                .with_authorization(&auth)
                .send::<()>()
                .await
                .map(|_| ());
            set_pending.set(false);

            match result {
                Ok(_) => {
                    use_navigate()("/manage/directory/domains", Default::default());
                }
                Err(err) => {
                    alert.set(Alert::from(err));
                }
            }
        }
    });

    view! {
        <Form title="Create domain" subtitle="Create a new local domain name">

            <FormItem label="Domain name">
                <InputText
                    placeholder="example.org"

                    value=name
                />
            </FormItem>

            <ButtonBar slot>
                <Button
                    text="Cancel"
                    color=Color::Gray
                    on_click=move |_| {
                        use_navigate()("/manage/directory/domains", Default::default());
                    }
                />

                <Button
                    text="Create"
                    color=Color::Blue
                    on_click=Callback::new(move |_| {
                        if let Some(domain_name) = name
                            .validate([value_trim, value_lowercase], [value_is_domain])
                        {
                            save_changes.dispatch(domain_name);
                        }
                    })

                    disabled=pending
                />
            </ButtonBar>

        </Form>
    }
}

pub fn value_is_domain(value: String) -> Option<String> {
    if value.is_empty() || !value.contains('.') || value.starts_with('.') || value.ends_with('.') {
        Some("Invalid domain name".to_string())
    } else {
        None
    }
}
