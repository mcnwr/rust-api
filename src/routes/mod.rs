pub mod channel;
pub mod user;

use crate::routes::channel::channel_router;
use crate::routes::user::user_router;
use axum::Router;

pub async fn routes() -> Router {
    let app = Router::new()
        .nest("/user", user_router().await)
        .nest("/channel", channel_router().await);
    app
}
