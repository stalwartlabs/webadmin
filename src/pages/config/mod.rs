pub mod schema;

use std::sync::Arc;

use ahash::AHashMap;

#[derive(Clone, Default)]
pub enum Type {
    Input {
        transformers: Vec<Transformer>,
        validators: Vec<Validator>,
    },
    InputMulti {
        transformers: Vec<Transformer>,
        validators: Vec<Validator>,
    },
    Secret {
        transformers: Vec<Transformer>,
        validators: Vec<Validator>,
    },
    #[default]
    Expression,
    Select(Arc<Source>),
    Throttle,
    Checkbox,
    Duration,
}

#[derive(Clone, Copy)]
pub enum Transformer {
    Trim,
    RemoveSpaces,
    Lowercase,
}

#[derive(Clone, Copy)]
pub enum Validator {
    Required,
    IsEmail,
    IsCron,
    IsId,
    IsHost,
    IsPort,
    IsUrl,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
}

#[derive(Clone, Default)]
pub struct Field {
    pub id: &'static str,
    pub label_form: &'static str,
    pub label_column: &'static str,
    pub help: Value,
    pub typ_: Type,
    pub default: Value,
    pub placeholder: Value,
    pub display: Vec<Eval>,
    pub readonly: bool,
}

#[derive(Clone, Default)]
pub struct Schema {
    pub id: &'static str,
    pub name_singular: &'static str,
    pub name_plural: &'static str,
    pub fields: AHashMap<&'static str, Arc<Field>>,
    pub prefix: Option<&'static str>,
    pub suffix: Option<&'static str>,
    pub list: List,
    pub form: Form,
}

#[derive(Clone, Default)]
pub struct List {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub fields: Vec<Arc<Field>>,
}

#[derive(Clone, Default)]
pub struct Form {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub sections: Vec<Section>,
}

#[derive(Clone, Default)]
pub struct Section {
    pub title: Option<&'static str>,
    pub display: Vec<Eval>,
    pub fields: Vec<Arc<Field>>,
}

#[derive(Clone)]
pub enum Source {
    Static(Vec<SelectItem>),
    Dynamic {
        schema: Arc<Schema>,
        field: Arc<Field>,
    },
}

#[derive(Clone, Default)]
pub struct Value {
    pub if_thens: Vec<IfThen>,
}

#[derive(Clone)]
pub struct IfThen {
    pub eval: Eval,
    pub value: &'static str,
}

#[derive(Clone)]
pub struct SelectItem {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Default)]
pub enum Eval {
    #[default]
    True,
    Condition {
        field: Arc<Field>,
        values: Vec<&'static str>,
        condition: Condition,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    MatchAny,
    MatchNone,
}
