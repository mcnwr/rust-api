use axum::{
    routing::{get, post},
    Router,
};

use crate::controller::mqtt::{consumer, publisher};

pub async fn mqtt_router() -> Router {
    let router = Router::new()
        .route("/pub", post(publisher))
        .route("/consume", get(consumer));
    router
}
