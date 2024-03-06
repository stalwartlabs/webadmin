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
