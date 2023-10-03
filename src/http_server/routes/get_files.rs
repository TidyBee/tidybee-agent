use axum::extract::{Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Greeting {
    message: String,
    amount: u32
}

pub async fn get_files(Json(payload): Json<Greeting>) -> Json<Greeting> {
    let greeting = Greeting {
        message: format!("Amount is : {}", payload.amount),
        amount: payload.amount,
    };
    Json(greeting)
}
