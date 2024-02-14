use crate::agent_data;
use crate::agent_data::AgentData;
use crate::file_info::FileInfo;
use crate::my_files;
use crate::my_files::{ConfigurationPresent, ConnectionManagerPresent, Sealed};
use axum::{extract::Query, extract::State, routing::get, Json, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level};
use reqwest::Client;

lazy_static! {
    static ref AGENT_LOGGING_LEVEL: HashMap<String, Level> = {
        let mut m = HashMap::new();
        m.insert("trace".to_owned(), Level::TRACE);
        m.insert("debug".to_owned(), Level::DEBUG);
        m.insert("info".to_owned(), Level::INFO);
        m.insert("warn".to_owned(), Level::WARN);
        m.insert("error".to_owned(), Level::ERROR);
        m
    };
}

#[derive(Serialize)]
struct Greeting {
    message: String,
}

#[derive(Deserialize)]
struct GetFilesParams {
    amount: usize,
    sort_by: String,
}

#[derive(Serialize)]
pub struct HttpRequest {
    host: String,
    body: String
}

#[derive(Deserialize)]
pub struct HttpResponse {
    body: String
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
    client: Client
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
        my_files::MyFilesBuilder<ConfigurationPresent, ConnectionManagerPresent, Sealed>,
}

impl HttpServerBuilder {
    pub fn new() -> Self {
        HttpServerBuilder::default()
    }

    pub fn my_files_builder(
        mut self,
        my_files_builder: my_files::MyFilesBuilder<
            ConfigurationPresent,
            ConnectionManagerPresent,
            Sealed,
        >,
    ) -> Self {
        self.my_files_builder = my_files_builder;
        self
    }

    pub fn build(
        self,
        latest_version: String,
        minimal_version: String,
        dirs_watch: Vec<PathBuf>,
        address: String,
        logging_level: String,
    ) -> HttpServer {
        let my_files_instance = self.my_files_builder.build().unwrap();
        info!("MyFiles instance successfully created for HTTP Server");
        let my_files_state = MyFilesState {
            my_files: Arc::new(Mutex::new(my_files_instance)),
        };
        let agent_data_state = AgentDataState {
            agent_data: Arc::new(Mutex::new(AgentData::build(
                latest_version,
                minimal_version,
                dirs_watch,
            ))),
        };

        let server_logging_level: Level = AGENT_LOGGING_LEVEL.get(&logging_level).map_or_else(
            || {
                error!(
                    "Invalid logging level: {}. Defaulting to info.",
                    logging_level
                );
                Level::INFO
            },
            |level| *level,
        );

        let router = self
            .router
            .route("/", get(hello_world))
            .route("/get_files", get(get_files).with_state(my_files_state))
            .route("/get_status", get(get_status).with_state(agent_data_state))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(server_logging_level))
                    .on_response(trace::DefaultOnResponse::new().level(server_logging_level))
                    .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
            );

        let client = Client::new();

        HttpServer { address, router, client }
    }
}

impl HttpServer {
    pub async fn start(self) {
        let addr: SocketAddr = match self.address.parse() {
            Ok(addr) => addr,
            Err(_) => {
                let default_config: HttpServer = HttpServer::default();
                error!(
                    "Invalid host or port: {}, defaulting to {}",
                    self.address, default_config.address
                );
                default_config.address.parse().unwrap()
            }
        };
        let tcp_listener = match TcpListener::bind::<SocketAddr>(addr).await {
            Ok(tcp_listener) => tcp_listener,
            Err(e) => {
                error!("Failed to bind to {}: {}", addr.to_string(), e);
                return;
            }
        };
        info!("Http Server running at {}", addr.to_string());
        axum::serve(tcp_listener, self.router).await.unwrap();
    }

    pub async fn handle_post(self, request: Json<HttpRequest>) -> Json<HttpResponse> {
        let host = request.host.clone();
        let response = self.client.post(host).json(&request.0).send().await;

        match response {
            Ok(response) => {
                let body = response.json::<HttpResponse>().await;
                match body {
                    Ok(body) => Json(body),
                    Err(_) => {
                        error!("Error reading response body");
                        panic!("Error reading response body")
                    }
                }
            }
            Err(_) => {
                error!("Error sending POST request");
                panic!("Error sending POST request")
            }
        }
    }
}
