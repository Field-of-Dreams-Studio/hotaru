//! Chapter 2: Advanced Routing and State Management
//! 
//! This example demonstrates CRUD operations, JSON handling with Akari,
//! and state management from the QUICK_TUTORIAL.md

use hotaru::prelude::*;
use hotaru::http::*;
use std::sync::RwLock;
use std::collections::HashMap;

// Simple in-memory user store
static USERS: Lazy<RwLock<HashMap<u32, User>>> = Lazy::new(|| {
    let mut users = HashMap::new();
    users.insert(1, User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    });
    users.insert(2, User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    });
    RwLock::new(users)
});

static NEXT_ID: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(3));

#[derive(Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

impl User {
    fn to_json(&self) -> Value {
        object!({
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone()
        })
    }
}

// Define your application
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3001")
        .build()
});

// List all users
endpoint! {
    APP.url("/api/users"),
    
    /// Get all users
    /// Returns a JSON array of user objects
    pub list_users <HTTP> {
        let users = USERS.read().unwrap();
        let users_array: Vec<Value> = users
            .values()
            .map(|user| user.to_json())
            .collect();
        
        json_response(object!({
            users: users_array,
            count: users.len()
        }))
    }
}

// Get a single user by ID
endpoint! {
    APP.url("/api/users/{id}"),
    
    /// Get a specific user by ID
    /// Returns 404 if user not found
    pub get_user <HTTP> {
        let id = req.param("id").string().parse::<u32>().unwrap_or(0);
        
        let users = USERS.read().unwrap();
        match users.get(&id) {
            Some(user) => json_response(user.to_json()),
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND)
                    .content_type(HttpContentType::Json);
                response.body = serde_json::to_string(&object!({
                    error: "User not found",
                    id: id
                })).unwrap().into();
                response
            }
        }
    }
}

// Create a new user
endpoint! {
    APP.url("/api/users/create"),
    
    /// Create a new user
    /// Expects JSON body with name and email fields
    pub create_user <HTTP> {
        if req.method() != HttpMethod::POST {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::METHOD_NOT_ALLOWED);
            response.body = "Only POST method allowed".into();
            return response;
        }
        
        // Parse JSON from request body
        let json_data = match req.json().await {
            Some(data) => data,
            None => return json_response(object!({
                error: "Invalid or missing JSON body"
            }))
        };
        
        // Extract fields using Akari's Value methods
        let name = json_data.get("name").string();
        let email = json_data.get("email").string();
        
        if name.is_empty() || email.is_empty() {
            return json_response(object!({
                error: "Name and email are required"
            }));
        }
        
        // Create new user
        let mut id_lock = NEXT_ID.write().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        drop(id_lock);
        
        let user = User {
            id,
            name: name.to_string(),
            email: email.to_string(),
        };
        
        let user_json = user.to_json();
        USERS.write().unwrap().insert(id, user);
        
        json_response(object!({
            message: "User created successfully",
            user: user_json
        }))
    }
}

// Update a user
endpoint! {
    APP.url("/api/users/{id}/update"),
    
    /// Update an existing user
    /// Accepts partial updates (only provided fields are updated)
    pub update_user <HTTP> {
        if req.method() != HttpMethod::PUT && req.method() != HttpMethod::PATCH {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::METHOD_NOT_ALLOWED);
            response.body = "Only PUT or PATCH methods allowed".into();
            return response;
        }
        
        let id = req.param("id").string().parse::<u32>().unwrap_or(0);
        
        let json_data = match req.json().await {
            Some(data) => data,
            None => return json_response(object!({
                error: "Invalid or missing JSON body"
            }))
        };
        
        let mut users = USERS.write().unwrap();
        match users.get_mut(&id) {
            Some(user) => {
                // Update fields if provided
                let new_name = json_data.get("name").string();
                if !new_name.is_empty() {
                    user.name = new_name.to_string();
                }
                
                let new_email = json_data.get("email").string();
                if !new_email.is_empty() {
                    user.email = new_email.to_string();
                }
                
                json_response(object!({
                    message: "User updated successfully",
                    user: user.to_json()
                }))
            },
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND)
                    .content_type(HttpContentType::Json);
                response.body = serde_json::to_string(&object!({
                    error: "User not found",
                    id: id
                })).unwrap().into();
                response
            }
        }
    }
}

// Delete a user
endpoint! {
    APP.url("/api/users/{id}/delete"),
    
    /// Delete a user by ID
    pub delete_user <HTTP> {
        if req.method() != HttpMethod::DELETE {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::METHOD_NOT_ALLOWED);
            response.body = "Only DELETE method allowed".into();
            return response;
        }
        
        let id = req.param("id").string().parse::<u32>().unwrap_or(0);
        
        let mut users = USERS.write().unwrap();
        match users.remove(&id) {
            Some(user) => json_response(object!({
                message: "User deleted successfully",
                deleted_user: user.to_json()
            })),
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND)
                    .content_type(HttpContentType::Json);
                response.body = serde_json::to_string(&object!({
                    error: "User not found",
                    id: id
                })).unwrap().into();
                response
            }
        }
    }
}

// Search users
endpoint! {
    APP.url("/api/users/search"),
    
    /// Search users by name or email
    /// Query parameter: q (search term)
    pub search_users <HTTP> {
        let query = req.query("q")
            .map(|v| v.string().to_lowercase())
            .unwrap_or_default();
        
        if query.is_empty() {
            return json_response(object!({
                error: "Search query 'q' is required"
            }));
        }
        
        let users = USERS.read().unwrap();
        let results: Vec<Value> = users
            .values()
            .filter(|user| {
                user.name.to_lowercase().contains(&query) ||
                user.email.to_lowercase().contains(&query)
            })
            .map(|user| user.to_json())
            .collect();
        
        json_response(object!({
            query: query,
            results: results,
            count: results.len()
        }))
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 Chapter 2: Advanced Routing & State Management");
    println!("==================================================");
    println!("Server running at http://127.0.0.1:3001\n");
    
    println!("API Endpoints:");
    println!("  GET    /api/users              - List all users");
    println!("  GET    /api/users/{id}         - Get specific user");
    println!("  POST   /api/users/create       - Create new user");
    println!("  PUT    /api/users/{id}/update  - Update user");
    println!("  DELETE /api/users/{id}/delete  - Delete user");
    println!("  GET    /api/users/search?q=... - Search users\n");
    
    println!("Example curl commands:");
    println!("  # List all users");
    println!("  curl http://localhost:3001/api/users\n");
    
    println!("  # Get user with ID 1");
    println!("  curl http://localhost:3001/api/users/1\n");
    
    println!("  # Create a new user");
    println!(r#"  curl -X POST http://localhost:3001/api/users/create \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{"name":"Charlie","email":"charlie@example.com"}'"#);
    println!();
    
    println!("  # Update user");
    println!(r#"  curl -X PUT http://localhost:3001/api/users/1/update \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{"name":"Alice Updated"}'"#);
    println!();
    
    println!("  # Search users");
    println!("  curl 'http://localhost:3001/api/users/search?q=alice'\n");
    
    APP.clone().run().await;
}