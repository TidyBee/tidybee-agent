use axum::{
    routing::get,
    Router,
};

pub async fn server_start () {
    init_server_configuration().await;
}

async fn init_server_configuration() {
    let app = init_basic_routes();

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_basic_routes() -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(get_users))
        .route("/heaviest_files", get(get_heaviest_files));
    return app
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_users() -> &'static str {
    "Get Users"
}

async fn get_heaviest_files() -> &'static str {
    "Get Heaviest Files"
}
