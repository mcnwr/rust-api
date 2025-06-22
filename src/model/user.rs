use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}
