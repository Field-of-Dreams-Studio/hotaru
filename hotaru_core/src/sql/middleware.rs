//! Built-in SQL middleware
//!
//! This module provides common middleware for SQL operations like
//! logging, caching, and timeouts.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::app::middleware::AsyncMiddleware;
use crate::sql::SqlContext;

/// Query logger middleware
///
/// Logs all SQL queries and their execution time.
pub struct QueryLogger;

impl QueryLogger {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AsyncMiddleware<SqlContext> for QueryLogger {
    fn handle<'a>(
        &'a self,
        ctx: SqlContext,
        next: Box<dyn Fn(SqlContext) -> Pin<Box<dyn Future<Output = SqlContext> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = SqlContext> + Send + 'static>> {
        Box::pin(async move {
            let query = ctx.query.clone();
            println!("[SQL] Query: {}", query);

            let start = Instant::now();
            let ctx = next(ctx).await;
            let elapsed = start.elapsed();

            println!("[SQL] Query completed in {:?}", elapsed);
            ctx
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn return_self() -> Self
    where
        Self: Sized,
    {
        QueryLogger
    }
}

impl Default for QueryLogger {
    fn default() -> Self {
        Self
    }
}
