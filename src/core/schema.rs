/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::{hash::Hasher, sync::Arc};

use ahash::AHashMap;

use super::form::{FormData, FormValue};

#[derive(Default)]
pub struct Schemas {
    pub schemas: AHashMap<&'static str, Arc<Schema>>,
}

pub struct Builder<P, I> {
    pub parent: P,
    pub item: I,
}

#[derive(Clone, Default, Debug)]
pub enum Type<S, F> {
    Input,
    Array,
    Secret,
    Text,
    #[default]
    Expression,
    Select {
        typ: SelectType,
        source: Source<S, F>,
    },
    Boolean,
    Duration,
    Rate,
    Size,
    Cron,
}

#[derive(Clone, Copy, Default, Debug)]
pub enum SelectType {
    #[default]
    Single,
    Many,
    ManyWithSearch,
}

#[derive(Clone, Default, Debug)]
pub struct Field {
    pub id: &'static str,
    pub label_form: &'static str,
    pub label_column: &'static str,
    pub help: Option<&'static str>,
    pub checks: Value<InputCheck>,
    pub typ_: Type<Arc<Schema>, Arc<Field>>,
    pub default: Value<FormValue>,
    pub placeholder: Value<&'static str>,
    pub display: Vec<Eval>,
    pub readonly: bool,
    pub enterprise: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Schema {
    pub id: &'static str,
    pub name_singular: &'static str,
    pub name_plural: &'static str,
    pub fields: AHashMap<&'static str, Arc<Field>>,
    pub typ: SchemaType,
    pub reload_prefix: Option<&'static str>,
    pub list: List,
    pub form: Form,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum SchemaType {
    Record {
        prefix: &'static str,
        suffix: &'static str,
    },
    Entry {
        prefix: &'static str,
    },
    #[default]
    List,
}

impl std::hash::Hash for Schema {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Schema {}

#[derive(Clone, Default, Debug)]
pub struct List {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub fields: Vec<Arc<Field>>,
    pub actions: Vec<Action>,
    pub page_size: u32,
}

#[derive(Clone, Default, Debug)]
pub struct Form {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub sections: Vec<Section>,
    pub actions: Vec<Action>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Create,
    Save,
    Cancel,
    Modify,
    Delete,
    Search,
    Reload,
}

#[derive(Clone, Default, Debug)]
pub struct Section {
    pub title: Option<&'static str>,
    pub display: Vec<Eval>,
    pub fields: Vec<Arc<Field>>,
}

#[derive(Clone, Debug)]
pub enum Source<S, F> {
    Static(&'static [(&'static str, &'static str)]),
    StaticId(&'static [&'static str]),
    Dynamic {
        schema: S,
        field: F,
        filter: Value<&'static [&'static str]>,
    },
    DynamicSelf {
        field: F,
        filter: Value<&'static [&'static str]>,
    },
}

#[derive(Clone, Default, Debug)]
pub struct Value<T> {
    pub if_thens: Vec<IfThen<T>>,
    pub default: Option<T>,
}

#[derive(Clone, Debug)]
pub struct IfThen<T> {
    pub eval: Eval,
    pub value: T,
}

#[derive(Clone, Debug)]
pub struct Eval {
    pub field: Arc<Field>,
    pub values: Vec<&'static str>,
    pub condition: Condition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    Uppercase,
    HashSecret,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validator {
    Required,
    IsEmail,
    IsId,
    IsHost,
    IsDomain,
    IsPort,
    IsIpOrMask,
    IsUrl,
    IsRegex,
    IsSocketAddr,
    MinLength(usize),
    MaxLength(usize),
    MinValue(NumberType),
    MaxValue(NumberType),
    MinItems(usize),
    MaxItems(usize),
    IsValidExpression(ExpressionValidator),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ExpressionValidator {
    pub variables: &'static [&'static str],
    pub constants: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberType {
    Integer(i64),
    Float(f64),
}

impl Eq for NumberType {}

impl Schemas {
    pub fn get(&self, id: &str) -> Arc<Schema> {
        self.schemas.get(id).cloned().unwrap_or_else(|| {
            panic!("Schema {id:?} not found.");
        })
    }

    pub fn build_form(&self, id: &str) -> FormData {
        self.get(id).into()
    }
}

impl From<Arc<Schema>> for FormData {
    fn from(schema: Arc<Schema>) -> Self {
        FormData {
            values: Default::default(),
            errors: Default::default(),
            external_sources: Default::default(),
            schema,
            is_update: false,
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
        self.list.actions.contains(&action)
    }

    pub fn has_form_action(&self, action: Action) -> bool {
        self.form.actions.contains(&action)
    }

    pub fn unwrap_prefix(&self) -> &str {
        match self.typ {
            SchemaType::Record { prefix, .. } | SchemaType::Entry { prefix } => prefix,
            SchemaType::List => panic!("Schema type is not Record or Entry for {:?}.", self.id),
        }
    }

    pub fn try_unwrap_suffix(&self) -> Option<&str> {
        match self.typ {
            SchemaType::Record { suffix, .. } => Some(suffix),
            SchemaType::Entry { .. } | SchemaType::List => None,
        }
    }

    pub fn external_sources(&self) -> impl Iterator<Item = (Option<Arc<Schema>>, Arc<Field>)> + '_ {
        self.fields
            .values()
            .filter_map(|field_| match &field_.typ_ {
                Type::Select {
                    source: Source::Dynamic { schema, field, .. },
                    ..
                } => Some((schema.clone().into(), field.clone())),
                Type::Select {
                    source: Source::DynamicSelf { field, .. },
                    ..
                } => Some((None, field.clone())),
                _ => None,
            })
    }
}

impl Field {
    pub fn value(&self, settings: &FormData) -> String {
        settings
            .get(self.id)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn display(&self, settings: &FormData) -> bool {
        self.display.is_empty() || self.display.iter().any(|eval| eval.eval(settings))
    }

    pub fn placeholder(&self, settings: &FormData) -> Option<&str> {
        self.placeholder.eval(settings).copied()
    }

    pub fn default(&self, settings: &FormData) -> Option<&FormValue> {
        self.default.eval(settings)
    }

    pub fn input_check(&self, settings: &FormData) -> Option<&InputCheck> {
        self.checks.eval(settings)
    }

    pub fn is_required(&self, settings: &FormData) -> bool {
        matches!(self.typ_, Type::Boolean | Type::Select { .. })
            || self
                .input_check(settings)
                .map(|c| c.validators.contains(&Validator::Required))
                .unwrap_or_default()
    }

    pub fn is_multivalue(&self) -> bool {
        matches!(
            self.typ_,
            Type::Array
                | Type::Expression
                | Type::Select {
                    typ: SelectType::Many | SelectType::ManyWithSearch,
                    ..
                }
        )
    }
}

impl<T> Value<T> {
    pub fn eval(&self, settings: &FormData) -> Option<&T> {
        for if_then in &self.if_thens {
            if if_then.eval.eval(settings) {
                return Some(&if_then.value);
            }
        }

        self.default.as_ref()
    }
}

impl Eval {
    pub fn eval(&self, settings: &FormData) -> bool {
        let value = settings.get(self.field.id);
        match self.condition {
            Condition::MatchAny => self.values.iter().any(|v| value == Some(v)),
            Condition::MatchNone => self.values.iter().all(|v| value != Some(v)),
        }
    }
}

impl Section {
    pub fn display(&self, settings: &FormData) -> bool {
        self.display.is_empty() || self.display.iter().any(|eval| eval.eval(settings))
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
                        Action::Reload,
                    ],
                    page_size: 10,
                    ..Default::default()
                },
                form: Form {
                    actions: vec![Action::Save, Action::Cancel, Action::Reload],
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
        if matches!(self.item.typ, SchemaType::List) {
            self.item.typ = SchemaType::Entry { prefix };
        } else {
            panic!("Schema type is not List.");
        }
        self
    }

    pub fn suffix(mut self, suffix: &'static str) -> Self {
        match self.item.typ {
            SchemaType::Entry { prefix } => {
                self.item.typ = SchemaType::Record { prefix, suffix };
            }
            _ => panic!("Schema type is not Record."),
        }
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

    pub fn no_list_action(mut self, action: Action) -> Self {
        self.item.list.actions.retain(|a| *a != action);
        self
    }

    pub fn list_action(mut self, action: Action) -> Self {
        self.item.list.actions.push(action);
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

    pub fn form_action(mut self, action: Action) -> Self {
        self.item.form.actions.push(action);
        self
    }

    pub fn reload_prefix(mut self, prefix: &'static str) -> Self {
        self.item.reload_prefix = Some(prefix);
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

impl<T, I> Type<T, I> {
    pub fn label<'x>(&'x self, id: &'x str) -> &'x str {
        match self {
            Type::Select {
                source: Source::Static(items),
                ..
            } => items
                .iter()
                .find_map(|(k, v)| if *k == id { Some(*v) } else { None })
                .unwrap_or(id),
            _ => id,
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
        self.item.help = Some(help);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.item.readonly = true;
        self
    }

    pub fn enterprise_feature(mut self) -> Self {
        self.item.enterprise = true;
        self
    }

    pub fn typ(mut self, typ_: Type<&'static str, &'static str>) -> Self {
        self.item.typ_ = match typ_ {
            Type::Select {
                source:
                    Source::Dynamic {
                        schema,
                        field,
                        filter,
                    },
                typ,
            } => {
                let schema = self.schema(schema);

                Type::Select {
                    source: Source::Dynamic {
                        field: schema
                            .fields
                            .get(field)
                            .unwrap_or_else(|| {
                                panic!("Field {field:?} not found in schema {}.", schema.id)
                            })
                            .clone(),
                        schema,
                        filter,
                    },
                    typ,
                }
            }
            Type::Select {
                source: Source::DynamicSelf { field, filter },
                typ,
            } => Type::Select {
                source: Source::DynamicSelf {
                    field: self.field(field),
                    filter,
                },
                typ,
            },
            typ_ => typ_.into(),
        };
        self
    }

    pub fn source_filter_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        filters: &'static [&'static str],
    ) -> Self {
        let field = self.field(field);
        match &mut self.item.typ_ {
            Type::Select {
                source: Source::Dynamic { filter, .. } | Source::DynamicSelf { filter, .. },
                ..
            } => {
                filter.push_if_matches_eq(field, conditions, filters);
            }
            _ => panic!("Field type is not a dynamic source."),
        }
        self
    }

    pub fn source_filter(mut self, filters: &'static [&'static str]) -> Self {
        match &mut self.item.typ_ {
            Type::Select {
                source: Source::Dynamic { filter, .. } | Source::DynamicSelf { filter, .. },
                ..
            } => {
                filter.push_else(filters);
            }
            _ => panic!("Field type is not a dynamic source."),
        }
        self
    }

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

    pub fn placeholder_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        placeholder: &'static str,
    ) -> Self {
        self.item
            .placeholder
            .push_if_matches_eq(self.field(field), conditions, placeholder);
        self
    }

    pub fn placeholder(mut self, placeholder: &'static str) -> Self {
        self.item.placeholder.push_else(placeholder);
        self
    }

    pub fn default(mut self, default: impl Into<FormValue>) -> Self {
        self.item.default.push_else(default.into());
        self
    }

    pub fn default_if_eq(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        value: impl Into<FormValue>,
    ) -> Self {
        self.item
            .default
            .push_if_matches_eq(self.field(field), conditions, value.into());
        self
    }

    pub fn display_if(
        mut self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
        condition: Condition,
    ) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        if !values.is_empty() {
            self.item.display.push(Eval {
                field: self.field(field),
                values,
                condition,
            });
        }
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

impl ExpressionValidator {
    pub fn new(variables: &'static [&'static str], constants: &'static [&'static str]) -> Self {
        ExpressionValidator {
            variables,
            constants,
        }
    }

    pub fn constants(mut self, constants: &'static [&'static str]) -> Self {
        self.constants = constants;
        self
    }

    pub fn variables(mut self, variables: &'static [&'static str]) -> Self {
        self.variables = variables;
        self
    }
}

impl From<Type<&'static str, &'static str>> for Type<Arc<Schema>, Arc<Field>> {
    fn from(typ_: Type<&'static str, &'static str>) -> Self {
        match typ_ {
            Type::Boolean => Type::Boolean,
            Type::Duration => Type::Duration,
            Type::Expression => Type::Expression,
            Type::Input => Type::Input,
            Type::Array => Type::Array,
            Type::Secret => Type::Secret,
            Type::Text => Type::Text,
            Type::Size => Type::Size,
            Type::Cron => Type::Cron,
            Type::Rate => Type::Rate,
            Type::Select {
                source: Source::Static(items),
                typ,
            } => Type::Select {
                source: Source::Static(items),
                typ,
            },
            Type::Select {
                source: Source::StaticId(items),
                typ,
            } => Type::Select {
                source: Source::StaticId(items),
                typ,
            },
            Type::Select { .. } => unreachable!(),
        }
    }
}

impl From<i64> for NumberType {
    fn from(value: i64) -> Self {
        NumberType::Integer(value)
    }
}

impl From<f64> for NumberType {
    fn from(value: f64) -> Self {
        NumberType::Float(value)
    }
}
