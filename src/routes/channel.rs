use axum::{routing::post, Router};

use crate::controller::channel::pub_user;

pub async fn channel_router() -> Router {
    Router::new().route("/pub", post(pub_user))
}
