/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::borrow::Cow;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use ahash::AHashMap;
use leptos::RwSignal;

use crate::pages::config::{Settings, SettingsValues};

use super::expr::parser::ExpressionParser;
use super::expr::tokenizer::Tokenizer;
use super::expr::{Constant, ParseValue, Token};
use super::schema::{NumberType, SchemaType, SelectType, Type};

use super::schema::{InputCheck, Schema, Transformer, Validator};

pub type ExternalSources = AHashMap<String, Vec<(String, String)>>;

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct FormData {
    pub values: AHashMap<String, FormValue>,
    pub errors: AHashMap<String, FormError>,
    pub external_sources: Arc<ExternalSources>,
    pub schema: Arc<Schema>,
    pub is_update: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValue {
    Value(String),
    Array(Vec<String>),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Expression {
    pub if_thens: Vec<ExpressionIfThen>,
    pub else_: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionIfThen {
    pub if_: String,
    pub then_: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormError {
    pub id: FormErrorType,
    pub error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormErrorType {
    Expression(ExpressionError<usize>),
    Array(usize),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionError<T> {
    If(T),
    Then(T),
    Else,
}

impl Copy for ExpressionError<usize> {}

impl FormData {
    pub fn with_external_sources(mut self, sources: impl Into<Arc<ExternalSources>>) -> Self {
        self.external_sources = sources.into();
        self
    }

    pub fn with_value(mut self, id: impl Into<String>, value: impl Into<FormValue>) -> Self {
        self.values.insert(id.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| match v {
            FormValue::Value(s) => Some(s.as_str()),
            _ => None,
        })
    }

    pub fn set(&mut self, id: impl Into<String>, value: impl Into<FormValue>) {
        self.values.insert(id.into(), value.into());
    }

    pub fn new_error(&mut self, id: impl Into<String>, error: impl Into<String>) {
        self.errors.insert(
            id.into(),
            FormError {
                id: FormErrorType::None,
                error: error.into(),
            },
        );
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
        self.values.get(id).is_none_or(|v| match v {
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
        //let c = log::debug!("Updating field {id:?} with value {value:?}");
        self.values.insert(id.to_string(), value);
        self.update_defaults(id);
        self.errors.remove(id);
    }

    pub fn remove(&mut self, id: &str) {
        self.values.remove(id);
        self.errors.remove(id);
    }

    pub fn reset(&mut self) {
        self.values.clear();
        self.errors.clear();
        self.apply_defaults(false);
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

    pub fn array_push(&mut self, id: &str, value: impl Into<String>, unique: bool) {
        let v = self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Array(vec![]));
        let value = value.into();

        match v {
            FormValue::Value(val) if !unique || val != &value => {
                *v = FormValue::Array(vec![std::mem::take(val), value]);
            }
            FormValue::Array(arr) if !unique || !arr.contains(&value) => {
                arr.push(value);
            }
            _ => (),
        };
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

    pub fn array_delete_item(&mut self, id: &str, item: &str) {
        let left = self.values.get_mut(id).and_then(|v| match v {
            FormValue::Array(values) => {
                values.retain(|v| v != item);
                Some(values.len())
            }
            FormValue::Value(value) if value == item => Some(0),
            _ => None,
        });
        if left == Some(0) {
            self.values.remove(id);
        }
        self.errors.remove(id);
    }

    pub fn expr_if_thens<'x>(
        &'x self,
        id: &str,
    ) -> Box<dyn Iterator<Item = &'x ExpressionIfThen> + 'x> {
        match self.values.get(id) {
            Some(FormValue::Expression(expr)) => Box::new(expr.if_thens.iter()),
            _ => Box::new([].into_iter()),
        }
    }

    pub fn expr_else(&self, id: &str) -> Option<&str> {
        match self.values.get(id) {
            Some(FormValue::Expression(expr)) => Some(expr.else_.as_str()),
            Some(FormValue::Value(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn expr_update_else(&mut self, id: &str, value: impl Into<String>) {
        match self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Expression(Expression::default()))
        {
            FormValue::Expression(expr) => {
                expr.else_ = value.into();
            }
            FormValue::Value(_) => {
                self.values.insert(
                    id.to_string(),
                    FormValue::Expression(Expression {
                        else_: value.into(),
                        ..Default::default()
                    }),
                );
            }
            _ => (),
        }
        self.errors.remove(id);
    }

    pub fn expr_push_if_then(
        &mut self,
        id: &str,
        if_: impl Into<String>,
        then_: impl Into<String>,
    ) {
        let if_then = ExpressionIfThen {
            if_: if_.into(),
            then_: then_.into(),
        };

        match self
            .values
            .entry(id.to_string())
            .or_insert_with(|| FormValue::Expression(Expression::default()))
        {
            FormValue::Expression(expr) => {
                expr.if_thens.push(if_then);
            }
            FormValue::Value(v) => {
                let else_ = std::mem::take(v);
                self.values.insert(
                    id.to_string(),
                    FormValue::Expression(Expression {
                        if_thens: vec![if_then],
                        else_,
                    }),
                );
            }
            _ => (),
        }
        self.errors.remove(id);
    }

    pub fn expr_delete_if_then(&mut self, id: &str, idx: usize) {
        if let Some(FormValue::Expression(expr)) = self.values.get_mut(id) {
            expr.if_thens.remove(idx);
        }
        self.errors.remove(id);
    }

    pub fn expr_update_if(&mut self, id: &str, idx: usize, if_: impl Into<String>) {
        if let Some(FormValue::Expression(expr)) = self.values.get_mut(id) {
            if let Some(if_then) = expr.if_thens.get_mut(idx) {
                if_then.if_ = if_.into();
            }
        }
        self.errors.remove(id);
    }

    pub fn expr_update_then(&mut self, id: &str, idx: usize, then_: impl Into<String>) {
        if let Some(FormValue::Expression(expr)) = self.values.get_mut(id) {
            if let Some(if_then) = expr.if_thens.get_mut(idx) {
                if_then.then_ = then_.into();
            }
        }
        self.errors.remove(id);
    }

    fn cascading_reset(&mut self, id: &str) {
        let schema = self.schema.clone();
        let mut ids = vec![id.to_string()];
        let mut removed_fields = Vec::new();

        while let Some(id) = ids.pop() {
            // Remove the field
            removed_fields.push(id.clone());
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

        //let c = log::debug!("Removed fields {removed_fields:?}");
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
                    if let Some(default) = field.default.eval(self) {
                        //let c = log::debug!("adding default {:?} = {default:?}", field.id);
                        let value = match (&field.typ_, default) {
                            (Type::Expression, FormValue::Value(default)) => {
                                FormValue::Expression(Expression {
                                    else_: default.to_string(),
                                    ..Default::default()
                                })
                            }
                            _ => default.clone(),
                        };
                        self.set(field.id.to_string(), value);
                    }

                    if ids.iter().all(|id| id != field.id) {
                        ids.push(field.id.to_string());
                    }
                }
            }
        }
    }

    pub fn apply_defaults(&mut self, only_required: bool) {
        // Add default values for top-level fields
        let schema = self.schema.clone();
        let mut added_fields = Vec::new();
        for field in schema.fields.values() {
            if field.display.is_empty()
                && field.default.if_thens.is_empty()
                && !self.values.contains_key(field.id)
                && field.checks.if_thens.is_empty()
                && (!only_required
                    || (matches!(field.typ_, Type::Boolean | Type::Expression)
                        || field
                            .checks
                            .default
                            .as_ref()
                            .is_some_and(|d| d.validators.contains(&Validator::Required))))
            {
                if let Some(default) = field.default.default.as_ref() {
                    let value = match (&field.typ_, default) {
                        (Type::Expression, FormValue::Value(default)) => {
                            FormValue::Expression(Expression {
                                else_: default.to_string(),
                                ..Default::default()
                            })
                        }
                        _ => default.clone(),
                    };
                    self.set(field.id.to_string(), value);
                    added_fields.push(field.id);
                }
            }
        }

        /*let c = log::debug!(
            "Applied defaults to fields {added_fields:?}: {:?}",
            self.values
        );*/

        // Add default values for fields that depend on top-level fields
        for field_id in added_fields {
            self.update_defaults(field_id);
        }
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
            log::debug!("Skipping validation, form has errors: {:#?}", self.errors);
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
                    | Type::Boolean
                    | Type::Duration
                    | Type::Rate
                    | Type::Cron
                    | Type::Select {
                        typ: SelectType::Single,
                        ..
                    } => {
                        match check.check_value(self.value::<String>(field.id).unwrap_or_default())
                        {
                            Ok(value) => {
                                if !value.is_empty() {
                                    self.values.insert(field.id.into(), value.into());
                                } else {
                                    self.values.remove(field.id);
                                }
                            }
                            Err(err) => {
                                self.new_error(field.id, err);
                            }
                        }
                    }
                    Type::Array
                    | Type::Select {
                        typ: SelectType::Many | SelectType::ManyWithSearch,
                        ..
                    } => {
                        let mut total_values = 0;
                        let mut has_errors = false;

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
                                            id: FormErrorType::Array(idx),
                                            error: err.to_string(),
                                        },
                                    );
                                    has_errors = true;
                                }
                            }
                        }

                        if !has_errors {
                            for validator in &check.validators {
                                match validator {
                                    Validator::Required => {
                                        if total_values == 0 {
                                            self.new_error(field.id, "This field is required");
                                        }
                                    }
                                    Validator::MinItems(min) => {
                                        if total_values < *min {
                                            self.new_error(
                                                field.id,
                                                format!("At least {min} items are required"),
                                            );
                                        }
                                    }
                                    Validator::MaxItems(max) => {
                                        if total_values > *max {
                                            self.new_error(
                                                field.id,
                                                format!("At most {max} items are allowed"),
                                            );
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    Type::Expression => {
                        let mut has_expression = false;
                        let validator = *check
                            .validators
                            .iter()
                            .find_map(|v| match v {
                                Validator::IsValidExpression(v) => Some(v),
                                _ => None,
                            })
                            .unwrap_or_else(|| {
                                panic!("Missing expression validator for field {}", field.id)
                            });

                        if let Some(FormValue::Expression(expr)) = self.values.get(field.id) {
                            for (expr_item, expr_value) in expr
                                .if_thens
                                .iter()
                                .enumerate()
                                .flat_map(|(idx, if_then)| {
                                    [
                                        (ExpressionError::If(idx), &if_then.if_),
                                        (ExpressionError::Then(idx), &if_then.then_),
                                    ]
                                })
                                .chain([(ExpressionError::Else, &expr.else_)])
                            {
                                match ExpressionParser::new(Tokenizer::new(expr_value, |token| {
                                    if validator.variables.contains(&token) {
                                        Ok(Token::Variable(0))
                                    } else if validator.constants.contains(&token) {
                                        Ok(Token::Constant(Constant::Integer(0)))
                                    } else {
                                        Duration::parse_value(token)
                                            .map(|d| {
                                                Token::Constant(Constant::Integer(
                                                    d.as_secs() as i64
                                                ))
                                            })
                                            .ok_or_else(|| {
                                                format!(
                                                    "Invalid variable or function name {:?}",
                                                    token
                                                )
                                            })
                                    }
                                }))
                                .parse()
                                {
                                    Ok(expr) => {
                                        if matches!(expr_item, ExpressionError::Else) {
                                            has_expression = true;
                                        } else if expr.items.is_empty() {
                                            self.errors.insert(
                                                field.id.to_string(),
                                                FormError {
                                                    id: FormErrorType::Expression(expr_item),
                                                    error: "This expression cannot be empty"
                                                        .to_string(),
                                                },
                                            );
                                            has_expression = true;
                                            break;
                                        }
                                    }
                                    Err(error) => {
                                        self.errors.insert(
                                            field.id.to_string(),
                                            FormError {
                                                id: FormErrorType::Expression(expr_item),
                                                error,
                                            },
                                        );
                                        has_expression = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !has_expression && check.validators.contains(&Validator::Required) {
                            self.errors.insert(
                                field.id.to_string(),
                                FormError {
                                    id: FormErrorType::Expression(ExpressionError::Else),
                                    error: "This field is required".to_string(),
                                },
                            );
                        }
                    }
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
                    | Type::Select {
                        typ: SelectType::Single,
                        ..
                    }
                    | Type::Boolean
                    | Type::Duration
                    | Type::Rate
                    | Type::Cron
                    | Type::Size => {
                        if let Some(value) = settings.remove(field.id) {
                            data.set(field.id, value);
                        }
                    }
                    Type::Array
                    | Type::Select {
                        typ: SelectType::Many | SelectType::ManyWithSearch,
                        ..
                    } => {
                        let values = settings.array_values(field.id);
                        if !values.is_empty() {
                            data.array_set(field.id, values.into_iter().map(|(_, value)| value));
                        }
                    }
                    Type::Expression => {
                        let mut expr = Expression::default();
                        if let Some(else_) = settings.remove(field.id) {
                            expr.else_ = else_;
                        } else {
                            let mut last_if = "";
                            let mut last_then = "";
                            let mut last_array_pos = "";
                            let field_prefix = format!("{}.", field.id);

                            for (key, value) in settings.array_values(field.id) {
                                let value = value.trim();
                                if value.is_empty() {
                                    log::warn!("Ignoring empty expression value");
                                    continue;
                                }

                                if let Some((array_pos, statement)) = key
                                    .strip_prefix(&field_prefix)
                                    .and_then(|v| v.split_once('.'))
                                {
                                    if array_pos != last_array_pos {
                                        if !last_array_pos.is_empty() {
                                            if !last_if.is_empty() && !last_then.is_empty() {
                                                expr.if_thens.push(ExpressionIfThen {
                                                    if_: last_if.to_string(),
                                                    then_: last_then.to_string(),
                                                });
                                            } else {
                                                log::warn!("Ignoring incomplete expression in key {key:?} with value {value:?}.");
                                            }
                                            last_if = "";
                                            last_then = "";
                                        }
                                        last_array_pos = array_pos;
                                    }

                                    match statement {
                                        "if" => {
                                            if last_if.is_empty() {
                                                last_if = value;
                                            } else {
                                                log::warn!("Ignoring duplicate 'if' statement in key {key:?} with value {value:?}.");
                                            }
                                        }
                                        "then" => {
                                            if last_then.is_empty() {
                                                last_then = value;
                                            } else {
                                                log::warn!("Ignoring duplicate 'then' statement in key {key:?} with value {value:?}.");
                                            }
                                        }
                                        "else" => {
                                            if expr.else_.is_empty() {
                                                expr.else_ = value.to_string();
                                            } else {
                                                log::warn!("Ignoring duplicate 'else' statement in key {key:?} with value {value:?}.");
                                            }
                                        }
                                        _ => {
                                            log::warn!("Ignoring unknown expression key {key:?} with value {value:?}.")
                                        }
                                    }
                                } else {
                                    log::warn!("Ignoring unknown expression key {key:?} with value {value:?}.")
                                }
                            }

                            if !last_if.is_empty() && !last_then.is_empty() {
                                expr.if_thens.push(ExpressionIfThen {
                                    if_: last_if.to_string(),
                                    then_: last_then.to_string(),
                                });
                            } else if !last_if.is_empty() || !last_then.is_empty() {
                                log::warn!("Ignoring incomplete expression with 'if' {last_if:?} and 'then' {last_then:?}.");
                            }

                            if !expr.if_thens.is_empty() && expr.else_.is_empty() {
                                log::warn!("Missing 'else' statement in expression {:?}.", expr);
                            }
                        }

                        if !expr.is_empty() {
                            data.set(field.id, FormValue::Expression(expr));
                        }
                    }
                }
            }
            data.is_update = true;
            data.apply_defaults(schema.typ != SchemaType::List);
        } else {
            data.apply_defaults(false);
        }
        data
    }

    pub fn is_required(&self, id: &str) -> bool {
        self.schema.fields.get(id).unwrap().is_required(self)
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
                Transformer::HashSecret => {
                    if !is_hashed_secret(&value) {
                        pwhash::sha512_crypt::hash(value).unwrap()
                    } else {
                        value
                    }
                }
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
                    Validator::IsId => {
                        if let Some(ch) = value
                            .chars()
                            .find(|c| !c.is_alphanumeric() && !['_', '-', '.'].contains(c))
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
                    Validator::IsSocketAddr => {
                        if value.parse::<SocketAddr>().is_err() {
                            return Err("This field must be a valid socket address".into());
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
                    Validator::IsRegex => {
                        if regex::Regex::new(&value).is_err() {
                            return Err("This field must be a valid regular expression".into());
                        }
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
                    Validator::IsIpOrMask => {
                        let value = if let Some((ip, mask)) = value.rsplit_once('/') {
                            if mask.parse::<u8>().is_err() {
                                return Err("Invalid IP address mask".into());
                            }
                            ip
                        } else {
                            value.as_str()
                        };

                        if value.parse::<std::net::IpAddr>().is_err() {
                            return Err("This field must be a valid IP address or network".into());
                        }
                    }
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

fn is_hashed_secret(value: &str) -> bool {
    if let Some(value) = value.strip_prefix('$') {
        value.starts_with("argon2")
            || value.starts_with("pbkdf2")
            || value.starts_with("scrypt")
            || value.starts_with("2")
            || value.starts_with("6$")
            || value.starts_with("5$")
            || value.starts_with("sha1")
            || value.starts_with("1")
    } else if let Some(value) = value.strip_prefix('{') {
        value.starts_with("ARGON2")
            || value.starts_with("PBKDF2")
            || value.starts_with("SSHA")
            || value.starts_with("SHA")
            || value.starts_with("MD5")
            || value.starts_with("CRYPT")
    } else {
        false
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

impl From<&[&str]> for FormValue {
    fn from(value: &[&str]) -> Self {
        FormValue::Array(value.iter().map(|v| v.to_string()).collect())
    }
}

impl From<Expression> for FormValue {
    fn from(value: Expression) -> Self {
        FormValue::Expression(value)
    }
}

impl Expression {
    pub fn new(
        if_thens: impl IntoIterator<Item = (&'static str, &'static str)>,
        default: impl AsRef<str>,
    ) -> Self {
        Self {
            if_thens: if_thens
                .into_iter()
                .map(|(if_, then)| ExpressionIfThen {
                    if_: if_.to_string(),
                    then_: then.to_string(),
                })
                .collect(),
            else_: default.as_ref().to_string(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.if_thens.is_empty() && self.else_.is_empty()
    }
}

impl ExpressionIfThen {
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.if_.hash(&mut hasher);
        self.then_.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for FormValue {
    fn default() -> Self {
        FormValue::Value("".to_string())
    }
}
