use std::sync::{Arc};
use axum::{Extension, Json};
use crate::agent_infos::AgentData;

pub async fn get_status(state: Extension<Arc<AgentData>>) -> Json<AgentData> {
    let pid = state.get_pid();

    Json(
        AgentData {
            process_id: pid
        }
    )
}
