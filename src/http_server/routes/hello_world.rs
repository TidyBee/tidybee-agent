use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Greeting {
    message: String,
}

pub async fn hello_world() -> Json<Greeting> {
    let greeting: Greeting = Greeting {
        message: "hello world".to_owned(),
    };
    Json(greeting)
}
