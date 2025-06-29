use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod channel;
mod health;
mod mqtt;
mod performance_viewer;
mod user;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create application router
    let app = Router::new()
        // API routes
        .route("/health", get(health::health_check))
        .route("/users", get(user::get_users))
        .route("/users", post(user::create_user))
        .route("/users/:id", get(user::get_user))
        .route("/mqtt/publish", post(mqtt::publish_message))
        .route("/mqtt/status", get(mqtt::get_status))
        .route("/channels", get(channel::get_channels))
        .route("/channels", post(channel::create_channel))
        .route("/channels/:id", get(channel::get_channel))
        .route("/channels/:id/messages", get(channel::get_channel_messages))
        .route("/channels/:id/messages", post(channel::send_message))
        // Performance viewer routes
        .nest("/", performance_viewer::create_router())
        // API routes for performance data
        .nest("/api/performance", performance_viewer::create_router())
        // Enable CORS for all routes
        .layer(CorsLayer::permissive());

    // Determine the address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("🚀 Server starting on http://{}", addr);
    println!("📊 Performance Reports available at http://{}/", addr);

    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
