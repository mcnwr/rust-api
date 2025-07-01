use axum::{
    routing::{get, post},
    Router,
};

use crate::controller::mqtt::{consumer, publisher, publisher_with_task};

pub async fn mqtt_router() -> Router {
    let router = Router::new()
        .route("/publisher", post(publisher))
        .route("/pub", post(publisher_with_task))
        .route("/consume", get(consumer));
    router
}
