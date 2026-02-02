use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{BufReader, BufWriter};

use crate::client::{ConnectionTarget, Client};
use crate::connection::error::ConnectionError;
use crate::connection::{ProtocolRole, RequestContext};
use crate::extensions::{Locals, Params};
use crate::http::body::HttpBody;
use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::http::safety::HttpSafety;
use crate::http::traits::HTTP;
use crate::url::PathPattern;

pub struct HttpClientContext {
    pub request: HttpRequest,
    pub response: HttpResponse,
    pub client: Arc<Client>,
    pub safety: HttpSafety,
    pub url_patterns: Vec<PathPattern>,
    pub url_names: Vec<Option<String>>,
    pub url_params: HashMap<String, String>,
    pub params: Params,
    pub locals: Locals,
}

impl HttpClientContext {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(),
            client,
            safety: HttpSafety::default(),
            url_patterns: Vec::new(),
            url_names: Vec::new(),
            url_params: HashMap::new(),
            params: Params::new(),
            locals: Locals::new(),
        }
    }

    pub fn url_patterns(mut self, patterns: Vec<PathPattern>, names: Vec<Option<String>>) -> Self {
        self.url_patterns = patterns;
        self.url_names = names;
        self
    }

    pub fn set_url_patterns(&mut self, patterns: Vec<PathPattern>, names: Vec<Option<String>>) {
        self.url_patterns = patterns;
        self.url_names = names;
    }

    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.url_params.insert(key.into(), value.into());
        self
    }

    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self {
        self.params.set(value);
        self
    }

    pub fn get_config<V: Clone + Send + Sync + 'static>(&self) -> Option<V> {
        self.params.get::<V>().cloned()
    }

    pub fn set_local<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        let key = key.into();
        if let Some(s) = (&value as &dyn Any).downcast_ref::<&'static str>().copied() {
            self.locals.set(key.clone(), s);
            self.locals.set(key, s.to_string());
        } else {
            self.locals.set(key, value);
        }
        self
    }

    pub fn get_local<V: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<V> {
        self.locals.get::<V>(key).cloned()
    }

    pub fn request(mut self, request: HttpRequest) -> Self {
        self.request = request;
        self
    }

    pub fn safety(mut self, safety: HttpSafety) -> Self {
        self.safety = safety;
        self
    }

    pub fn build_url(&self) -> Result<String, String> {
        use crate::url::parser::substitute;

        if self.url_patterns.is_empty() {
            return Err("URL patterns not set".to_string());
        }

        let path = substitute(&self.url_patterns, &self.url_names, &self.url_params)?;

        if let Some(base_url) = &self.client.base_url {
            let base = base_url.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            Ok(format!("{}/{}", base, path))
        } else {
            Ok(path)
        }
    }

    pub async fn send(&mut self, url: &str) -> Result<(), ConnectionError> {
        let target =
            ConnectionTarget::<HTTP>::from_url(url).map_err(ConnectionError::Other)?;

        if self.request.meta.get_host().is_none() {
            if let Some(port) = target.port {
                self.request
                    .meta
                    .set_host(Some(format!("{}:{}", target.host, port)));
            } else {
                self.request.meta.set_host(Some(target.host.clone()));
            }
        }

        self.request.meta.start_line.set_path(target.path.clone());

        let connection = target
            .to_connection_builder()
            .map_err(ConnectionError::Other)?
            .connect()
            .await?;
        let (read, write) = connection.split();

        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(write);

        let request = std::mem::take(&mut self.request);
        HttpContext::write_frame(&mut writer, request).await?;
        let response = HttpContext::read_next_frame(&self.safety, &mut reader).await?;
        self.response = response;
        Ok(())
    }

    pub fn status_code(&self) -> u16 {
        self.response.meta.start_line.status_code().as_u16()
    }

    pub fn response_body(&self) -> &HttpBody {
        &self.response.body
    }
}

impl RequestContext for HttpClientContext {
    type Request = HttpRequest;
    type Response = HttpResponse;

    fn handle_error(&mut self) {
        self.response = HttpResponse::default();
    }

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Client
    }
}

#[cfg(test)]
mod test {
    use super::HttpClientContext;
    use crate::client::Client;

    #[test]
    fn test_http_client_context_url_building() {
        let client = Client::new()
            .name("test")
            .base_url("https://api.example.com")
            .build();

        let ctx = HttpClientContext::new(client)
            .url_patterns(
                vec![
                    crate::url::PathPattern::Literal("users".to_string()),
                    crate::url::PathPattern::Any,
                ],
                vec![None, Some("id".to_string())],
            )
            .param("id", "123");

        assert_eq!(
            ctx.build_url().unwrap(),
            "https://api.example.com/users/123"
        );
    }

    #[test]
    fn test_http_client_context_no_base_url() {
        let client = Client::new().name("test").build();

        let ctx = HttpClientContext::new(client).url_patterns(
            vec![crate::url::PathPattern::Literal("get".to_string())],
            vec![None],
        );

        assert_eq!(ctx.build_url().unwrap(), "/get");
    }

    #[test]
    fn test_http_client_context_params_locals() {
        let client = Client::new().name("test").build();

        let ctx = HttpClientContext::new(client)
            .set_config(crate::http::safety::HttpSafety::default())
            .set_local("auth_token", "xyz".to_string());

        assert!(ctx.get_config::<crate::http::safety::HttpSafety>().is_some());
        assert_eq!(ctx.get_local::<String>("auth_token").unwrap(), "xyz");
    }
}
