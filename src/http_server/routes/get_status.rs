use axum::extract::{State};
use axum::Json;
use crate::agent_data::AgentData;
use crate::http_server::http_server::AgentDataState;

pub async fn get_status(State(agent_data): State<AgentDataState>) -> Json<AgentData>{
    let mut agent_data_cloned = agent_data
        .agent_data
        .lock()
        .unwrap()
        .clone();

    agent_data_cloned.update();
    return Json(agent_data_cloned);
}