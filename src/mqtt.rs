use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct PublishMessage {
    pub topic: String,
    pub message: String,
    pub qos: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct MqttStatus {
    pub connected: bool,
    pub broker_url: String,
    pub active_subscriptions: u32,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// Publish message to MQTT broker
pub async fn publish_message(
    Json(payload): Json<PublishMessage>,
) -> Result<Json<Value>, StatusCode> {
    // Simulate MQTT publishing
    println!(
        "Publishing to topic '{}': {}",
        payload.topic, payload.message
    );

    // Simulate potential failure for testing
    if payload.topic.contains("error") {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(json!({
        "success": true,
        "topic": payload.topic,
        "message_id": "msg_12345",
        "qos": payload.qos.unwrap_or(0),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get MQTT broker status
pub async fn get_status() -> Result<Json<MqttStatus>, StatusCode> {
    let status = MqttStatus {
        connected: true,
        broker_url: std::env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| "amqp://localhost:5672".to_string()),
        active_subscriptions: 5,
        messages_sent: 1247,
        messages_received: 892,
    };

    Ok(Json(status))
}
