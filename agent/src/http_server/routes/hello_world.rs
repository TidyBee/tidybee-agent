use serde::Serialize;
use axum::{
    Json
};

#[derive(Serialize)]
pub struct Greeting {
    message: String,
}

pub async fn hello_world() ->  Json<Greeting> {
    let greeting: Greeting = Greeting {
        message: "hello world".to_string(),
    };
    Json(greeting)
}