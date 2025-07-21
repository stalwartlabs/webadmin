/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::borrow::Cow;

use ahash::AHashMap;

pub struct UrlBuilder {
    pub path: String,
    pub params: AHashMap<Cow<'static, str>, String>,
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

    pub fn with_optional_subpath(self, subpath: Option<impl AsRef<str>>) -> Self {
        if let Some(subpath) = subpath {
            self.with_subpath(subpath)
        } else {
            self
        }
    }

    pub fn with_parameter(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<String>,
    ) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn with_optional_parameter(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            self.params.insert(key.into(), value.into());
        }
        self
    }

    pub fn with_parameters(mut self, params: AHashMap<String, String>) -> Self {
        self.params
            .extend(params.into_iter().map(|(k, v)| (k.into(), v)));
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
