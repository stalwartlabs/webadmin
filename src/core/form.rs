use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;

use ahash::AHashMap;
use leptos::RwSignal;

use crate::pages::config::{ArrayValues, Settings};

use super::schema::{NumberType, Type};

use super::schema::{InputCheck, Schema, Transformer, Validator};

#[derive(Clone, PartialEq, Eq, Default)]
pub struct FormData {
    pub values: AHashMap<String, FormValue>,
    pub errors: AHashMap<String, FormError>,
    pub schema: Arc<Schema>,
    pub is_update: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValue {
    Value(String),
    Array(Vec<String>),
    Expression(Vec<ExpressionValue>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionValue {
    IfThen { expr: String, value: String },
    Else { value: String },
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

    pub fn value_as_str(&self, id: &str) -> Option<&str> {
        self.values.get(id).and_then(|v| match v {
            FormValue::Value(v) => Some(v.as_str()),
            _ => None,
        })
    }

    pub fn value_is_empty(&self, id: &str) -> bool {
        self.values.get(id).map_or(true, |v| match v {
            FormValue::Value(v) => v.is_empty(),
            FormValue::Array(v) => v.is_empty(),
            FormValue::Expression(v) => v.is_empty(),
        })
    }

    pub fn has_value(&self, id: &str) -> bool {
        self.values.contains_key(id)
    }

    pub fn update(&mut self, id: &str, value: impl Into<FormValue>) {
        let value = value.into();
        self.cascading_reset(id);
        self.values.insert(id.to_string(), value);
        self.update_defaults(id);
        self.errors.remove(id);
        let c = log::debug!("Values: {:?}", self.values);
    }

    pub fn remove(&mut self, id: &str) {
        self.values.remove(id);
        self.errors.remove(id);
    }

    pub fn array_value<'x>(&'x self, id: &str) -> Box<dyn Iterator<Item = &'x str> + 'x> {
        match self.values.get(id) {
            Some(FormValue::Array(values)) => Box::new(values.iter().map(|v| v.as_str())),
            Some(FormValue::Value(v)) => Box::new(std::iter::once(v.as_str())),
            _ => Box::new([].into_iter()),
        }
    }

    pub fn array_set(&mut self, id: &str, values: impl IntoIterator<Item = impl Into<String>>) {
        self.values.insert(
            id.to_string(),
            FormValue::Array(values.into_iter().map(Into::into).collect()),
        );
        self.errors.remove(id);
    }

    pub fn array_update(&mut self, id: &str, idx: usize, value: impl Into<String>) {
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
            FormValue::Value(v) if idx == 0 => {
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

    pub fn array_push(&mut self, id: &str, value: impl Into<String>) {
        let v = self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Array(vec![]));

        match v {
            FormValue::Value(val) => {
                *v = FormValue::Array(vec![std::mem::take(val), value.into()]);
            }
            FormValue::Array(arr) => {
                arr.push(value.into());
            }
            _ => unreachable!(),
        };
        self.errors.remove(id);
    }

    fn cascading_reset(&mut self, id: &str) {
        let schema = self.schema.clone();
        let mut ids = vec![id.to_string()];

        while let Some(id) = ids.pop() {
            // Remove the field
            let c = log::debug!("removing {id:?}");
            let prefix = format!("{id}.");
            self.values
                .retain(|k, _| k != &id && !k.starts_with(&prefix));

            // Obtain fields that depend on this field
            for field in schema.fields.values() {
                if ids.iter().all(|id| id != field.id)
                    && (field
                        .default
                        .if_thens
                        .iter()
                        .any(|if_then| if_then.eval.field.id == id)
                        || field.display.iter().any(|eval| eval.field.id == id))
                {
                    ids.push(field.id.to_string());
                }
            }
        }
    }

    fn update_defaults(&mut self, id: &str) {
        let schema = self.schema.clone();
        let mut ids = vec![id.to_string()];

        while let Some(id) = ids.pop() {
            for field in schema.fields.values() {
                if field
                    .default
                    .if_thens
                    .iter()
                    .any(|if_then| if_then.eval.field.id == id)
                    || (field.display.iter().any(|eval| eval.field.id == id) && field.display(self))
                {
                    if let Some(value) = field.default.eval(self) {
                        let c = log::debug!("adding default {:?} = {value:?}", field.id);
                        self.set(field.id.to_string(), *value);
                    }

                    if ids.iter().all(|id| id != field.id) {
                        ids.push(field.id.to_string());
                    }
                }
            }
        }
    }

    pub fn apply_defaults(&mut self) {
        // Add default values for top-level fields
        let schema = self.schema.clone();
        let mut added_fields = Vec::new();
        for field in schema.fields.values() {
            if field.display.is_empty() && field.default.if_thens.is_empty() {
                if let Some(default) = field.default.default.as_ref() {
                    self.set(field.id.to_string(), default.to_string());
                    added_fields.push(field.id);
                }
            }
        }

        // Add default values for fields that depend on top-level fields
        for field_id in added_fields {
            self.update_defaults(field_id);
        }

        log::debug!("Applied defaults: {:#?}", self.values);
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
            if !field.display(self) {
                continue;
            }

            if let Some(check) = field.input_check(self) {
                match field.typ_ {
                    Type::Input
                    | Type::Secret
                    | Type::Text
                    | Type::Size
                    | Type::Checkbox
                    | Type::Duration
                    | Type::Select(_) => {
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
                    Type::Array => {
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
                }
            }
        }

        self.errors.is_empty()
    }

    pub fn from_settings(schema: Arc<Schema>, settings: Option<Settings>) -> Self {
        let mut data = FormData::from(schema);
        let schema = data.schema.clone();

        if let Some(mut settings) = settings {
            for field in schema.fields.values() {
                match &field.typ_ {
                    Type::Input
                    | Type::Secret
                    | Type::Text
                    | Type::Select(_)
                    | Type::Checkbox
                    | Type::Duration
                    | Type::Size => {
                        if let Some(value) = settings.remove(field.id) {
                            data.set(field.id, value);
                        }
                    }
                    Type::Array => {
                        data.array_set(
                            field.id,
                            settings.array_values(field.id).map(|(_, value)| value),
                        );
                    }
                    Type::Expression => todo!(),
                }
            }
            data.is_update = true;
        } else {
            data.apply_defaults();
        }
        data
    }

    pub fn into_signal(self) -> RwSignal<Self> {
        RwSignal::new(self)
    }
}

impl InputCheck {
    pub fn check_value(&self, mut value: String) -> Result<String, Cow<'static, str>> {
        for transformer in &self.transformers {
            value = match transformer {
                Transformer::Trim => value.trim().to_string(),
                Transformer::RemoveSpaces => value.replace(' ', ""),
                Transformer::Lowercase => value.to_lowercase(),
                Transformer::Uppercase => value.to_uppercase(),
            };
        }

        if !value.is_empty() {
            for validator in &self.validators {
                match validator {
                    Validator::IsEmail => {
                        if !value.contains('@') {
                            return Err("This field must be a valid email address".into());
                        }
                    }
                    Validator::IsCron => {
                        if value.split_whitespace().count() != 3 {
                            return Err("This field must be a valid cron expression".into());
                        }
                    }
                    Validator::IsId => {
                        if let Some(ch) = value
                            .chars()
                            .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
                        {
                            return Err(format!("Invalid character '{ch}' in this field").into());
                        }
                    }
                    Validator::IsHost => {
                        if value.contains('/') || value.contains(':') {
                            return Err("This field must be a valid hostname".into());
                        }
                    }
                    Validator::IsPort => {
                        if value.parse::<u16>().is_err() {
                            return Err("This field must be a valid port number".into());
                        }
                    }
                    Validator::IsUrl => {
                        if !value.contains("://") {
                            return Err("This field must be a valid URL".into());
                        }
                    }
                    Validator::IsDomain => {
                        if !value.contains('.') || value.starts_with('.') || value.ends_with('.') {
                            return Err("This field must be a valid domain name".into());
                        }
                    }
                    Validator::IsGlobPattern => {
                        let todo = 1;
                    }
                    Validator::IsRegexPattern => {
                        let todo = 1;
                    }
                    Validator::MinLength(length) => {
                        if value.len() < *length {
                            return Err(format!(
                                "This field must be at least {} characters",
                                length
                            )
                            .into());
                        }
                    }
                    Validator::MaxLength(length) => {
                        if value.len() > *length {
                            return Err(format!(
                                "This field must be at most {} characters",
                                length
                            )
                            .into());
                        }
                    }
                    Validator::MinValue(val) => match val {
                        NumberType::Integer(val) => {
                            if value.parse::<i64>().ok().filter(|v| v >= val).is_none() {
                                return Err(format!("This field must be at least {val}").into());
                            }
                        }
                        NumberType::Float(val) => {
                            if value.parse::<f64>().ok().filter(|v| v >= val).is_none() {
                                return Err(format!("This field must be at least {val}").into());
                            }
                        }
                    },
                    Validator::MaxValue(val) => match val {
                        NumberType::Integer(val) => {
                            if value.parse::<i64>().ok().filter(|v| v <= val).is_none() {
                                return Err(format!("This field must be at most {}", val).into());
                            }
                        }
                        NumberType::Float(val) => {
                            if value.parse::<f64>().ok().filter(|v| v <= val).is_none() {
                                return Err(format!("This field must be at most {}", val).into());
                            }
                        }
                    },
                    Validator::IsValidExpression { .. }
                    | Validator::MinItems(_)
                    | Validator::MaxItems(_)
                    | Validator::Required => (),
                }
            }
        } else if self.validators.contains(&Validator::Required) {
            return Err("This field is required".into());
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
        FormValue::Array(value)
    }
}
