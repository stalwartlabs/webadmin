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

pub struct UrlBuilder {
    pub url: form_urlencoded::Serializer<'static, String>,
}

impl UrlBuilder {
    pub fn new(url: impl AsRef<str>) -> Self {
        let url = url.as_ref();
        let url = if !url.ends_with('?') {
            format!("{url}?")
        } else {
            url.to_string()
        };
        let url_len = url.len();
        Self {
            url: form_urlencoded::Serializer::for_suffix(url, url_len),
        }
    }

    pub fn with_parameter(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.url.append_pair(key.as_ref(), value.as_ref());
        self
    }

    pub fn with_optional_parameter(
        mut self,
        key: impl AsRef<str>,
        value: Option<impl AsRef<str>>,
    ) -> Self {
        if let Some(value) = value {
            self.url.append_pair(key.as_ref(), value.as_ref());
        }
        self
    }

    pub fn finish(mut self) -> String {
        self.url.finish()
    }
}
