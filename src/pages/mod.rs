/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
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
