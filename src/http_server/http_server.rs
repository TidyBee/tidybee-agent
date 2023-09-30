use axum::{routing::get, Router};

use crate::http_server::routes;

pub struct HttpServer {
    host: String,
    port: String,
}

impl HttpServer {
    pub fn new(host: String, port: String) -> HttpServer {
        HttpServer { host, port }
    }
    pub async fn server_start(self) {
        let (app, addr) = HttpServer::init_server_configuration(self).await;

        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    async fn init_server_configuration(self) -> (Router, String) {
        let app = HttpServer::init_basic_routes();
        let addr = format!("{}:{}", self.host, self.port);

        return (app.clone(), addr);
    }

    fn init_basic_routes() -> Router {
        let app = Router::new()
            .route("/", get(routes::hello_world))
            .route("/users", get(routes::get_users))
            .route("/heaviest_files", get(routes::get_heaviest_files));
        return app;
    }
}
