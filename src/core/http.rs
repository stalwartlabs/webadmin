/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use ahash::AHashMap;
use base64::{engine::general_purpose::STANDARD, Engine};
use gloo_net::http::{Headers, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{url::UrlBuilder, AccessToken};

pub struct HttpRequest {
    method: Method,
    url: UrlBuilder,
    headers: Headers,
    body: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Error(ManagementApiError),
    Data { data: T },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error")]
#[serde(rename_all = "camelCase")]
pub enum ManagementApiError {
    FieldAlreadyExists {
        field: String,
        value: String,
    },
    FieldMissing {
        field: String,
    },
    NotFound {
        item: String,
    },
    Unsupported {
        details: String,
    },
    AssertFailed,
    Other {
        details: String,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    Unauthorized,
    Forbidden,
    NotFound,
    TotpRequired,
    Network(String),
    Serializer { error: String, response: String },
    Server(ManagementApiError),
}

pub trait IntoUrlBuilder {
    fn into_url_builder(self) -> UrlBuilder;
}

pub type Result<T> = std::result::Result<T, Error>;

impl HttpRequest {
    pub fn new(method: Method, url: impl IntoUrlBuilder) -> Self {
        Self {
            method,
            url: url.into_url_builder(),
            headers: Headers::new(),
            body: None,
        }
    }

    pub fn get(url: impl IntoUrlBuilder) -> Self {
        Self::new(Method::GET, url)
    }

    pub fn post(url: impl IntoUrlBuilder) -> Self {
        Self::new(Method::POST, url)
    }

    pub fn put(url: impl IntoUrlBuilder) -> Self {
        Self::new(Method::PUT, url)
    }

    pub fn delete(url: impl IntoUrlBuilder) -> Self {
        Self::new(Method::DELETE, url)
    }

    pub fn patch(url: impl IntoUrlBuilder) -> Self {
        Self::new(Method::PATCH, url)
    }

    pub fn with_parameter(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.url = self.url.with_parameter(key, value);
        self
    }

    pub fn with_parameters(mut self, params: AHashMap<String, String>) -> Self {
        self.url = self.url.with_parameters(params);
        self
    }

    pub fn with_optional_parameter(
        mut self,
        key: &'static str,
        value: Option<impl Into<String>>,
    ) -> Self {
        self.url = self.url.with_optional_parameter(key, value);
        self
    }

    pub fn with_authorization(self, auth_token: impl AsRef<AccessToken>) -> Self {
        let auth_token = auth_token.as_ref();
        let mut result = self.with_header(
            "Authorization",
            format!("Bearer {}", auth_token.access_token),
        );
        if !auth_token.base_url.is_empty() {
            result.url.prepend_path(auth_token.base_url.as_str());
        }
        result
    }

    pub fn with_base_url(mut self, auth_token: impl AsRef<AccessToken>) -> Self {
        let auth_token = auth_token.as_ref();
        if !auth_token.base_url.is_empty() {
            self.url.prepend_path(auth_token.base_url.as_str());
        }
        self
    }

    pub fn with_basic_authorization(
        self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Self {
        self.with_header(
            "Authorization",
            format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()).as_bytes())
            ),
        )
    }

    pub fn with_header(self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers.set(name.as_ref(), value.as_ref());
        self
    }

    pub fn with_body<B: Serialize>(self, body: B) -> Result<Self> {
        match serde_json::to_string(&body) {
            Ok(body) => Ok(self
                .with_raw_body(body)
                .with_header("Content-Type", "application/json")),
            Err(err) => Err(Error::Serializer {
                error: err.to_string(),
                response: "".to_string(),
            }),
        }
    }

    pub fn with_raw_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub async fn send<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.send_raw().await?;
        match serde_json::from_slice::<Response<T>>(response.as_slice()) {
            Ok(Response::Data { data }) => Ok(data),
            Ok(Response::Error(error)) => Err(Error::Server(error)),
            Err(err) => Err(Error::Serializer {
                error: err.to_string(),
                response: String::from_utf8_lossy(&response).to_string(),
            }),
        }
    }

    pub async fn try_send<T>(self) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match self.send::<T>().await {
            Ok(data) => Ok(Some(data)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn send_raw(self) -> Result<Vec<u8>> {
        let abort_controller = web_sys::AbortController::new().ok();
        let abort_signal = abort_controller.as_ref().map(|a| a.signal());

        // abort in-flight requests if, e.g., we've navigated away from this page
        leptos::on_cleanup(move || {
            if let Some(abort_controller) = abort_controller {
                abort_controller.abort()
            }
        });

        let builder = RequestBuilder::new(&self.url.finish())
            .method(self.method)
            .headers(self.headers)
            .abort_signal(abort_signal.as_ref());
        let req = if let Some(body) = self.body {
            builder.body(body)
        } else {
            builder.build()
        }?;

        let response = req.send().await?;

        match response.status() {
            200..=299 => response.binary().await.map_err(Into::into),
            401 => Err(Error::Unauthorized),
            402 => Err(Error::TotpRequired),
            403 => Err(Error::Forbidden),
            404 => Err(Error::NotFound),
            code => Err(Error::Server(ManagementApiError::Other {
                details: format!("Invalid response code {code}"),
                reason: response.status_text().into(),
            })),
        }
    }
}

impl IntoUrlBuilder for String {
    fn into_url_builder(self) -> UrlBuilder {
        UrlBuilder::new(self)
    }
}

impl IntoUrlBuilder for &str {
    fn into_url_builder(self) -> UrlBuilder {
        UrlBuilder::new(self)
    }
}

impl IntoUrlBuilder for (&str, &str) {
    fn into_url_builder(self) -> UrlBuilder {
        UrlBuilder::new(self.0).with_subpath(self.1)
    }
}

impl IntoUrlBuilder for (&str, String) {
    fn into_url_builder(self) -> UrlBuilder {
        UrlBuilder::new(self.0).with_subpath(self.1)
    }
}

impl IntoUrlBuilder for (&str, &String) {
    fn into_url_builder(self) -> UrlBuilder {
        UrlBuilder::new(self.0).with_subpath(self.1)
    }
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Error::Network(format!("HTTP request failed: {err}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serializer {
            error: err.to_string(),
            response: String::new(),
        }
    }
}
