use crate::configuration_wrapper::ConfigurationWrapper;
use axum::routing::get;
use axum::Router;
use log::{error, info};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::http_server::routes;
use crate::my_files;
use crate::my_files::my_files::{ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed};

#[derive(Debug, Deserialize, Clone)]
pub struct HttpServerConfig {
    host: String,
    port: String,
}

#[derive(Clone, Default)]
pub struct HttpServer {
    http_server_config: HttpServerConfig,
    router: Router,
}

#[derive(Clone)]
pub struct MyFilesState {
    pub my_files: Arc<Mutex<my_files::MyFiles>>,
}

#[derive(Clone, Default)]
pub struct HttpServerBuilder {
    router: Router,
    my_files_builder:
        my_files::MyFilesBuilder<ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed>,
    configuration_wrapper: ConfigurationWrapper,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        let host = "0.0.0.0".to_string();
        let port = "8080".to_string();

        HttpServerConfig { host, port }
    }
}

impl HttpServerBuilder {
    pub fn new() -> Self {
        HttpServerBuilder::default()
    }

    pub fn configuration_wrapper(
        mut self,
        configuration_wrapper: impl Into<ConfigurationWrapper>,
    ) -> Self {
        self.configuration_wrapper = configuration_wrapper.into();
        self
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

    pub async fn build(self) -> HttpServer {
        let http_server_config: HttpServerConfig = self
            .configuration_wrapper
            .bind::<HttpServerConfig>("http_server_config")
            .unwrap_or_default();

        let my_files_instance = self.my_files_builder.build().unwrap();
        info!("MyFiles instance sucessfully created for HTTP Server");
        let my_files_state = MyFilesState {
            my_files: Arc::new(Mutex::new(my_files_instance)),
        };

        let router = self
            .router
            .route("/", get(routes::hello_world))
            .route("/heaviest_files", get(routes::get_heaviest_files))
            .route("/get_files", get(routes::get_files).with_state(my_files_state));
        HttpServer {
            http_server_config,
            router,
        }
    }
}

impl HttpServer {
    pub async fn start(self) {
        let addr: SocketAddr = match format!(
            "{}:{}",
            self.http_server_config.host, self.http_server_config.port
        )
        .parse()
        {
            Ok(addr) => addr,
            Err(_) => {
                let default_config: HttpServerConfig = HttpServerConfig::default();
                error!(
                    "Invalid host or port: {}:{}, defaulting to {}:{}",
                    self.http_server_config.host,
                    self.http_server_config.port,
                    default_config.host,
                    default_config.port
                );
                format!("{}:{}", default_config.host, default_config.port)
                    .parse()
                    .unwrap()
            }
        };

        info!("Http Server running at {}", addr.to_string());
        axum::Server::bind(&addr)
            .serve(self.router.into_make_service())
            .await
            .unwrap();
    }
}
