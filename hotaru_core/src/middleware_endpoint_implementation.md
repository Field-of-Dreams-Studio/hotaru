# Full Middleware in Endpoint Implementation

## Overview
This document provides the complete implementation design for middleware functionality at the endpoint level in the Hotaru framework. The implementation enables middleware chains to be attached at multiple levels: Protocol, URL routes, and individual endpoints, with proper inheritance and execution order.

## Architecture Components

### 1. Core Middleware Types

```rust
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

/// A boxed future returning a context
pub type BoxFuture<C> = Pin<Box<dyn Future<Output = C> + Send + 'static>>;

/// Middleware chain type for storing multiple middlewares
pub type AsyncMiddlewareChain<C> = Vec<Arc<dyn AsyncMiddleware<C>>>;

/// Core middleware trait that all middlewares must implement
pub trait AsyncMiddleware<C: RequestContext>: Send + Sync + 'static {
    /// Allow downcasting for type checking
    fn as_any(&self) -> &dyn Any;

    /// Factory method for creating instances
    fn return_self() -> Self where Self: Sized;

    /// Main handler that processes requests
    fn handle(
        &self,
        ctx: C,
        next: Box<dyn Fn(C) -> Pin<Box<dyn Future<Output = C> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = C> + Send + 'static>>;
}
```

### 2. Endpoint Structure with Middleware Support

```rust
pub struct Url<C: RequestContext> {
    // Path pattern for this URL segment
    pub path: PathPattern,

    // Child URL segments
    pub children: RwLock<Children<C>>,

    // Parent URL or App reference
    pub ancestor: RwLock<Ancestor<C>>,

    // Final handler for this endpoint
    pub method: RwLock<Option<Arc<dyn AsyncFinalHandler<C>>>>,

    // Middleware chain specific to this endpoint
    pub middlewares: RwLock<Vec<Arc<dyn AsyncMiddleware<C>>>>,

    // Configuration parameters
    pub params: RwLock<ParamsClone>,

    // Named segments for parameter extraction
    pub names: StepName,
}
```

### 3. Middleware Inheritance Implementation

```rust
impl<C: RequestContext + 'static> Url<C> {
    /// Collect all middlewares from ancestors up to the root (App or Protocol)
    /// Returns middlewares in execution order (root first, then descendants)
    pub fn collect_inherited_middlewares(&self) -> Vec<Arc<dyn AsyncMiddleware<C>>> {
        let mut middleware_stack = Vec::new();

        // Recursive helper to collect from ancestors
        fn collect_from_ancestors<C: RequestContext>(
            url: &Url<C>,
            stack: &mut Vec<Arc<dyn AsyncMiddleware<C>>>
        ) {
            match &*url.ancestor.read().unwrap() {
                Ancestor::Some(parent) => {
                    // Recursively collect from parent first
                    collect_from_ancestors(&**parent, stack);
                    // Then add parent's middlewares
                    let parent_middlewares = parent.middlewares.read().unwrap().clone();
                    stack.extend(parent_middlewares);
                }
                Ancestor::App(app) => {
                    // Get protocol-level middlewares from the app
                    if let Some(protocol_middlewares) = app.get_protocol_middlewares::<C>() {
                        stack.extend(protocol_middlewares);
                    }
                }
                Ancestor::Nil => {
                    // Base case - no more ancestors
                }
            }
        }

        collect_from_ancestors(self, &mut middleware_stack);

        // Finally add this URL's own middlewares
        let own_middlewares = self.middlewares.read().unwrap().clone();
        middleware_stack.extend(own_middlewares);

        middleware_stack
    }

    /// Run endpoint with full middleware chain including inherited middlewares
    pub async fn run_with_inheritance(&self, mut ctx: C) -> C {
        let final_handler = {
            let guard = self.method.read().unwrap();
            guard.clone()
        };

        // Collect full middleware chain including inherited ones
        let full_middleware_chain = self.collect_inherited_middlewares();

        // Execute with full middleware chain
        if let Some(method) = final_handler {
            run_chain(full_middleware_chain, method, ctx).await
        } else {
            ctx.handle_error();
            ctx
        }
    }
}
```

### 4. Protocol-Level Middleware Integration

```rust
impl ProtocolRegistryKind {
    /// Get protocol-level middleware for inheritance
    pub fn get_protocol_middlewares<P: Protocol + 'static>(&self)
        -> Vec<Arc<dyn AsyncMiddleware<P::Context>>>
    {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                if let Some(protocol_handler) =
                    handler.as_any().downcast_ref::<ProtocolHandler<P>>()
                {
                    protocol_handler.middlewares.clone()
                } else {
                    vec![]
                }
            }
            ProtocolRegistryKind::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(protocol_handler) =
                        handler.as_any().downcast_ref::<ProtocolHandler<P>>()
                    {
                        return protocol_handler.middlewares.clone();
                    }
                }
                vec![]
            }
        }
    }
}
```

### 5. Endpoint Builder with Middleware Support

```rust
impl<C: RequestContext + 'static> Url<C> {
    /// Create a new sub-URL with optional middleware
    pub fn sub_url_with_middleware<A: AsRef<str>>(
        self: Arc<Self>,
        pattern: A,
        handler: Option<Arc<dyn AsyncFinalHandler<C>>>,
        middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
        params: ParamsClone,
    ) -> Result<Arc<Url<C>>, String> {
        let segments = parse(pattern.as_ref())?;

        // Build the URL tree
        let mut current = self;
        for (i, segment) in segments.iter().enumerate() {
            if i == segments.len() - 1 {
                // Last segment - attach handler and middlewares
                current = current.register_child(
                    segment.clone(),
                    handler.clone(),
                    middlewares.clone(),
                    params.clone()
                )?;
            } else {
                // Intermediate segment
                current = current.register_child(
                    segment.clone(),
                    None,
                    vec![],
                    ParamsClone::default()
                )?;
            }
        }

        Ok(current)
    }

    /// Add middleware to an existing endpoint
    pub fn add_middleware(&self, middleware: Arc<dyn AsyncMiddleware<C>>) {
        let mut guard = self.middlewares.write().unwrap();
        guard.push(middleware);
    }

    /// Replace entire middleware chain
    pub fn set_middlewares(&self, middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>) {
        let mut guard = self.middlewares.write().unwrap();
        *guard = middlewares;
    }

    /// Clear all middlewares from this endpoint
    pub fn clear_middlewares(&self) {
        let mut guard = self.middlewares.write().unwrap();
        guard.clear();
    }
}
```

## Usage Examples

### 1. Basic Endpoint with Middleware

```rust
// Create an endpoint with logging middleware
let endpoint = root_url
    .sub_url_with_middleware(
        "/api/users",
        Some(Arc::new(user_handler)),
        vec![
            Arc::new(LoggingMiddleware::new()),
            Arc::new(AuthMiddleware::new()),
        ],
        ParamsClone::default()
    )?;
```

### 2. Middleware Inheritance Example

```rust
// Protocol level - applies to all HTTP endpoints
let http_protocol = HttpProtocol::new()
    .with_middleware(Arc::new(CorsMiddleware::new()))
    .with_middleware(Arc::new(CompressionMiddleware::new()));

// Route level - applies to all /api/* endpoints
let api_route = root_url
    .sub_url_with_middleware(
        "/api",
        None,
        vec![
            Arc::new(RateLimitMiddleware::new()),
            Arc::new(ApiKeyMiddleware::new()),
        ],
        ParamsClone::default()
    )?;

// Endpoint level - specific to /api/users
let users_endpoint = api_route
    .sub_url_with_middleware(
        "/users",
        Some(Arc::new(users_handler)),
        vec![
            Arc::new(ValidationMiddleware::new()),
            Arc::new(CacheMiddleware::new()),
        ],
        ParamsClone::default()
    )?;

// When a request hits /api/users, middlewares execute in order:
// 1. CorsMiddleware (protocol)
// 2. CompressionMiddleware (protocol)
// 3. RateLimitMiddleware (route)
// 4. ApiKeyMiddleware (route)
// 5. ValidationMiddleware (endpoint)
// 6. CacheMiddleware (endpoint)
// 7. users_handler (final handler)
```

### 3. Custom Middleware Implementation

```rust
pub struct TimingMiddleware {
    name: String,
}

impl TimingMiddleware {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl AsyncMiddleware<HttpReqCtx> for TimingMiddleware {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn return_self() -> Self where Self: Sized {
        TimingMiddleware::new("timing")
    }

    fn handle(
        &self,
        ctx: HttpReqCtx,
        next: Box<dyn Fn(HttpReqCtx) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send + 'static>> {
        let name = self.name.clone();
        Box::pin(async move {
            let start = std::time::Instant::now();

            // Call next middleware/handler
            let ctx = next(ctx).await;

            let duration = start.elapsed();
            println!("[{}] Request took: {:?}", name, duration);

            ctx
        })
    }
}
```

### 4. Dynamic Middleware Management

```rust
// Add middleware to existing endpoint at runtime
let endpoint = root_url.walk_str("/api/users").await;
endpoint.add_middleware(Arc::new(NewMiddleware::new()));

// Replace entire middleware chain
endpoint.set_middlewares(vec![
    Arc::new(Middleware1::new()),
    Arc::new(Middleware2::new()),
]);

// Clear all middlewares
endpoint.clear_middlewares();
```

## Execution Flow

### Request Processing Pipeline

1. **Request Arrival**: Request arrives at the connection handler
2. **Protocol Detection**: System detects protocol (HTTP, WebSocket, etc.)
3. **URL Routing**: Walk the URL tree to find matching endpoint
4. **Middleware Collection**: Collect all middlewares in inheritance order:
   - Protocol-level middlewares (from ProtocolHandler)
   - Route-level middlewares (from parent URLs)
   - Endpoint-level middlewares (from the endpoint itself)
5. **Chain Execution**: Execute middleware chain with final handler
6. **Response Return**: Return processed response through middleware chain

### Middleware Chain Execution Order

```
Request
  ↓
Protocol Middleware 1
  ↓
Protocol Middleware 2
  ↓
Route Middleware 1
  ↓
Route Middleware 2
  ↓
Endpoint Middleware 1
  ↓
Endpoint Middleware 2
  ↓
Final Handler
  ↓
Endpoint Middleware 2 (return)
  ↓
Endpoint Middleware 1 (return)
  ↓
Route Middleware 2 (return)
  ↓
Route Middleware 1 (return)
  ↓
Protocol Middleware 2 (return)
  ↓
Protocol Middleware 1 (return)
  ↓
Response
```

## Performance Considerations

### Middleware Caching
- Middleware chains are cloned as Arc references (cheap clones)
- Consider caching collected middleware chains for frequently accessed endpoints
- Use RwLock for concurrent read access to middleware vectors

### Optimization Strategies

```rust
impl<C: RequestContext + 'static> Url<C> {
    /// Cached middleware chain to avoid repeated collection
    cached_chain: RwLock<Option<Vec<Arc<dyn AsyncMiddleware<C>>>>>,

    /// Get or compute cached middleware chain
    pub fn get_cached_middleware_chain(&self) -> Vec<Arc<dyn AsyncMiddleware<C>>> {
        let guard = self.cached_chain.read().unwrap();
        if let Some(chain) = &*guard {
            return chain.clone();
        }
        drop(guard);

        // Compute and cache
        let chain = self.collect_inherited_middlewares();
        let mut guard = self.cached_chain.write().unwrap();
        *guard = Some(chain.clone());
        chain
    }

    /// Invalidate cache when middlewares change
    pub fn invalidate_middleware_cache(&self) {
        let mut guard = self.cached_chain.write().unwrap();
        *guard = None;
    }
}
```

## Error Handling

### Middleware Error Propagation

```rust
pub struct ErrorHandlingMiddleware;

impl<C: RequestContext> AsyncMiddleware<C> for ErrorHandlingMiddleware {
    fn handle(
        &self,
        mut ctx: C,
        next: Box<dyn Fn(C) -> Pin<Box<dyn Future<Output = C> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = C> + Send + 'static>> {
        Box::pin(async move {
            // Wrap next call in error handling
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| next(ctx))) {
                Ok(future) => {
                    // Execute the future
                    future.await
                }
                Err(_) => {
                    // Handle panic
                    ctx.set_error("Internal middleware error");
                    ctx
                }
            }
        })
    }
}
```

## Testing

### Unit Testing Middleware

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_middleware_inheritance() {
        // Create URL hierarchy
        let root = Arc::new(Url::default());
        root.add_middleware(Arc::new(Middleware1::new()));

        let child = root.sub_url("/child", None, vec![
            Arc::new(Middleware2::new())
        ], ParamsClone::default()).unwrap();

        // Collect middlewares
        let chain = child.collect_inherited_middlewares();

        // Verify order
        assert_eq!(chain.len(), 2);
        assert!(chain[0].as_any().is::<Middleware1>());
        assert!(chain[1].as_any().is::<Middleware2>());
    }

    #[tokio::test]
    async fn test_middleware_execution_order() {
        let mut order = Vec::new();
        let order_ref = Arc::new(RwLock::new(&mut order));

        // Create middlewares that record execution order
        // ... test implementation
    }
}
```

## Conclusion

This implementation provides a complete, flexible middleware system for endpoints in the Hotaru framework with:

- **Multi-level middleware attachment** (Protocol, Route, Endpoint)
- **Proper inheritance and execution order**
- **Dynamic middleware management**
- **Performance optimizations**
- **Comprehensive error handling**
- **Type-safe implementation**

The system is designed to be extensible, allowing new middleware types and patterns to be added without modifying the core implementation.