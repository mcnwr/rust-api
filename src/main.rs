use axum::{routing::get, Router};

mod controller;
mod model;
mod routes;

use routes::user::user_router;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(user_router().await);

    let listener = "127.0.0.1:3000".parse().unwrap();
    println!("Server is running on {}", listener);

    axum::Server::bind(&listener)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
