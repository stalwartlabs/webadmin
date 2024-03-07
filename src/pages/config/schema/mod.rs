pub mod common;
pub mod store;

use ahash::AHashMap;

use super::*;

#[derive(Default)]
pub struct Schemas {
    pub schemas: AHashMap<&'static str, Arc<Schema>>,
    pub sources: AHashMap<&'static str, Arc<Source>>,
}

pub struct Builder<P, I> {
    pub parent: P,
    pub item: I,
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
                ..Default::default()
            },
        }
    }

    pub fn with_source_static(
        mut self,
        id: &'static str,
        items: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        self.parent.sources.insert(
            id,
            Arc::new(Source::Static(
                items
                    .into_iter()
                    .map(|(value, label)| SelectItem { value, label })
                    .collect(),
            )),
        );
        self
    }

    pub fn with_source_dynamic(
        mut self,
        id: &'static str,
        schema: &'static str,
        field: &'static str,
    ) -> Self {
        let schema = self
            .parent
            .schemas
            .get(schema)
            .expect("Schema not found.")
            .clone();
        self.parent.sources.insert(
            id,
            Arc::new(Source::Dynamic {
                field: schema.fields.get(field).expect("Field not found.").clone(),
                schema,
            }),
        );
        self
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
    }

    pub fn new_form_section(self) -> Builder<(Schemas, Schema), Section> {
        Builder {
            parent: (self.parent, self.item),
            item: Section::default(),
        }
    }

    pub fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.item.prefix = Some(prefix);
        self
    }

    pub fn with_suffix(mut self, suffix: &'static str) -> Self {
        self.item.suffix = Some(suffix);
        self
    }

    pub fn with_list_title(mut self, title: &'static str) -> Self {
        self.item.list.title = title;
        self
    }

    pub fn with_list_subtitle(mut self, subtitle: &'static str) -> Self {
        self.item.list.subtitle = subtitle;
        self
    }

    pub fn with_list_field(mut self, field: &'static str) -> Self {
        self.item.list.fields.push(
            self.item
                .fields
                .get(field)
                .expect("Field not found.")
                .clone(),
        );
        self
    }

    pub fn with_list_fields(self, fields: impl IntoIterator<Item = &'static str>) -> Self {
        let mut builder = self;
        for field in fields {
            builder = builder.with_list_field(field);
        }
        builder
    }

    pub fn with_form_title(mut self, title: &'static str) -> Self {
        self.item.form.title = title;
        self
    }

    pub fn with_form_subtitle(mut self, subtitle: &'static str) -> Self {
        self.item.form.subtitle = subtitle;
        self
    }

    pub fn with_names(mut self, singular: &'static str, plural: &'static str) -> Self {
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
            .expect("Field not found.")
            .clone()
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
        self.item.label_column = label;
        self.item.label_form = label;
        self
    }

    pub fn with_label_column(mut self, label: &'static str) -> Self {
        self.item.label_column = label;
        self
    }

    pub fn with_label_form(mut self, label: &'static str) -> Self {
        self.item.label_form = label;
        self
    }

    pub fn with_help(mut self, help: &'static str) -> Self {
        self.item.help.push_else(help);
        self
    }

    pub fn with_help_if_any(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        value: &'static str,
    ) -> Self {
        self.item
            .help
            .push_if_matches_any(self.field(field), conditions, value);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.item.readonly = true;
        self
    }

    pub fn with_type_input(
        mut self,
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        self.item.typ_ = Type::Input {
            transformers: transformers.into_iter().collect(),
            validators: validators.into_iter().collect(),
        };
        self
    }

    pub fn with_type_multi_input(
        mut self,
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        self.item.typ_ = Type::InputMulti {
            transformers: transformers.into_iter().collect(),
            validators: validators.into_iter().collect(),
        };
        self
    }

    pub fn with_type_secret(
        mut self,
        transformers: impl IntoIterator<Item = Transformer>,
        validators: impl IntoIterator<Item = Validator>,
    ) -> Self {
        self.item.typ_ = Type::Secret {
            transformers: transformers.into_iter().collect(),
            validators: validators.into_iter().collect(),
        };
        self
    }

    pub fn with_type_select(mut self, source: &'static str) -> Self {
        self.item.typ_ = Type::Select(
            self.parent
                .0
                .sources
                .get(source)
                .expect("Source not found.")
                .clone(),
        );
        self
    }

    pub fn with_type_checkbox(mut self) -> Self {
        self.item.typ_ = Type::Checkbox;
        self
    }

    pub fn with_type_duration(mut self) -> Self {
        self.item.typ_ = Type::Duration;
        self
    }

    pub fn with_placeholder(mut self, placeholder: &'static str) -> Self {
        self.item.placeholder.push_else(placeholder);
        self
    }

    pub fn with_default(mut self, default: &'static str) -> Self {
        self.item.default.push_else(default);
        self.item.placeholder.push_else(default);
        self
    }

    pub fn with_default_if_any(
        mut self,
        field: &'static str,
        conditions: impl IntoIterator<Item = &'static str>,
        value: &'static str,
    ) -> Self {
        self.item
            .default
            .push_if_matches_any(self.field(field), conditions, value);
        self
    }

    pub fn with_display_if(
        mut self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
        condition: Condition,
    ) -> Self {
        self.item.display.push(Eval::Condition {
            field: self.field(field),
            values: values.into_iter().collect(),
            condition,
        });
        self
    }

    pub fn with_display_if_any(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.with_display_if(field, values, Condition::MatchAny)
    }

    pub fn with_display_if_none(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.with_display_if(field, values, Condition::MatchNone)
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
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.item.title = Some(title);
        self
    }

    pub fn with_field(mut self, field: &'static str) -> Self {
        self.item.fields.push(
            self.parent
                .1
                .fields
                .get(field)
                .expect("Field not found.")
                .clone(),
        );
        self
    }

    pub fn with_fields(self, fields: impl IntoIterator<Item = &'static str>) -> Self {
        let mut builder = self;
        for field in fields {
            builder = builder.with_field(field);
        }
        builder
    }

    fn with_display_if(
        mut self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
        condition: Condition,
    ) -> Self {
        self.item.display.push(Eval::Condition {
            field: self
                .parent
                .1
                .fields
                .get(field)
                .expect("Field not found.")
                .clone(),
            values: values.into_iter().collect(),
            condition,
        });
        self
    }

    pub fn with_display_if_any(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.with_display_if(field, values, Condition::MatchAny)
    }

    pub fn with_display_if_none(
        self,
        field: &'static str,
        values: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.with_display_if(field, values, Condition::MatchNone)
    }

    pub fn build(mut self) -> Builder<Schemas, Schema> {
        self.parent.1.form.sections.push(self.item);
        Builder {
            parent: self.parent.0,
            item: self.parent.1,
        }
    }
}

impl Value {
    pub fn push_if_matches_any(
        &mut self,
        field: Arc<Field>,
        contains: impl IntoIterator<Item = &'static str>,
        then: &'static str,
    ) {
        self.if_thens.push(IfThen {
            eval: Eval::Condition {
                field,
                values: contains.into_iter().collect(),
                condition: Condition::MatchAny,
            },
            value: then,
        });
    }

    pub fn push_if_matches_none(
        &mut self,
        field: Arc<Field>,
        contains: impl IntoIterator<Item = &'static str>,
        then: &'static str,
    ) {
        self.if_thens.push(IfThen {
            eval: Eval::Condition {
                field,
                values: contains.into_iter().collect(),
                condition: Condition::MatchNone,
            },
            value: then,
        });
    }

    pub fn push_else(&mut self, then: &'static str) {
        self.if_thens.push(IfThen {
            eval: Eval::True,
            value: then,
        });
    }
}
