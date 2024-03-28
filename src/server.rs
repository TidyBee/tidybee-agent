use crate::agent_data::AgentData;
use crate::http::routes::{
    get_config, get_files, get_status, hello_world, AgentDataState, GlobalConfigState, MyFilesState,
};
use crate::my_files::{ConfigurationPresent, ConnectionManagerPresent, Sealed};
use crate::tidy_algo::{TidyAlgo, TidyRule};
use crate::{configuration, my_files};
use axum::{routing::get, Router};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level};

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

#[derive(Default)]
pub struct Server {
    address: String,
    router: Router,
}

#[derive(Clone, Default)]
pub struct ServerBuilder {
    router: Router,
    global_configuration: configuration::Configuration,
    tidy_rules: Vec<TidyRule>,
    my_files_builder:
        my_files::MyFilesBuilder<ConfigurationPresent, ConnectionManagerPresent, Sealed>,
}

trait ServerConfig {
    fn host(&self) -> &str;
    fn route(&self) -> &str;
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self::default()
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

    pub fn inject_global_configuration(
        mut self,
        global_configuration: configuration::Configuration,
    ) -> Self {
        self.global_configuration = global_configuration;
        self
    }

    pub fn inject_tidy_rules(mut self, tidy_algo: TidyAlgo) -> Self {
        self.tidy_rules = tidy_algo.get_rules().to_vec();
        self
    }

    pub fn build(
        self,
        latest_version: String,
        minimal_version: String,
        dirs_watch: Vec<PathBuf>,
        address: String,
        logging_level: &str,
    ) -> Server {
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
        let global_config_state = GlobalConfigState {
            config: self.global_configuration,
            rules: self.tidy_rules,
        };

        let server_logging_level: Level = AGENT_LOGGING_LEVEL.get(logging_level).map_or_else(
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
            .route("/config", get(get_config).with_state(global_config_state))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(server_logging_level))
                    .on_response(trace::DefaultOnResponse::new().level(server_logging_level))
                    .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
            );

        Server { address, router }
    }
}

impl Server {
    pub async fn start(self) {
        let addr: SocketAddr = match self.address.parse() {
            Ok(addr) => addr,
            Err(_) => {
                let default_config: Self = Self::default();
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
        axum::serve(tcp_listener, self.router).await.unwrap();
    }
}
