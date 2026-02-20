// ============================================================================
// Core Protocol Types
// ============================================================================

/// Role of the protocol handler - server or client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolRole {
    /// Server role - accepts connections and handles requests
    Server,
    /// Client role - initiates connections and sends requests  
    Client,
}

/// Protocol index type for efficient registry lookups.
pub type ProtocolIndex = u16;
