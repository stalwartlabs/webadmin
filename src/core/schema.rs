use std::sync::Arc;

use ahash::AHashMap;

use crate::pages::config::Settings;

use super::form::{FormData, FormValue};

#[derive(Default)]
pub struct Schemas {
    pub schemas: AHashMap<&'static str, Arc<Schema>>,
}

pub struct Builder<P, I> {
    pub parent: P,
    pub item: I,
}

#[derive(Clone, Default)]
pub enum Type<S, F> {
    Input,
    InputMulti,
    Secret,
    Text,
    #[default]
    Expression,
    Select(Source<S, F>),
    Checkbox,
    Duration,
}

#[derive(Clone, Default)]
pub struct Field {
    pub id: &'static str,
    pub label_form: &'static str,
    pub label_column: &'static str,
    pub help: Value<&'static str>,
    pub checks: Value<InputCheck>,
    pub typ_: Type<Arc<Schema>, Arc<Field>>,
    pub default: Value<&'static str>,
    pub placeholder: Value<&'static str>,
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

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Schema {}

#[derive(Clone, Default)]
pub struct List {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub fields: Vec<Arc<Field>>,
    pub actions: Vec<Action>,
    pub page_size: u32,
}

#[derive(Clone, Default)]
pub struct Form {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub sections: Vec<Section>,
    pub actions: Vec<Action>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Save,
    Cancel,
    Modify,
    Delete,
    Search,
}

#[derive(Clone, Default)]
pub struct Section {
    pub title: Option<&'static str>,
    pub display: Vec<Eval>,
    pub fields: Vec<Arc<Field>>,
}

#[derive(Clone)]
pub enum Source<S, F> {
    Static(&'static [(&'static str, &'static str)]),
    Dynamic { schema: S, field: F },
}

#[derive(Clone, Default)]
pub struct Value<T> {
    pub if_thens: Vec<IfThen<T>>,
    pub default: Option<T>,
}

#[derive(Clone)]
pub struct IfThen<T> {
    pub eval: Eval,
    pub value: T,
}

#[derive(Clone)]
pub struct Eval {
    pub field: Arc<Field>,
    pub values: Vec<&'static str>,
    pub condition: Condition,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    MatchAny,
    MatchNone,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct InputCheck {
    pub transformers: Vec<Transformer>,
    pub validators: Vec<Validator>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transformer {
    Trim,
    RemoveSpaces,
    Lowercase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validator {
    Required,
    IsEmail,
    IsCron,
    IsId,
    IsHost,
    IsDomain,
    IsPort,
    IsUrl,
    IsGlobPattern,
    IsRegexPattern,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
    MinItems(usize),
    MaxItems(usize),
    IsValidExpression {
        variables: &'static [&'static str],
        functions: &'static [(&'static str, u32)],
        constants: &'static [&'static str],
    },
}

impl Schemas {
    pub fn get(&self, id: &str) -> Arc<Schema> {
        self.schemas.get(id).cloned().unwrap_or_else(|| {
            panic!("Schema {id:?} not found.");
        })
    }

    pub fn build_form(&self, id: &str) -> FormData {
        FormData {
            values: Default::default(),
            errors: Default::default(),
            schema: self.get(id),
        }
    }
}

impl Schema {
    pub fn can_edit(&self) -> bool {
        self.has_list_action(Action::Modify)
    }

    pub fn can_delete(&self) -> bool {
        self.has_list_action(Action::Delete)
    }

    pub fn can_create(&self) -> bool {
        self.has_list_action(Action::Create)
    }

    pub fn has_list_action(&self, action: Action) -> bool {
        self.list.actions.iter().any(|a| *a == action)
    }

    pub fn has_form_action(&self, action: Action) -> bool {
        self.form.actions.iter().any(|a| *a == action)
    }

    pub fn to_settings(&self) -> Settings {
        let mut settings = Settings::default();
        settings.insert("_id".to_string(), "".to_string());

        // Add default values for fields that don't have if-thens
        for field in self.fields.values() {
            if field.default.if_thens.is_empty() {
                if let Some(default) = field.default.default.as_ref() {
                    settings.insert(field.id.to_string(), default.to_string());
                }
            }
        }

        // Add default values for fields that depend on other fields
        for field in self.fields.values() {
            if !field.default.if_thens.is_empty() {
                if let Some(value) = field.default.eval(&settings) {
                    settings.insert(field.id.to_string(), value.to_string());
                }
            }
        }
        settings
    }
}

pub trait SettingsValue {
    fn get(&self, key: &str) -> Option<&str>;
}

impl SettingsValue for Settings {
    fn get(&self, key: &str) -> Option<&str> {
        self.get(key).map(|s| s.as_str())
    }
}

impl SettingsValue for FormData {
    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| match v {
            FormValue::Value(s) => Some(s.as_str()),
            _ => None,
        })
    }
}

impl Field {
    pub fn value(&self, settings: &impl SettingsValue) -> String {
        settings
            .get(self.id)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn label(&self, settings: &impl SettingsValue) -> String {
        let value = self.value(settings);
        match &self.typ_ {
            Type::Select(source) => source
                .display(&value, settings)
                .map(|s| s.to_string())
                .unwrap_or(value),
            _ => value,
        }
    }

    pub fn display(&self, settings: &impl SettingsValue) -> bool {
        self.display.iter().any(|eval| eval.eval(settings))
    }

    pub fn placeholder(&self, settings: &impl SettingsValue) -> Option<&str> {
        self.placeholder.eval(settings).copied()
    }

    pub fn help(&self, settings: &impl SettingsValue) -> Option<&str> {
        self.help.eval(settings).copied()
    }

    pub fn default(&self, settings: &impl SettingsValue) -> Option<&str> {
        self.default.eval(settings).copied()
    }

    pub fn input_check(&self, settings: &impl SettingsValue) -> Option<&InputCheck> {
        self.checks.eval(settings)
    }
}

impl<T> Value<T> {
    pub fn eval(&self, settings: &impl SettingsValue) -> Option<&T> {
        for if_then in &self.if_thens {
            if if_then.eval.eval(settings) {
                return Some(&if_then.value);
            }
        }

        self.default.as_ref()
    }
}

impl Eval {
    pub fn eval(&self, settings: &impl SettingsValue) -> bool {
        let value = settings.get(self.field.id);
        match self.condition {
            Condition::MatchAny => self.values.iter().any(|v| value == Some(v)),
            Condition::MatchNone => self.values.iter().all(|v| value != Some(v)),
        }
    }
}

impl Source<Arc<Schema>, Arc<Field>> {
    pub fn display<'x>(&self, id: &str, settings: &'x impl SettingsValue) -> Option<&'x str> {
        match self {
            Source::Static(items) => items
                .iter()
                .find_map(|(k, v)| if *k == id { Some(*v) } else { None }),
            Source::Dynamic { field, .. } => settings.get(field.id),
        }
    }
}

impl Section {
    pub fn display(&self, settings: &impl SettingsValue) -> bool {
        self.display.iter().any(|eval| eval.eval(settings))
    }
}

impl Schemas {
    pub fn builder() -> Builder<Schemas, ()> {
        Builder {
            parent: Default::default(),
            item: (),
        }
    }
}

impl Builder<Schemas, ()> {
    pub fn new_schema(self, id: &'static str) -> Builder<Schemas, Schema> {
        Builder {
            parent: self.parent,
            item: Schema {
                id,
                list: List {
                    actions: vec![
                        Action::Create,
                        Action::Search,
                        Action::Delete,
                        Action::Modify,
                    ],
                    page_size: 10,
                    ..Default::default()
                },
                form: Form {
                    actions: vec![Action::Save, Action::Cancel],
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    pub fn build(self) -> Schemas {
        self.parent
    }
}

impl Builder<Schemas, Schema> {
    pub fn new_field(self, id: &'static str) -> Builder<(Schemas, Schema), Field> {
        Builder {
            parent: (self.parent, self.item),
            item: Field {
                id,
                ..Default::default()
            },
        }
        .typ(Type::Input)
    }

    pub fn new_id_field(self) -> Builder<(Schemas, Schema), Field> {
        Builder {
            parent: (self.parent, self.item),
            item: Field {
                id: "_id",
                ..Default::default()
            },
        }
        .label("Id")
        .typ(Type::Input)
        .input_check(
            [Transformer::Trim, Transformer::Lowercase],
            [Validator::Required, Validator::IsId],
        )
        .readonly()
    }

    pub fn new_value_field(self) -> Builder<(Schemas, Schema), Field> {
        Builder {
            parent: (self.parent, self.item),
            item: Field {
                id: "_value",
                ..Default::default()
            },
        }
        .label("Value")
        .typ(Type::Input)
        .input_check([Transformer::Trim], [])
    }

    pub fn new_form_section(self) -> Builder<(Schemas, Schema), Section> {
        Builder {
            parent: (self.parent, self.item),
            item: Section::default(),
        }
    }

    pub fn prefix(mut self, prefix: &'static str) -> Self {
        self.item.prefix = Some(prefix);
        self
    }

    pub fn suffix(mut self, suffix: &'static str) -> Self {
        self.item.suffix = Some(suffix);
        self
    }

    pub fn list_title(mut self, title: &'static str) -> Self {
        self.item.list.title = title;
        self
    }

    pub fn list_subtitle(mut self, subtitle: &'static str) -> Self {
        self.item.list.subtitle = subtitle;
        self
    }

    pub fn list_field(mut self, field: &'static str) -> Self {
        self.item.list.fields.push(
            self.item
                .fields
                .get(field)
                .unwrap_or_else(|| {
                    panic!("Field {field:?} not found in schema {:?}.", self.item.id)
                })
                .clone(),
        );
        self
    }

    pub fn list_fields(self, fields: impl IntoIterator<Item = &'static str>) -> Self {
        let mut builder = self;
        for field in fields {
            builder = builder.list_field(field);
        }
        builder
    }

    pub fn list_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.item.list.actions = actions.into_iter().collect();
        self
    }

    pub fn form_title(mut self, title: &'static str) -> Self {
        self.item.form.title = title;
        self
    }

    pub fn form_subtitle(mut self, subtitle: &'static str) -> Self {
        self.item.form.subtitle = subtitle;
        self
    }

    pub fn form_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.item.form.actions = actions.into_iter().collect();
        self
    }

    pub fn names(mut self, singular: &'static str, plural: &'static str) -> Self {
        self.item.name_singular = singular;
        self.item.name_plural = plural;
        self
    }

    pub fn build(mut self) -> Builder<Schemas, ()> {
        self.parent
            .schemas
            .insert(self.item.id, Arc::new(self.item));
        Builder {
            parent: self.parent,
            item: (),
        }
    }
}

impl Builder<(Schemas, Schema), Field> {
    fn field(&self, id: &'static str) -> Arc<Field> {
        self.parent
            .1
            .fields
            .get(id)
            .unwrap_or_else(|| panic!("Field {id:?} not found in schema {:?}.", self.parent.1.id))
            .clone()
    }

    fn schema(&self, id: &'static str) -> Arc<Schema> {
        self.parent
            .0
            .schemas
            .get(id)
            .expect("Schema not found.")
            .clone()
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.item.label_column = label;
        self.item.label_form = label;
        self
    }

    pub fn label_column(mut self, label: &'static str) -> Self {
        self.item.label_column = label;
        self
    }

    pub fn label_form(mut self, label: &'static str) -> Self {
        self.item.label_form = label;
        self
    }

    pub fn help(mut self, help: &'static str) -> Self {
        self.item.help.push_else(help);
        self
    }

    pub fn help_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        value: &'static str,
    ) -> Self {
        self.item
            .help
            .push_if_matches_eq(self.field(field), conditions, value);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.item.readonly = true;
        self
    }

    pub fn typ(mut self, typ_: Type<&'static str, &'static str>) -> Self {
        self.item.typ_ = match typ_ {
            Type::Select(Source::Dynamic { schema, field }) => Type::Select(Source::Dynamic {
                schema: self.schema(schema),
                field: self.field(field),
            }),
            typ_ => typ_.into(),
        };
        self
    }

    /*pub fn typ_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        typ_: Type<&'static str, &'static str>,
    ) -> Self {
        self.item.typ_.push_if_matches_eq(
            self.field(field),
            conditions,
            match typ_ {
                Type::Select(Source::Dynamic { schema, field }) => Type::Select(Source::Dynamic {
                    schema: self.schema(schema),
                    field: self.field(field),
                }),
                typ_ => typ_.into(),
            },
        );
        self
    }*/

    pub fn input_check_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        self.item.checks.push_if_matches_eq(
            self.field(field),
            conditions,
            InputCheck::new(transformers, validators),
        );
        self
    }

    pub fn input_check(
        mut self,
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        self.item
            .checks
            .push_else(InputCheck::new(transformers, validators));
        self
    }

    pub fn placeholder(mut self, placeholder: &'static str) -> Self {
        self.item.placeholder.push_else(placeholder);
        self
    }

    pub fn default(mut self, default: &'static str) -> Self {
        self.item.default.push_else(default);
        self.item.placeholder.push_else(default);
        self
    }

    pub fn default_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        value: &'static str,
    ) -> Self {
        self.item
            .default
            .push_if_matches_eq(self.field(field), conditions, value);
        self
    }

    pub fn display_if(
        mut self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
        condition: Condition,
    ) -> Self {
        self.item.display.push(Eval {
            field: self.field(field),
            values: values.into_iter().collect(),
            condition,
        });
        self
    }

    pub fn display_if_eq(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.display_if(field, values, Condition::MatchAny)
    }

    pub fn display_if_ne(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.display_if(field, values, Condition::MatchNone)
    }

    pub fn build(mut self) -> Builder<Schemas, Schema> {
        self.parent
            .1
            .fields
            .insert(self.item.id, Arc::new(self.item));
        Builder {
            parent: self.parent.0,
            item: self.parent.1,
        }
    }

    pub fn new_field(mut self, id: &'static str) -> Self {
        let cloned_field = Field {
            id,
            typ_: self.item.typ_.clone(),
            display: self.item.display.clone(),
            checks: self.item.checks.clone(),
            ..Default::default()
        };
        self.parent
            .1
            .fields
            .insert(self.item.id, Arc::new(self.item));
        Builder {
            parent: self.parent,
            item: cloned_field,
        }
    }
}

impl Builder<(Schemas, Schema), Section> {
    pub fn title(mut self, title: &'static str) -> Self {
        self.item.title = Some(title);
        self
    }

    pub fn field(mut self, field: &'static str) -> Self {
        self.item.fields.push(
            self.parent
                .1
                .fields
                .get(field)
                .unwrap_or_else(|| {
                    panic!(
                        "Field {field:?} not found in schema {:?}.",
                        self.parent.1.id
                    )
                })
                .clone(),
        );
        self
    }

    pub fn fields(self, fields: impl IntoIterator<Item = &'static str>) -> Self {
        let mut builder = self;
        for field in fields {
            builder = builder.field(field);
        }
        builder
    }

    fn display_if(
        mut self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
        condition: Condition,
    ) -> Self {
        self.item.display.push(Eval {
            field: self
                .parent
                .1
                .fields
                .get(field)
                .unwrap_or_else(|| {
                    panic!(
                        "Field {field:?} not found in schema {:?}.",
                        self.parent.1.id
                    )
                })
                .clone(),
            values: values.into_iter().collect(),
            condition,
        });
        self
    }

    pub fn display_if_eq(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.display_if(field, values, Condition::MatchAny)
    }

    pub fn display_if_ne(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.display_if(field, values, Condition::MatchNone)
    }

    pub fn build(mut self) -> Builder<Schemas, Schema> {
        self.parent.1.form.sections.push(self.item);
        Builder {
            parent: self.parent.0,
            item: self.parent.1,
        }
    }
}

impl<T> Value<T> {
    pub fn push_if_matches_eq(
        &mut self,
        field: Arc<Field>,
        contains: impl IntoIterator<Item = &'static str>,
        then: T,
    ) {
        self.if_thens.push(IfThen {
            eval: Eval {
                field,
                values: contains.into_iter().collect(),
                condition: Condition::MatchAny,
            },
            value: then,
        });
    }

    pub fn push_if_matches_ne(
        &mut self,
        field: Arc<Field>,
        contains: impl IntoIterator<Item = &'static str>,
        then: T,
    ) {
        self.if_thens.push(IfThen {
            eval: Eval {
                field,
                values: contains.into_iter().collect(),
                condition: Condition::MatchNone,
            },
            value: then,
        });
    }

    pub fn push_else(&mut self, value: T) {
        self.default = Some(value);
    }
}

impl InputCheck {
    pub fn new(
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        InputCheck {
            transformers: transformers.into_iter().collect(),
            validators: validators.into_iter().collect(),
        }
    }
}

impl From<Type<&'static str, &'static str>> for Type<Arc<Schema>, Arc<Field>> {
    fn from(typ_: Type<&'static str, &'static str>) -> Self {
        match typ_ {
            Type::Checkbox => Type::Checkbox,
            Type::Duration => Type::Duration,
            Type::Expression => Type::Expression,
            Type::Input => Type::Input,
            Type::InputMulti => Type::InputMulti,
            Type::Secret => Type::Secret,
            Type::Text => Type::Text,
            Type::Select(Source::Static(items)) => Type::Select(Source::Static(items)),
            Type::Select(_) => unreachable!(),
        }
    }
}
