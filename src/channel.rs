use axum::{extract::Path, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub message_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannel {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u32,
    pub channel_id: u32,
    pub content: String,
    pub sender: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessage {
    pub content: String,
    pub sender: String,
}

/// Get all channels
pub async fn get_channels() -> Result<Json<Value>, StatusCode> {
    let channels = vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            description: "General discussion".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            message_count: 42,
        },
        Channel {
            id: 2,
            name: "random".to_string(),
            description: "Random chat".to_string(),
            created_at: "2024-01-02T00:00:00Z".to_string(),
            message_count: 13,
        },
    ];

    Ok(Json(json!({
        "channels": channels,
        "count": channels.len()
    })))
}

/// Get channel by ID
pub async fn get_channel(Path(id): Path<u32>) -> Result<Json<Channel>, StatusCode> {
    match id {
        1 => Ok(Json(Channel {
            id: 1,
            name: "general".to_string(),
            description: "General discussion".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            message_count: 42,
        })),
        2 => Ok(Json(Channel {
            id: 2,
            name: "random".to_string(),
            description: "Random chat".to_string(),
            created_at: "2024-01-02T00:00:00Z".to_string(),
            message_count: 13,
        })),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Create new channel
pub async fn create_channel(
    Json(payload): Json<CreateChannel>,
) -> Result<Json<Channel>, StatusCode> {
    let new_channel = Channel {
        id: 3,
        name: payload.name,
        description: payload.description,
        created_at: chrono::Utc::now().to_rfc3339(),
        message_count: 0,
    };

    Ok(Json(new_channel))
}

/// Get messages from a channel
pub async fn get_channel_messages(Path(channel_id): Path<u32>) -> Result<Json<Value>, StatusCode> {
    let messages = vec![
        Message {
            id: 1,
            channel_id,
            content: "Hello everyone!".to_string(),
            sender: "Alice".to_string(),
            timestamp: "2024-01-01T10:00:00Z".to_string(),
        },
        Message {
            id: 2,
            channel_id,
            content: "How's everyone doing?".to_string(),
            sender: "Bob".to_string(),
            timestamp: "2024-01-01T10:05:00Z".to_string(),
        },
    ];

    Ok(Json(json!({
        "messages": messages,
        "channel_id": channel_id,
        "count": messages.len()
    })))
}

/// Send message to a channel
pub async fn send_message(
    Path(channel_id): Path<u32>,
    Json(payload): Json<SendMessage>,
) -> Result<Json<Message>, StatusCode> {
    // Simulate potential failure for load testing
    if payload.content.contains("error") {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let message = Message {
        id: 123, // In real app, this would be generated
        channel_id,
        content: payload.content,
        sender: payload.sender,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(message))
}
