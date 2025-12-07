use hotaru::prelude::*;
use hotaru::http::*;

// Use the main APP and global middleware from main.rs
use crate::{APP, GlobalLogger, GlobalMetrics};

// ============================================================================
// Local Middleware 
// ============================================================================

middleware! {
    /// Local auth middleware
    pub LocalAuth <HTTP> {
        println!("🔐 LocalAuth: Checking authentication");
        next(req).await
    }
}

middleware! {
    /// Local cache middleware
    pub LocalCache <HTTP> {
        println!("💾 LocalCache: Checking cache");
        let result = next(req).await;
        println!("💾 LocalCache: Updating cache");
        result
    }
}

// ============================================================================
// Test Endpoints with .. Pattern
// ============================================================================

endpoint! {
    APP.url("/test-global-first"),
    middleware = [.., LocalAuth],
    
    /// Test: Global middleware first, then local
    pub test_global_first <HTTP> {
        println!("🎯 Handler: test_global_first executing");
        text_response("Global first test completed")
    }
}

endpoint! {
    APP.url("/test-local-first"),
    middleware = [LocalAuth, ..],
    
    /// Test: Local middleware first, then global
    pub test_local_first <HTTP> {
        println!("🎯 Handler: test_local_first executing");
        text_response("Local first test completed")
    }
}

endpoint! {
    APP.url("/test-sandwich"),
    middleware = [LocalAuth, .., LocalCache],
    
    /// Test: Local, then global, then local again
    pub test_sandwich <HTTP> {
        println!("🎯 Handler: test_sandwich executing");
        text_response("Sandwich test completed")
    }
}

endpoint! {
    APP.url("/test-only-global"),
    middleware = [..],
    
    /// Test: Only inherited global middleware
    pub test_only_global <HTTP> {
        println!("🎯 Handler: test_only_global executing");
        text_response("Only global test completed")
    }
}

endpoint! {
    APP.url("/test-no-inheritance"),
    middleware = [LocalAuth, LocalCache],
    
    /// Test: No inheritance, only local middleware
    pub test_no_inheritance <HTTP> {
        println!("🎯 Handler: test_no_inheritance executing");
        text_response("No inheritance test completed")
    }
}