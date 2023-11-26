use crate::agent_data::AgentData;
use crate::http_server::http_server::AgentDataState;
use axum::extract::State;
use axum::Json;

pub async fn get_status(State(agent_data): State<AgentDataState>) -> Json<AgentData> {
    let mut agent_data_cloned = agent_data.agent_data.lock().unwrap().clone();

    agent_data_cloned.update();
    Json(agent_data_cloned)
}
