//! TLS transport layer for the Hotaru framework.
//!
//! Provides TLS stream implementations that plug into `hotaru_core`'s
//! `ConnStream` / `TransportSpec` abstractions, plus flexible types for
//! runtime TCP/TLS selection.
//!
//! # Module layout
//!
//! - `tls/` — `TlsStream`, `TlsAccepter`, `TlsConnector`, `TlsTransport` (TLS-only)
//! - `config/` — `TlsConfig` (server) and `TlsClientConfig` (client) builders
//! - `flexible/` — `TcpOrTlsStream`, `ConnectionBuilder` (runtime TCP-or-TLS choice)

pub mod config;
pub mod flexible;
pub mod tls;

// ── TLS stream layer ──────────────────────────────────────────────────────────
pub use tls::{
    TlsAccepter, TlsAccepterError, TlsConnector, TlsConnectorError, TlsInbound, TlsInboundTarget,
    TlsMeta, TlsOutbound, TlsOutboundTarget, TlsStream, TlsTransport,
};

// ── Configuration builders ────────────────────────────────────────────────────
pub use config::{
    ClientAuth, TlsClientConfig, TlsClientConfigBuilder, TlsClientConfigError, TlsConfig,
    TlsConfigBuilder, TlsConfigError,
};

// ── Flexible TCP/TLS (runtime transport selection) ───────────────────────────
pub use flexible::{
    Authentication, ConnectionBuilder, FlexMeta, Protocol, TcpOrTlsStream, TcpReader, TcpWriter,
    split_connection,
};
