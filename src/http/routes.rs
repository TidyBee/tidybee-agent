use std::sync::{Arc, Mutex};
use axum::extract::{Query, State};
use axum::Json;
use serde_derive::{Deserialize, Serialize};
use crate::agent_data::AgentData;
use crate::my_files;
use crate::file_info::FileInfo;
use tracing::{error, info, Level};

#[derive(Clone)]
pub struct AgentDataState {
    pub(crate) agent_data: Arc<Mutex<AgentData>>,
}

#[derive(Clone)]
pub struct MyFilesState {
    pub(crate) my_files: Arc<Mutex<my_files::MyFiles>>,
}

#[derive(Deserialize)]
pub struct GetFilesParams {
    amount: usize,
    sort_by: String,
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

pub async fn get_files(
    State(my_files): State<MyFilesState>,
    Query(query_params): Query<GetFilesParams>,
) -> Json<Vec<FileInfo>> {
    let mut files_vec: Vec<FileInfo> = my_files
        .my_files
        .lock()
        .unwrap()
        .get_all_files_from_db()
        .unwrap();

    files_vec.sort_by(|a, b| match query_params.sort_by.to_lowercase().as_str() {
        "size" => b.size.cmp(&a.size),
        "last_update" => b.last_modified.cmp(&a.last_modified),
        _ => {
            error!("Invalid sort parameter in get_files route. Defaulting to size.");
            b.size.cmp(&a.size)
        }
    });
    let result = files_vec.into_iter().take(query_params.amount).collect();
    Json(result)
}