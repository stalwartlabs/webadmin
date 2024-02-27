pub mod button;
pub mod input;
pub mod select;
pub mod stacked_badge;
pub mod stacked_input;

use std::hash::{DefaultHasher, Hash, Hasher};

use leptos::*;

use crate::components::messages::alert::Alerts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValue<T: Default + Clone + 'static> {
    Ok(T),
    Err { value: T, reason: String },
}

#[derive(Clone, Default)]
pub struct FormValidator<T: Default + Clone + 'static> {
    signal: RwSignal<FormValue<T>>,
}

#[derive(Clone, Default)]
pub struct FormListValidator<T: Default + Clone + 'static> {
    signal: RwSignal<Vec<FormValue<T>>>,
}

#[slot]
pub struct ButtonBar {
    children: ChildrenFn,
}

#[component]
pub fn Form(
    #[prop(into)] title: MaybeSignal<String>,
    #[prop(into)] subtitle: MaybeSignal<String>,
    children: Children,
    button_bar: ButtonBar,
    #[prop(optional, into)] single_column: bool,
) -> impl IntoView {
    let class = if !single_column {
        "grid sm:grid-cols-12 gap-2 sm:gap-6"
    } else {
        "space-y-4 sm:space-y-6"
    };

    view! {
        <div class="max-w-4xl px-4 py-10 sm:px-6 lg:px-8 lg:py-14 mx-auto">
            <div class="bg-white rounded-xl shadow p-4 sm:p-7 dark:bg-slate-900">
                <div class="mb-8">
                    <h2 class="text-xl font-bold text-gray-800 dark:text-gray-200">
                        {title.get()}
                    </h2>
                    <p class="text-sm text-gray-600 dark:text-gray-400">{subtitle.get()}</p>
                </div>

                <Alerts/>

                <form>

                    <div class=class>{children()}</div>
                    <div class="mt-5 flex justify-end gap-x-2">{(button_bar.children)()}</div>
                </form>
            </div>
        </div>
    }
}

#[component]
pub fn FormItem(
    #[prop(into)] label: MaybeSignal<String>,
    children: Children,
    #[prop(optional, into)] single_column: bool,
) -> impl IntoView {
    if !single_column {
        view! {
            <div class="sm:col-span-3">
                <label
                    for="af-account-email"
                    class="inline-block text-sm text-gray-800 mt-2.5 dark:text-gray-200"
                >
                    {label}
                </label>
            </div>
            <div class="sm:col-span-9">{children()}</div>
        }
        .into_view()
    } else {
        view! {
            <div class="space-y-2">
                <label class="inline-block text-sm font-bold text-gray-800 mt-2.5 dark:text-gray-200">
                    {label}
                </label>

                {children()}
            </div>
        }.into_view()
    }
}

impl<T: Default + Clone + PartialEq + Hash> FormValue<T> {
    pub fn signal(value: T) -> RwSignal<Self> {
        create_rw_signal(FormValue::Ok(value))
    }

    pub fn err(reason: impl Into<String>) -> FormValue<T> {
        FormValue::Err {
            value: Default::default(),
            reason: reason.into(),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            FormValue::Ok(v) => v,
            FormValue::Err { value, .. } => value,
        }
    }

    pub fn unwrap_err(self) -> String {
        match self {
            FormValue::Err { reason, .. } => reason,
            FormValue::Ok(_) => String::new(),
        }
    }

    pub fn get(&self) -> &T {
        match self {
            FormValue::Ok(ref value) | FormValue::Err { ref value, .. } => value,
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        match self {
            FormValue::Ok(ref mut value) | FormValue::Err { ref mut value, .. } => value,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, FormValue::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, FormValue::Err { .. })
    }

    pub fn is_err_value(&self, value: &T) -> bool {
        matches!(self, FormValue::Err { value: v, .. } if v == value)
    }

    pub fn ok(self) -> Option<T> {
        match self {
            FormValue::Ok(value) => Some(value),
            FormValue::Err { .. } => None,
        }
    }

    pub fn into_error(self, reason: String) -> FormValue<T> {
        match self {
            FormValue::Ok(value) => FormValue::Err { value, reason },
            FormValue::Err { value, reason: _ } => FormValue::Err { value, reason },
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        match self {
            FormValue::Ok(value) => {
                value.hash(&mut hasher);
            }
            FormValue::Err { value, reason } => {
                value.hash(&mut hasher);
                reason.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

impl<T: Default + Clone + PartialEq + Hash + 'static> FormValidator<T> {
    pub fn new(value: T) -> Self {
        FormValidator {
            signal: FormValue::signal(value),
        }
    }

    pub fn validate<S, V, SI, VI>(&self, sanitizers: S, validators: V) -> Option<T>
    where
        S: IntoIterator<Item = SI>,
        V: IntoIterator<Item = VI>,
        SI: Into<Callback<T, T>>,
        VI: Into<Callback<T, Option<String>>>,
    {
        let mut value = self.signal.get().ok()?;
        for sanitizer in sanitizers {
            value = sanitizer.into().call(value);
        }
        for validator in validators {
            if let Some(reason) = validator.into().call(value.clone()) {
                self.signal.set(FormValue::Err { value, reason });
                return None;
            }
        }

        Some(value)
    }

    pub fn signal(&self) -> RwSignal<FormValue<T>> {
        self.signal
    }

    pub fn update(&self, value: T) {
        self.signal.set(FormValue::Ok(value));
    }
}

impl<T: Default + Clone + PartialEq + Hash + 'static> FormListValidator<T> {
    pub fn new(value: Vec<T>) -> Self {
        FormListValidator {
            signal: create_rw_signal(value.into_iter().map(FormValue::Ok).collect()),
        }
    }

    pub fn validate<S, V, SI, VI>(&self, sanitizers: S, validators: V) -> Option<Vec<T>>
    where
        S: IntoIterator<Item = SI>,
        V: IntoIterator<Item = VI>,
        SI: Into<Callback<T, T>>,
        VI: Into<Callback<T, Option<String>>>,
    {
        let values = self.signal.get();
        let mut result = Vec::with_capacity(values.len());
        let sanitizers: Vec<Callback<T, T>> = sanitizers.into_iter().map(|s| s.into()).collect();
        let validators: Vec<Callback<T, Option<String>>> =
            validators.into_iter().map(|v| v.into()).collect();

        for (idx, value) in values.into_iter().enumerate() {
            let mut value = value.ok()?;

            for sanitizer in &sanitizers {
                value = sanitizer.call(value);
            }
            for validator in &validators {
                if let Some(reason) = validator.call(value.clone()) {
                    self.signal.update(|v| {
                        v[idx] = FormValue::Err { value, reason };
                    });
                    return None;
                }
            }
            result.push(value);
        }

        Some(result)
    }

    pub fn signal(&self) -> RwSignal<Vec<FormValue<T>>> {
        self.signal
    }

    pub fn update(&self, value: Vec<T>) {
        self.signal
            .set(value.into_iter().map(FormValue::Ok).collect());
    }

    pub fn get(&self) -> Vec<FormValue<T>> {
        self.signal.get()
    }
}

impl<T: Default + Clone> From<T> for FormValue<T> {
    fn from(value: T) -> Self {
        FormValue::Ok(value)
    }
}

impl<T: Default + Clone> Default for FormValue<T> {
    fn default() -> Self {
        FormValue::Ok(Default::default())
    }
}

pub type StringValidateFn = Callback<String, Option<String>>;
pub type ValidateCb = Callback<Result<String, String>, ()>;

pub fn value_trim(value: String) -> String {
    value.trim().to_string()
}

pub fn value_remove_spaces(value: String) -> String {
    value.replace(' ', "")
}

pub fn value_lowercase(value: String) -> String {
    value.to_lowercase()
}

pub fn value_is_not_empty(value: String) -> Option<String> {
    if value.is_empty() {
        Some("This field is required".to_string())
    } else {
        None
    }
}

pub fn value_is_email(value: String) -> Option<String> {
    if value.is_empty()
        || value.split_once('@').map_or(false, |(_, domain)| {
            domain.contains('.') && !domain.contains('@')
        })
    {
        None
    } else {
        Some("This is not a valid email address".to_string())
    }
}

impl<T: Default + Clone + 'static> Copy for FormValidator<T> {}

impl<T: Default + Clone + 'static> Copy for FormListValidator<T> {}
