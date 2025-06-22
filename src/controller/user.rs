use axum::{extract::Path, Json};

use crate::model::user::User;

pub async fn create_user(Json(payload): Json<User>) -> Json<User> {
    let user = User {
        id: 1,
        username: payload.username,
    };
    Json(user)
}

pub async fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user = User {
        id,
        username: String::from("mcnwr"),
    };
    Json(user)
}

pub async fn get_users() -> Json<Vec<User>> {
    let user = vec![
        User {
            id: 1,
            username: String::from("user1"),
        },
        User {
            id: 2,
            username: String::from("user2"),
        },
    ];
    Json(user)
}
