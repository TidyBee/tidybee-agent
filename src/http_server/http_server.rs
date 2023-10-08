use axum::{Router};
use axum::routing::MethodRouter;
use log::info;
use serde::Deserialize;
use crate::configuration_wrapper::ConfigurationWrapper;

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

#[derive(Clone, Default)]
pub struct HttpServerBuilder {
    router: Router,
    configuration_wrapper: ConfigurationWrapper
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        let host = "0.0.0.0".to_string();
        let port = "8080".to_string();

        HttpServerConfig {
            host,
            port
        }
    }
}

impl HttpServerBuilder {
    pub fn new() -> Self {
        HttpServerBuilder::default()
    }

    pub fn add_route(mut self, path: &str, method_router: MethodRouter) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    pub fn configuration_wrapper(mut self, configuration_wrapper: impl Into<ConfigurationWrapper>) -> Self{
        self.configuration_wrapper = configuration_wrapper.into();
        self
    }

    pub fn build(self) -> HttpServer {
        let http_server_config: HttpServerConfig = self.configuration_wrapper
            .bind::<HttpServerConfig>("http_server_config")
            .unwrap_or_default();
        let router = self.router;

        HttpServer {
            http_server_config,
            router
        }
    }
}

impl HttpServer {

    pub async fn start(self) {
        let addr = format!("{}:{}", self.http_server_config.host, self.http_server_config.port);

        info!("Http Server running on host : {} & port : {}", self.http_server_config.host, self.http_server_config.port);
        axum::Server::bind(&addr.parse().unwrap())
            .serve(self.router.into_make_service())
            .await
            .unwrap();
    }
}