use std::sync::Arc;
use axum::{Extension, Json};
use serde::Serialize;
use crate::agent_infos::AgentInfos;

#[derive(Serialize, Clone)]
pub struct User {
    name: String,
    username: String,
}

pub async fn get_status(state: Extension<AgentInfos>) -> Json<Vec<User>> {
    let users: Vec<User> = vec![
        User {
            name: "Alice".to_string(),
            username: "alice".to_string(),
        },
        User {
            name: "Bob".to_string(),
            username: "bob".to_string(),
        },
    ];
    Json(users)
}
