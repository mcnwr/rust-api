use axum::Router;
use std::net::SocketAddr;
mod config;
mod controller;
mod lambda;
mod model;

mod routes;
use config::db::DynamoDbConfig;
use dotenv::dotenv;
use lambda::function_handler;
use lambda_http::service_fn;
use routes::routes;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv().ok();

    tracing_subscriber::fmt::init();

    // Initialize DynamoDB connection
    let _db_config = match DynamoDbConfig::new().await {
        Ok(config) => {
            println!("✅ DynamoDB connection established successfully");

            // Test connection by listing tables
            match config.list_tables().await {
                Ok(tables) => {
                    println!("📊 Available DynamoDB tables: {:?}", tables);
                }
                Err(e) => {
                    println!("⚠️  Warning: Could not list tables: {}", e);
                }
            }

            Some(config)
        }
        Err(e) => {
            println!("❌ Failed to connect to DynamoDB: {}", e);
            println!("⚠️  Continuing without DynamoDB connection");
            None
        }
    };

    // Check if running in Lambda environment
    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        // Running in AWS Lambda environment
        lambda_http::run(service_fn(function_handler))
            .await
            .unwrap();
    } else {
        // Running as regular web server with Axum
        let app = Router::new().merge(routes().await);

        let port: u16 = std::env::var("PORT")
            .expect("PORT environment variable is required")
            .parse()
            .expect("PORT must be a valid number between 1 and 65535");

        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        println!("🚀 Server starting on http://{}", addr);
        println!("📊 Performance Reports available at http://{}/", addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{Request, RequestExt};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_lambda_handler() {
        let request = Request::default();

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello world, this is an AWS Lambda HTTP request"
        );
    }

    #[tokio::test]
    async fn test_lambda_handler_with_query_string() {
        let mut query_string_parameters: HashMap<String, String> = HashMap::new();
        query_string_parameters.insert("name".into(), "rust-lambda".into());

        let request = Request::default().with_query_string_parameters(query_string_parameters);

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello rust-lambda, this is an AWS Lambda HTTP request"
        );
    }
}
