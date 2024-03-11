//pub mod edit;
pub mod list;
pub mod schema;

use crate::core::schema::*;
use ahash::AHashMap;
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
        prefix: String,
        values: Vec<(String, String)>,
    },
}
