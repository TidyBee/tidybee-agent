use serde::Serialize;
use axum::{
    routing::get,
    Router,
    Json,
};

pub struct Server {
    host: String,
    port: String,
}

#[derive(Serialize, Clone)]
struct User {
    name: String,
    username: String,
}

impl Server {
    pub fn new(host: String, port: String) -> Server {
        Server {
            host,
            port,
        }
    }
//-----------------------------------------PUBLIC-------------------------------------//
    pub async fn server_start (self) {
        let (app, addr) = Server::init_server_configuration(self).await;

        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

//-----------------------------------------PRIVATE------------------------------------//
    async fn init_server_configuration(self) -> (Router, String) {
        let app = Server::init_basic_routes();
        let addr = format!("{}:{}", self.host, self.port);

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
//-----------------------------------------END----------------------------------------//
}

//----------------------------------------ROUTES--------------------------------------//
async fn root() -> &'static str {
    "Hello, World!"
}

async fn users() -> Json<User> {
    let test_user = User {
        name: "Hourcadette".to_string(),
        username: "Julien".to_string(),
    };
    Json(test_user)
}

async fn heaviest_files() -> &'static str {
    "Get Heaviest Files"
}
//-----------------------------------------END----------------------------------------//