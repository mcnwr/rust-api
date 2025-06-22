use axum::{
    routing::{get, post},
    Router,
};

use crate::controller::user::{create_user, get_user, get_users};

pub async fn user_router() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
}
