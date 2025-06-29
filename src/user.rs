use axum::{extract::Path, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

/// Get all users
pub async fn get_users() -> Result<Json<Value>, StatusCode> {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    Ok(Json(json!({
        "users": users,
        "count": users.len()
    })))
}

/// Get user by ID
pub async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, StatusCode> {
    // Simulate database lookup
    match id {
        1 => Ok(Json(User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        })),
        2 => Ok(Json(User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        })),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Create new user
pub async fn create_user(Json(payload): Json<CreateUser>) -> Result<Json<User>, StatusCode> {
    // Simulate user creation
    let new_user = User {
        id: 3, // In real app, this would be generated
        name: payload.name,
        email: payload.email,
    };

    Ok(Json(new_user))
}
