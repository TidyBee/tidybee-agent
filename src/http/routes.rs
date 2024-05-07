use crate::agent_data::AgentData;
use axum::extract::State;
use axum::Json;
use serde_derive::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AgentDataState {
    pub agent_data: Arc<Mutex<AgentData>>,
}

#[derive(Clone)]
pub struct GlobalConfigState {
    pub config: crate::configuration::Configuration,
}

#[derive(Serialize)]
pub struct Greeting {
    message: String,
}

pub async fn hello_world() -> Json<Greeting> {
    let greeting: Greeting = Greeting {
        message: "Hello from server".to_owned(),
    };
    Json(greeting)
}

pub async fn get_status(State(agent_data): State<AgentDataState>) -> Json<AgentData> {
    let mut agent_data_cloned = agent_data.agent_data.lock().unwrap().clone();

    agent_data_cloned.update();
    Json(agent_data_cloned)
}

#[derive(Serialize)]
pub struct GetConfigResponseType {
    configuration: crate::configuration::Configuration,
}

pub async fn get_config(
    State(global_config): State<GlobalConfigState>,
) -> Json<GetConfigResponseType> {
    let configuration = global_config.config;
    let response = GetConfigResponseType { configuration };

    Json(response)
}
