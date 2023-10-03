use axum::{Router};
use axum::routing::MethodRouter;
use log::info;

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

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: impl Into<String>) -> Self {
        self.port = Some(port.into());
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