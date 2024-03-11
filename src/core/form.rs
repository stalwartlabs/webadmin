use std::str::FromStr;
use std::sync::Arc;

use ahash::AHashMap;
use leptos::RwSignal;

use super::schema::Type;

use super::schema::{InputCheck, Schema, Transformer, Validator};

#[derive(Clone, PartialEq, Eq, Default)]
pub struct FormData {
    pub values: AHashMap<String, FormValue>,
    pub errors: AHashMap<String, FormError>,
    pub schema: Arc<Schema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValue {
    Value(String),
    Map(AHashMap<String, String>),
    Array(Vec<FormValue>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormError {
    pub id: FormErrorId,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormErrorId {
    Id(String),
    Index(usize),
    None,
}

impl FormData {
    pub fn with_value(mut self, id: impl Into<String>, value: impl Into<FormValue>) -> Self {
        self.values.insert(id.into(), value.into());
        self
    }

    pub fn set(&mut self, id: impl Into<String>, value: impl Into<FormValue>) {
        self.values.insert(id.into(), value.into());
    }

    pub fn value<T: FromStr>(&self, id: &str) -> Option<T> {
        self.values.get(id).and_then(|v| match v {
            FormValue::Value(v) => T::from_str(v.as_str()).ok(),
            _ => None,
        })
    }

    pub fn has_value(&self, id: &str) -> bool {
        self.values.contains_key(id)
    }

    pub fn update(&mut self, id: &str, value: impl Into<FormValue>) {
        let value = value.into();
        log::debug!("Updating form value: {id} {value:?}");
        self.values.insert(id.to_string(), value);
        self.errors.remove(id);
    }

    pub fn remove(&mut self, id: &str) {
        self.values.remove(id);
        self.errors.remove(id);
    }

    pub fn array_value<'x>(&'x self, id: &str) -> Box<dyn Iterator<Item = &'x str> + 'x> {
        match self.values.get(id) {
            Some(FormValue::Array(values)) => Box::new(values.iter().filter_map(|v| match v {
                FormValue::Value(v) => Some(v.as_str()),
                _ => None,
            })),
            Some(FormValue::Value(v)) => Box::new(std::iter::once(v.as_str())),
            _ => Box::new([].into_iter()),
        }
    }

    pub fn array_set(&mut self, id: &str, values: impl IntoIterator<Item = impl Into<FormValue>>) {
        self.values.insert(
            id.to_string(),
            FormValue::Array(values.into_iter().map(Into::into).collect()),
        );
        self.errors.remove(id);
    }

    pub fn array_update(&mut self, id: &str, idx: usize, value: impl Into<FormValue>) {
        match self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Array(vec![]))
        {
            FormValue::Array(values) => {
                if let Some(v) = values.get_mut(idx) {
                    *v = value.into();
                }
            }
            v @ FormValue::Value(_) if idx == 0 => {
                *v = value.into();
            }
            _ => (),
        }
        self.errors.remove(id);
    }

    pub fn array_delete(&mut self, id: &str, idx: usize) {
        let left = self.values.get_mut(id).and_then(|v| match v {
            FormValue::Array(values) => {
                values.remove(idx);
                Some(values.len())
            }
            FormValue::Value(_) if idx == 0 => Some(0),
            _ => None,
        });
        if left == Some(0) {
            self.values.remove(id);
        }
        self.errors.remove(id);
    }

    pub fn array_push(&mut self, id: &str, value: impl Into<FormValue>) {
        let v = self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Array(vec![]));

        match v {
            FormValue::Value(val) => {
                *v = FormValue::Array(vec![FormValue::Value(std::mem::take(val)), value.into()]);
            }
            FormValue::Array(arr) => {
                arr.push(value.into());
            }
            _ => unreachable!(),
        };
        self.errors.remove(id);
    }

    pub fn error(&self, id: &str) -> Option<&FormError> {
        self.errors.get(id)
    }

    pub fn error_string(&self, id: &str) -> Option<&str> {
        self.error(id).map(|e| e.error.as_str())
    }

    pub fn has_errors(&self, id: &str) -> bool {
        self.errors.contains_key(id)
    }

    pub fn validate_form(&mut self) -> bool {
        if !self.errors.is_empty() {
            return false;
        }

        let schema = self.schema.clone();
        for field in schema.fields.values() {
            if let Some(check) = field.input_check(self) {
                match field.typ_ {
                    Type::Input | Type::Secret | Type::Text | Type::Select(_) => {
                        match check.check_value(self.value::<String>(field.id).unwrap_or_default())
                        {
                            Ok(value) => {
                                if !value.is_empty() {
                                    self.update(field.id, value);
                                } else {
                                    self.remove(field.id);
                                }
                            }
                            Err(err) => {
                                self.errors.insert(
                                    field.id.to_string(),
                                    FormError {
                                        id: FormErrorId::None,
                                        error: err.to_string(),
                                    },
                                );
                            }
                        }
                    }
                    Type::InputMulti => {
                        let mut total_values = 0;
                        for (idx, result) in self
                            .array_value(field.id)
                            .map(|v| check.check_value(v.to_string()))
                            .enumerate()
                            .collect::<Vec<_>>()
                        {
                            match result {
                                Ok(value) => {
                                    if !value.is_empty() {
                                        self.array_update(field.id, idx, value);
                                        total_values += 1;
                                    } else {
                                        self.array_delete(field.id, idx);
                                    }
                                }
                                Err(err) => {
                                    self.errors.insert(
                                        field.id.to_string(),
                                        FormError {
                                            id: FormErrorId::Index(idx),
                                            error: err.to_string(),
                                        },
                                    );
                                }
                            }
                        }

                        for validator in &check.validators {
                            match validator {
                                Validator::MinItems(min) => {
                                    if total_values < *min {
                                        self.errors.insert(
                                            field.id.to_string(),
                                            FormError {
                                                id: FormErrorId::None,
                                                error: format!(
                                                    "At least {} items are required",
                                                    min
                                                ),
                                            },
                                        );
                                    }
                                }
                                Validator::MaxItems(max) => {
                                    if total_values > *max {
                                        self.errors.insert(
                                            field.id.to_string(),
                                            FormError {
                                                id: FormErrorId::None,
                                                error: format!("At most {} items are allowed", max),
                                            },
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    Type::Expression => todo!(),
                    Type::Checkbox | Type::Duration => todo!(),
                }
            }
        }

        self.errors.is_empty()
    }

    pub fn into_signal(self) -> RwSignal<Self> {
        RwSignal::new(self)
    }
}

impl InputCheck {
    pub fn check_value(&self, mut value: String) -> Result<String, &'static str> {
        for transformer in &self.transformers {
            value = match transformer {
                Transformer::Trim => value.trim().to_string(),
                Transformer::RemoveSpaces => value.replace(' ', ""),
                Transformer::Lowercase => value.to_lowercase(),
            };
        }

        if !value.is_empty() {
            for validator in &self.validators {
                match validator {
                    Validator::IsEmail => {
                        if !value.contains('@') {
                            return Err("This field must be a valid email address");
                        }
                    }
                    Validator::IsCron => todo!(),
                    Validator::IsId => todo!(),
                    Validator::IsHost => todo!(),
                    Validator::IsPort => todo!(),
                    Validator::IsUrl => {
                        if !value.contains("://") {
                            return Err("This field must be a valid URL");
                        }
                    }
                    Validator::IsDomain => {
                        if !value.contains('.') || value.starts_with('.') || value.ends_with('.') {
                            return Err("This field must be a valid domain name");
                        }
                    }
                    Validator::IsGlobPattern => todo!(),
                    Validator::IsRegexPattern => todo!(),
                    Validator::MinLength(_) => todo!(),
                    Validator::MaxLength(_) => todo!(),
                    Validator::MinValue(_) => todo!(),
                    Validator::MaxValue(_) => todo!(),
                    Validator::IsValidExpression { .. }
                    | Validator::MinItems(_)
                    | Validator::MaxItems(_)
                    | Validator::Required => (),
                }
            }
        } else if self.validators.contains(&Validator::Required) {
            return Err("This field is required");
        }

        Ok(value)
    }
}

impl From<String> for FormValue {
    fn from(value: String) -> Self {
        FormValue::Value(value)
    }
}

impl From<&String> for FormValue {
    fn from(value: &String) -> Self {
        FormValue::Value(value.to_string())
    }
}

impl From<&str> for FormValue {
    fn from(value: &str) -> Self {
        FormValue::Value(value.to_string())
    }
}

impl From<Vec<String>> for FormValue {
    fn from(value: Vec<String>) -> Self {
        FormValue::Array(value.into_iter().map(FormValue::Value).collect())
    }
}
