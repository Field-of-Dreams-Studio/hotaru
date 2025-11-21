# Connection Module Development Notes

## Migration from Rx/Tx to Protocol Trait System

### Current State (Before Migration)
- `ServerConnection<R: Rx>` - Handles server-side connections with Rx trait
- `ClientConnection<T: Tx>` - Handles client-side connections with Tx trait  
- `ConnectionStatus` - Enum tracking connection state
- Rx trait - Defines server-side protocol behavior
- Tx trait - Defines client-side protocol behavior

### Migration Strategy

#### Phase 1: Remove Old Connection Types
Since the Protocol trait now handles both server and client operations directly:
1. Remove `ServerConnection<R: Rx>` - No longer needed as protocols handle their own connections
2. Remove `ClientConnection<T: Tx>` - Protocol trait includes client functionality
3. Keep `ConnectionStatus` - Still useful for protocol state management

#### Phase 2: Clean Up Exports
1. Remove exports of Rx and Tx from connection module
2. Remove receive.rs and transmit.rs files completely
3. Update connection.rs to only export ConnectionStatus

#### Phase 3: Connection Status Design
ConnectionStatus remains important for protocol state tracking:
- `Established` - Initial connection state
- `Connected` - Active and processing frames
- `Stopped` - Connection should be closed
- `SwitchProtocol(TypeId)` - Protocol handoff required
- `Upgraded` - Just switched from another protocol

### Why This Change?

#### Problems with Rx/Tx Separation
1. **Artificial Split**: Server and client logic was unnecessarily separated
2. **Type Complexity**: Generic constraints made the API harder to use
3. **Protocol Switching**: Hard to implement with separate traits
4. **Multiplexing**: Difficult to add stream support with Rx/Tx model

#### Benefits of Protocol Trait
1. **Unified Interface**: One trait handles both server and client
2. **Better Abstraction**: Protocols define their own Transport/Stream/Message
3. **Cleaner Handoff**: Protocol switching is more natural
4. **Future Ready**: Easy to add WebSocket, HTTP/2, QUIC support

### Implementation Notes

#### ConnectionStatus Usage in Protocols
Protocols should use ConnectionStatus to signal state changes:
```rust
// In protocol implementation
if should_close {
    return Ok(ConnectionStatus::Stopped);
}
if switching_to_websocket {
    return Ok(ConnectionStatus::SwitchProtocol(TypeId::of::<WebSocketProtocol>()));
}
```

#### Protocol Registry Integration
The ProtocolRegistry uses ConnectionStatus to manage protocol lifecycle:
- Detects protocol switches via ConnectionStatus::SwitchProtocol
- Handles connection teardown on ConnectionStatus::Stopped
- Tracks upgrade path with ConnectionStatus::Upgraded

### Testing Considerations
1. Ensure HTTP/1.1 protocol works without Rx trait
2. Verify protocol switching still functions
3. Test that client connections work through Protocol trait
4. Validate ConnectionStatus transitions

### Security Implications
- No security model changes
- Protocol detection remains the same
- Connection lifecycle unchanged from security perspective

### Performance Considerations
- Fewer trait indirections (better performance)
- Direct protocol dispatch (reduced overhead)
- Simpler type system (faster compilation)

### Known Issues & TODOs
- [ ] Update all tests that use ServerConnection/ClientConnection
- [ ] Remove connection_1.rs backup file
- [ ] Update examples to use Protocol trait directly
- [ ] Document protocol implementation guide for users

### Migration Checklist
- [x] Create Protocol trait system
- [x] Add RequestContext trait for handler types
- [x] Implement HTTP 
- [x] Update ProtocolRegistry to use Protocol
- [x] Update App to use Protocol
- [x] Update Url to use Protocol
- [x] Update middleware to use Protocol
- [x] Remove Rx/Tx from HttpReqCtx
- [x] Remove ServerConnection/ClientConnection
- [x] Remove Rx/Tx trait files
- [x] Clean up connection module exports
- [ ] Merge HttpReqCtx and HttpResCtx into HttpContext
- [ ] Update HTTP to use unified HttpContext
- [ ] Update tests
- [ ] Update hotaru_meta macros