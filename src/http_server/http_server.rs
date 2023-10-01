use axum::{routing::get, Router};
use log::info;
use crate::http_server::routes;

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

    pub fn host(&mut self, host: impl Into<String>) -> &mut Self{
        self.host = Some(host.into());
        self
    }

    pub fn port(&mut self, port: impl Into<String>) -> &mut Self{
        self.port = Some(port.into());
        self
    }

    pub fn router(&mut self) -> &mut Self {
        self.router = Router::new()
            .route("/", get(routes::hello_world))
            .route("/users", get(routes::get_users))
            .route("/heaviest_files", get(routes::get_heaviest_files));
        self
    }

    pub fn build(&self) -> HttpServer {
        let host = self.host
            .as_ref().cloned()
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let port = self.port
            .as_ref().cloned()
            .unwrap_or_else(|| "8080".to_string());
        let router = self.router.clone();

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