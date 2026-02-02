use std::marker::PhantomData;

use http::Uri;

use crate::connection::{ConnectionBuilder, Protocol};

/// Connection details extracted from a URL.
pub struct ConnectionTarget<P: Protocol> {
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub use_tls: bool,
    _protocol: PhantomData<P>,
}

impl<P: Protocol> ConnectionTarget<P> {
    pub fn port(&self) -> Result<u16, String> {
        self.port
            .or_else(|| P::default_port(self.use_tls))
            .ok_or_else(|| "Port must be specified for this protocol".to_string())
    }

    pub fn to_connection_builder(&self) -> Result<ConnectionBuilder<P>, String> {
        let mut builder = ConnectionBuilder::<P>::new(&self.host).tls(self.use_tls);
        if let Some(port) = self.port {
            builder = builder.port(port);
        }
        Ok(builder)
    }

    pub fn from_url(url: &str) -> Result<Self, String> {
        let uri: Uri = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
        let scheme = uri
            .scheme_str()
            .ok_or_else(|| "URL must include a scheme".to_string())?;
        let authority = uri
            .authority()
            .ok_or_else(|| "URL must include an authority".to_string())?;

        let use_tls = match scheme {
            "https" | "wss" => true,
            "http" | "ws" => false,
            _ => return Err(format!("Unsupported scheme: {}", scheme)),
        };

        let host = authority.host().to_string();
        let port = authority.port_u16();
        let path = uri
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());

        Ok(Self {
            host,
            port,
            path,
            use_tls,
            _protocol: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use super::ConnectionTarget;
    use crate::http::traits::HTTP;

    #[test]
    fn test_connection_target_https() {
        let target =
            ConnectionTarget::<HTTP>::from_url("https://api.example.com:8443/v1/users").unwrap();
        assert_eq!(target.host, "api.example.com");
        assert_eq!(target.port(), Ok(8443));
        assert_eq!(target.path, "/v1/users");
        assert!(target.use_tls);
    }

    #[test]
    fn test_connection_target_default_port() {
        let target = ConnectionTarget::<HTTP>::from_url("https://api.example.com/users").unwrap();
        assert_eq!(target.port(), Ok(443));
    }

    #[test]
    fn test_connection_target_http_default_port() {
        let target = ConnectionTarget::<HTTP>::from_url("http://localhost/api").unwrap();
        assert_eq!(target.port(), Ok(80));
    }
}
