use axum::{
    routing::get,
    Router,
};

pub async fn server_start (host: String, port: String) {
    let (app, addr) = init_server_configuration(host, port).await;

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn init_server_configuration(host: String, port: String) -> (Router, String) {
    let app = init_basic_routes();
    let addr = format!("{}:{}", host, port);

    println!("Starting server at {}", addr);
    return (app.clone(), addr)
}

fn init_basic_routes() -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(users))
        .route("/heaviest_files", get(heaviest_files));
    return app
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn users() -> &'static str {
    "Get Users"
}

async fn heaviest_files() -> &'static str {
    "Get Heaviest Files"
}
