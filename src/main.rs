use axum::Router;
use std::net::SocketAddr;
mod controller;
mod model;
mod performance_viewer;

mod routes;
use routes::routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().merge(routes().await);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("🚀 Server starting on http://{}", addr);
    println!("📊 Performance Reports available at http://{}/", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
