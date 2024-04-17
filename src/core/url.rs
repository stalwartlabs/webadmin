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

use ahash::AHashMap;

pub struct UrlBuilder {
    pub path: String,
    pub params: AHashMap<&'static str, String>,
}

impl UrlBuilder {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            params: AHashMap::new(),
        }
    }

    pub fn prepend_path(&mut self, path: impl AsRef<str>) {
        self.path.insert_str(0, path.as_ref());
    }

    pub fn with_subpath(mut self, subpath: impl AsRef<str>) -> Self {
        self.path.push('/');
        self.path.push_str(
            &form_urlencoded::Serializer::new(String::new())
                .append_key_only(subpath.as_ref())
                .finish(),
        );
        self
    }

    pub fn with_parameter(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.params.insert(key, value.into());
        self
    }

    pub fn with_optional_parameter(
        mut self,
        key: &'static str,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            self.params.insert(key, value.into());
        }
        self
    }

    pub fn finish(self) -> String {
        if self.params.is_empty() {
            self.path
        } else {
            format!(
                "{}?{}",
                self.path,
                serde_urlencoded::to_string(&self.params).unwrap()
            )
        }
    }
}
