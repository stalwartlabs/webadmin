/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

pub mod account;
pub mod authorize;
pub mod config;
pub mod directory;
pub mod login;
pub mod manage;
pub mod notfound;
pub mod queue;
pub mod reports;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct List<T> {
    pub items: Vec<T>,
    pub total: u64,
}

pub fn maybe_plural(items: usize, singular: &str, plural: &str) -> String {
    if items == 1 {
        format!("{} {}", items, singular)
    } else {
        format!("{} {}", items, plural)
    }
}

pub trait FormatDateTime {
    fn format_date_time(&self) -> String;
    fn format_date(&self) -> String;
    fn format_time(&self) -> String;
}

impl FormatDateTime for DateTime<Utc> {
    fn format_date_time(&self) -> String {
        self.with_timezone(&Local)
            .format("%a, %d %b %Y %H:%M:%S")
            .to_string()
    }

    fn format_date(&self) -> String {
        self.with_timezone(&Local)
            .format("%a, %d %b %Y")
            .to_string()
    }

    fn format_time(&self) -> String {
        self.with_timezone(&Local).format("%H:%M:%S").to_string()
    }
}
