use crate::agent_data;
use crate::agent_data::{AgentData, AgentDataBuilder};
use crate::file_info::FileInfo;
use crate::my_files;
use crate::my_files::{ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed};
use axum::extract::Query;
use axum::{extract::State, routing::get, Json, Router};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct Greeting {
    message: String,
}

#[derive(Deserialize)]
struct GetFilesParams {
    amount: usize,
    sort_by: String,
}

async fn hello_world() -> Json<Greeting> {
    let greeting: Greeting = Greeting {
        message: "hello world".to_owned(),
    };
    Json(greeting)
}

async fn get_status(State(agent_data): State<AgentDataState>) -> Json<AgentData> {
    let mut agent_data_cloned = agent_data.agent_data.lock().unwrap().clone();

    agent_data_cloned.update();
    Json(agent_data_cloned)
}

async fn get_files(
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

#[derive(Clone, Default)]
pub struct HttpServer {
    address: String,
    router: Router,
}

#[derive(Clone)]
struct MyFilesState {
    my_files: Arc<Mutex<my_files::MyFiles>>,
}

#[derive(Clone)]
struct AgentDataState {
    agent_data: Arc<Mutex<agent_data::AgentData>>,
}

#[derive(Clone, Default)]
pub struct HttpServerBuilder {
    router: Router,
    my_files_builder:
        my_files::MyFilesBuilder<ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed>,
}

impl HttpServerBuilder {
    pub fn new() -> Self {
        HttpServerBuilder::default()
    }

    pub fn my_files_builder(
        mut self,
        my_files_builder: my_files::MyFilesBuilder<
            ConfigurationWrapperPresent,
            ConnectionManagerPresent,
            Sealed,
        >,
    ) -> Self {
        self.my_files_builder = my_files_builder;
        self
    }

    pub async fn build(self, dirs_watch: Vec<PathBuf>, address: String) -> HttpServer {
        let my_files_instance = self.my_files_builder.build().unwrap();
        info!("MyFiles instance successfully created for HTTP Server");
        let my_files_state = MyFilesState {
            my_files: Arc::new(Mutex::new(my_files_instance)),
        };
        let agent_data_state = AgentDataState {
            agent_data: Arc::new(Mutex::new(AgentDataBuilder::new().build(dirs_watch))),
        };
        let router = self
            .router
            .route("/", get(hello_world))
            .route("/get_files", get(get_files).with_state(my_files_state))
            .route("/get_status", get(get_status).with_state(agent_data_state));
        HttpServer { address, router }
    }
}

impl HttpServer {
    pub async fn start(self) {
        let addr: SocketAddr = self.address.parse().unwrap();

        info!("Http Server running at {addr}");
        axum::Server::bind(&addr)
            .serve(self.router.into_make_service())
            .await
            .unwrap();
    }
}
