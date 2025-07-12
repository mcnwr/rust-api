use crate::config::db::DynamoDbConfig;
use aws_sdk_dynamodb::types::AttributeValue;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemRequest {
    pub table_name: String,
    pub item: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemRequest {
    pub table_name: String,
    pub key: HashMap<String, String>,
}

// List all tables
pub async fn list_tables(State(db): State<DynamoDbConfig>) -> Result<Json<Value>, StatusCode> {
    match db.list_tables().await {
        Ok(tables) => Ok(Json(json!({
            "success": true,
            "tables": tables
        }))),
        Err(e) => {
            eprintln!("Error listing tables: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Check if table exists
pub async fn check_table(
    State(db): State<DynamoDbConfig>,
    Path(table_name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match db.table_exists(&table_name).await {
        Ok(exists) => Ok(Json(json!({
            "success": true,
            "table_name": table_name,
            "exists": exists
        }))),
        Err(e) => {
            eprintln!("Error checking table: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create item in table
pub async fn create_item(
    State(db): State<DynamoDbConfig>,
    Json(request): Json<CreateItemRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Convert HashMap<String, String> to HashMap<String, AttributeValue>
    let mut item: HashMap<String, AttributeValue> = HashMap::new();
    for (key, value) in request.item {
        item.insert(key, AttributeValue::S(value));
    }

    match db
        .get_client()
        .put_item()
        .table_name(&request.table_name)
        .set_item(Some(item))
        .send()
        .await
    {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Item created successfully"
        }))),
        Err(e) => {
            eprintln!("Error creating item: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get item from table
pub async fn get_item(
    State(db): State<DynamoDbConfig>,
    Json(request): Json<GetItemRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Convert HashMap<String, String> to HashMap<String, AttributeValue>
    let mut key: HashMap<String, AttributeValue> = HashMap::new();
    for (k, v) in request.key {
        key.insert(k, AttributeValue::S(v));
    }

    match db
        .get_client()
        .get_item()
        .table_name(&request.table_name)
        .set_key(Some(key))
        .send()
        .await
    {
        Ok(response) => {
            if let Some(item) = response.item {
                // Convert AttributeValue back to simple values
                let mut result: HashMap<String, String> = HashMap::new();
                for (key, value) in item {
                    if let AttributeValue::S(s) = value {
                        result.insert(key, s);
                    }
                }
                Ok(Json(json!({
                    "success": true,
                    "item": result
                })))
            } else {
                Ok(Json(json!({
                    "success": false,
                    "message": "Item not found"
                })))
            }
        }
        Err(e) => {
            eprintln!("Error getting item: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
