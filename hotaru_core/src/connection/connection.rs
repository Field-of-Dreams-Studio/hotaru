use std::any::TypeId;

/// Connection status enum used by protocols to signal state changes
/// 
/// Protocols return this from their handle methods to indicate:
/// - Whether to continue processing frames
/// - If a protocol switch is needed
/// - When the connection should be closed
pub enum ConnectionStatus { 
    /// Initial connection state, not yet processed
    Established, 
    
    /// Connection upgraded from another protocol
    Upgraded, 
    
    /// Active connection, ready for frame processing
    Connected, 
    
    /// Connection should be terminated
    Stopped, 
    
    /// Request to switch to a different protocol
    /// Contains the TypeId of the target protocol
    SwitchProtocol(TypeId)
}

impl ConnectionStatus { 
    /// Mark that a frame has been successfully processed
    /// Transitions from Established to Connected on first frame
    pub fn frame_passed(&mut self) { 
        match self {
            ConnectionStatus::Established => {
                *self = ConnectionStatus::Connected;
            },
            _ => {}
        }
    }

    /// Check if connection is in active state
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    /// Check if connection should be terminated
    pub fn is_stopped(&self) -> bool {
        matches!(self, ConnectionStatus::Stopped)
    }

    /// Get protocol TypeId if switching is requested
    pub fn should_switch(&self) -> Option<TypeId> {
        match self {
            ConnectionStatus::SwitchProtocol(type_id) => Some(*type_id),
            _ => None
        }
    }
    
    /// Check if connection was just upgraded
    pub fn is_upgraded(&self) -> bool {
        matches!(self, ConnectionStatus::Upgraded)
    }
}