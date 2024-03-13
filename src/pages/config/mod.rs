pub mod edit;
pub mod list;
pub mod schema;

use crate::{
    components::{
        icon::{IconCircleStack, IconShieldCheck, IconUserGroup},
        layout::{LayoutBuilder, MenuItem},
    },
    core::{
        form::{FormData, FormValue},
        schema::*,
    },
};
use ahash::AHashMap;
use leptos::view;
use serde::{Deserialize, Serialize};

pub type Settings = AHashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UpdateSettings {
    Delete {
        keys: Vec<String>,
    },
    Clear {
        prefix: String,
    },
    Insert {
        prefix: Option<String>,
        values: Vec<(String, String)>,
        assert_empty: bool,
    },
}

impl FormData {
    pub fn build_update(&self) -> Vec<UpdateSettings> {
        let mut updates = Vec::new();
        let mut insert_prefix = None;
        let mut assert_empty = false;

        match &self.schema.typ {
            SchemaType::Record { prefix, .. } => {
                if self.is_update {
                    updates.push(UpdateSettings::Clear {
                        prefix: format!("{prefix}.{}.", self.value_as_str("_id").unwrap()),
                    });
                } else {
                    assert_empty = true;
                }

                insert_prefix = format!("{prefix}.{}", self.value_as_str("_id").unwrap()).into();
            }
            SchemaType::Entry { prefix } => {
                updates.push(UpdateSettings::Insert {
                    prefix: None,
                    assert_empty: !self.is_update,
                    values: vec![(
                        format!("{prefix}.{}", self.value_as_str("_id").unwrap()),
                        self.value_as_str("_value").unwrap_or_default().to_string(),
                    )],
                });
                return updates;
            }
            SchemaType::List => {
                if self.is_update {
                    let mut delete_keys = Vec::new();
                    for field in self.schema.fields.values() {
                        if field.is_multivalue() {
                            updates.push(UpdateSettings::Clear {
                                prefix: format!("{}.", field.id),
                            });
                            delete_keys.push(field.id.to_string());
                        } else if self.value_is_empty(field.id) {
                            delete_keys.push(field.id.to_string());
                        }
                    }

                    if !delete_keys.is_empty() {
                        updates.push(UpdateSettings::Delete { keys: delete_keys });
                    }
                }
            }
        }

        let mut key_values = Vec::new();
        for (key, value) in &self.values {
            if key.starts_with('_') {
                continue;
            }

            match value {
                FormValue::Value(value) if !value.is_empty() => {
                    key_values.push((key.to_string(), value.to_string()));
                }
                FormValue::Array(values) if !values.is_empty() => {
                    let total_values = values.len();
                    if total_values > 1 {
                        let pad_len = total_values.to_string().len();

                        for (idx, value) in values.iter().enumerate() {
                            key_values.push((format!("{key}.{idx:0>pad_len$}"), value.to_string()));
                        }
                    } else {
                        key_values.push((key.to_string(), values.first().unwrap().to_string()));
                    }
                }
                FormValue::Expression(expr) if !expr.is_empty() => unimplemented!(),
                _ => (),
            }
        }

        if !key_values.is_empty() {
            updates.push(UpdateSettings::Insert {
                prefix: insert_prefix,
                values: key_values,
                assert_empty,
            });
        }

        updates
    }
}

pub trait ArrayValues {
    fn array_values(&self, prefix: &str) -> impl Iterator<Item = (&str, &str)>;
}

impl ArrayValues for Settings {
    fn array_values(&self, key: &str) -> impl Iterator<Item = (&str, &str)> {
        let full_prefix = key;
        let prefix = format!("{key}.");

        self.iter().filter_map(move |(key, value)| {
            if key.starts_with(&prefix) || key == full_prefix {
                (key.as_str(), value.as_str()).into()
            } else {
                None
            }
        })
    }
}

impl LayoutBuilder {
    pub fn settings() -> Vec<MenuItem> {
        LayoutBuilder::new("/settings")
            .create("Stores")
            .icon(view! { <IconCircleStack/> })
            .route("/store")
            .insert()
            .create("Directories")
            .icon(view! { <IconUserGroup/> })
            .route("/directory")
            .insert()
            .create("SPAM Filter")
            .icon(view! { <IconShieldCheck/> })
            .create("Scores")
            .route("/spam-scores")
            .insert()
            .create("Free domains")
            .route("/spam-free")
            .insert()
            .insert()
            .menu_items
    }
}
