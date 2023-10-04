use axum::{Router};
use axum::routing::MethodRouter;
use log::info;
use crate::HttpServerConfig;

#[derive(Clone, Default)]
pub struct HttpServer {
    host: String,
    port: String,
    router: Router,
}

#[derive(Clone, Default)]
pub struct HttpServerBuilder {
    host: Option<String>,
    port: Option<String>,
    router: Router,
}

impl HttpServerBuilder {
    pub fn new() -> Self {
        HttpServerBuilder::default()
    }

    pub fn configuration_wrapper(mut self, http_server_config: impl Into<HttpServerConfig>) -> Self {
        let config = http_server_config.into();

        self.host = Some(config.host);
        self.port = Some(config.port);
        self
    }

    pub fn add_route(mut self, path: &str, method_router: MethodRouter) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    pub fn build(self) -> HttpServer {
        let host = self.host
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let port = self.port
            .unwrap_or_else(|| "8080".to_string());
        let router = self.router;

        HttpServer {
            host,
            port,
            router
        }
    }
}

impl HttpServer {

    pub async fn start(self) {
        let addr = format!("{}:{}", self.host, self.port);

        info!("Http Server running on host : {} & port : {}", self.host, self.port);
        axum::Server::bind(&addr.parse().unwrap())
            .serve(self.router.into_make_service())
            .await
            .unwrap();
    }
}