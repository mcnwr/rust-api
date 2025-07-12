use crate::config::db::DynamoDbConfig;
use crate::controller::dynamodb_controller::{check_table, create_item, get_item, list_tables};
use axum::{
    routing::{get, post},
    Router,
};

pub async fn dynamodb_router() -> Router {
    // Initialize DynamoDB config
    let db_config = match DynamoDbConfig::new().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to initialize DynamoDB config: {}", e);
            // Return a router without DynamoDB functionality
            return Router::new().route("/health", get(|| async { "DynamoDB connection failed" }));
        }
    };

    Router::new()
        .route("/tables", get(list_tables))
        .route("/table/:table_name/exists", get(check_table))
        .route("/item", post(create_item))
        .route("/item/get", post(get_item))
        .with_state(db_config)
}
