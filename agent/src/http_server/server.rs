use axum::{prelude::*, Router};
use std::net::SocketAddr;

impl server {
    async fn handle_request() -> String {
        "Hello from rust".to_string()
    }

    pub async fn run_server() -> Result<(), std::io::Error> {
        let app = Router::new().route("/", get(handle_request));

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        println!("Server listening on http://{}", addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
