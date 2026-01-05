use hotaru::prelude::*; 
use hotaru::http::*;  

use crate::APP;

// ============================================================================
// Middleware definitions
// ============================================================================

middleware! {
    /// Logs the start of request processing
    pub LogStart <HTTP> {
        println!("Middleware: Received request for {}, start processing", req.path());
        next(req).await
    }
}

middleware! {
    /// Logs the end of request processing
    pub fn LogEnd(req: HTTP) {
        let path = req.path().to_owned();
        let result = next(req).await;
        println!("Middleware: Processed request for {}, end processing", path);
        result
    }
}

middleware! {
    /// Can short-circuit the request
    pub ShortCircuit <HTTP> {
        if req.path() == "/directly_return" {
            req.response = text_response("Directly returned from middleware");
            req
        } else {
            next(req).await
        }
    }
}

middleware! {
    /// Sets values in locals and params
    pub SetValues <HTTP> {
        println!("Middleware: Setting values for {}", req.path());
        req.locals.set("some_value", 42i32);
        req.params.set(true);
        next(req).await
    }
}

middleware! {
    /// Reads values from locals and params
    pub ReadValues <HTTP> {
        let mut result = next(req).await;
        let value = result.locals.take::<i32>("some_value").unwrap_or(0);
        let param = result.params.take::<bool>().unwrap_or(false);
        println!("Middleware: Read local: {}, params: {}", value, param);
        result
    }
}

// ============================================================================
// Endpoints with middleware
// ============================================================================

endpoint! {
    APP.url("/with_middleware"),
    middleware = [LogStart, LogEnd],
    
    /// Endpoint with logging middleware
    pub with_middleware <HTTP> {
        text_response("This endpoint has middleware")
    }
}

endpoint! {
    APP.url("/with_values"),
    middleware = [SetValues, ReadValues],
    
    /// Endpoint that uses values from middleware
    pub with_values <HTTP> {
        let value = req.locals.get::<i32>("some_value").unwrap_or(&0);
        let param = req.params.get::<bool>().unwrap_or(&false);
        text_response(format!("Values - local: {}, param: {}", value, param))
    }
}

endpoint! {
    APP.url("/directly_return"),
    middleware = [ShortCircuit],
    
    /// This endpoint will be short-circuited by middleware
    pub directly_return <HTTP> {
        text_response("This should not be reached")
    }
}