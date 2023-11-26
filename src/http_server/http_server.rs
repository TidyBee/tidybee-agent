use crate::agent_data::agent_data;
use crate::agent_data::agent_data::AgentDataBuilder;
use crate::configuration_wrapper::ConfigurationWrapper;
use crate::http_server::routes;
use crate::my_files;
use crate::my_files::my_files::{ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed};
use axum::routing::{get, post};
use axum::{Json, Router};
use log::{error, info};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use axum::extract::ConnectInfo;

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

#[derive(Clone)]
pub struct AgentDataState {
    pub agent_data: Arc<Mutex<agent_data::AgentData>>,
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
        let port = "8111".to_string();

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

    pub async fn build(
        self,
        directories_watch_args: Vec<PathBuf>,
        configuration_wrapper: ConfigurationWrapper,
    ) -> HttpServer {
        let http_server_config: HttpServerConfig = self
            .configuration_wrapper
            .bind::<HttpServerConfig>("http_server_config")
            .unwrap_or_default();
        let my_files_instance = self.my_files_builder.build().unwrap();
        info!("MyFiles instance successfully created for HTTP Server");
        let my_files_state = MyFilesState {
            my_files: Arc::new(Mutex::new(my_files_instance)),
        };
        let agent_data_state = AgentDataState {
            agent_data: Arc::new(Mutex::new(
                AgentDataBuilder::new()
                    .configuration_wrapper(configuration_wrapper)
                    .build(directories_watch_args),
            )),
        };
        let router = self
            .router
            .route("/", get(routes::hello_world))
            .route(
                "/get_files/:nb_files/sorted_by/:sort_type",
                get(routes::get_files).with_state(my_files_state),
            )
            .route(
                "/get_status",
                get(routes::get_status).with_state(agent_data_state),
            );
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

#[cfg(test)]
mod tests {
   use super::*;
   use axum::{
       body::Body,
       extract::connect_info::MockConnectInfo,
       http::{self, Request, StatusCode},
   };
   use serde_json::{json, Value};
   use std::net::SocketAddr;
   use tokio::net::{TcpListener, TcpStream};
   use tower::Service; // for `call`
   use tower::ServiceExt; // for `oneshot` and `ready` // for `collect`

   #[tokio::test]
   async fn hello_world() {
       let app = app();

       // `Router` implements `tower::Service<Request<Body>>` so we can
       // call it like any tower service, no need to run an HTTP server.
       let response = app
           .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::OK);

       let body = response.into_body().collect().await.unwrap().to_bytes();
       assert_eq!(&body[..], b"Hello, World!");
   }

   #[tokio::test]
   async fn json() {
       let app = app();

       let response = app
           .oneshot(
               Request::builder()
                   .method(http::Method::POST)
                   .uri("/json")
                   .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                   .body(Body::from(
                       serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
                   ))
                   .unwrap(),
           )
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::OK);

       let body = response.into_body().collect().await.unwrap().to_bytes();
       let body: Value = serde_json::from_slice(&body).unwrap();
       assert_eq!(body, json!({ "data": [1, 2, 3, 4] }));
   }

   // #[tokio::test]
   // async fn not_found() {
   //     let app = app();
   //
   //     let response = app
   //         .oneshot(
   //             Request::builder()
   //                 .uri("/does-not-exist")
   //                 .body(Body::empty())
   //                 .unwrap(),
   //         )
   //         .await
   //         .unwrap();
   //
   //     assert_eq!(response.status(), StatusCode::NOT_FOUND);
   //     let body = response.into_body().collect().await.unwrap().to_bytes();
   //     assert!(body.is_empty());
   // }
   //
   // // You can also spawn a server and talk to it like any other HTTP server:
   // #[tokio::test]
   // async fn the_real_deal() {
   //     // TODO(david): convert this to hyper-util when thats published
   //
   //     use hyper::client::conn;
   //
   //     let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
   //     let addr = listener.local_addr().unwrap();
   //
   //     tokio::spawn(async move {
   //         axum::serve(listener, app()).await.unwrap();
   //     });
   //
   //     let target_stream = TcpStream::connect(addr).await.unwrap();
   //
   //     let (mut request_sender, connection) = conn::http1::handshake(target_stream).await.unwrap();
   //
   //     tokio::spawn(async move { connection.await.unwrap() });
   //
   //     let response = request_sender
   //         .send_request(
   //             Request::builder()
   //                 .uri(format!("http://{addr}"))
   //                 .header("Host", "localhost")
   //                 .body(Body::empty())
   //                 .unwrap(),
   //         )
   //         .await
   //         .unwrap();
   //
   //     let body = response.into_body().collect().await.unwrap().to_bytes();
   //     assert_eq!(&body[..], b"Hello, World!");
   // }
   //
   // // You can use `ready()` and `call()` to avoid using `clone()`
   // // in multiple request
   // #[tokio::test]
   // async fn multiple_request() {
   //     let mut app = app().into_service();
   //
   //     let request = Request::builder().uri("/").body(Body::empty()).unwrap();
   //     let response = ServiceExt::<Request<Body>>::ready(&mut app)
   //         .await
   //         .unwrap()
   //         .call(request)
   //         .await
   //         .unwrap();
   //     assert_eq!(response.status(), StatusCode::OK);
   //
   //     let request = Request::builder().uri("/").body(Body::empty()).unwrap();
   //     let response = ServiceExt::<Request<Body>>::ready(&mut app)
   //         .await
   //         .unwrap()
   //         .call(request)
   //         .await
   //         .unwrap();
   //     assert_eq!(response.status(), StatusCode::OK);
   // }
   //
   // // Here we're calling `/requires-connect-into` which requires `ConnectInfo`
   // //
   // // That is normally set with `Router::into_make_service_with_connect_info` but we can't easily
   // // use that during tests. The solution is instead to set the `MockConnectInfo` layer during
   // // tests.
   // #[tokio::test]
   // async fn with_into_make_service_with_connect_info() {
   //     let mut app = app()
   //         .layer(MockConnectInfo(SocketAddr::from(([0, 0, 0, 0], 3000))))
   //         .into_service();
   //
   //     let request = Request::builder()
   //         .uri("/requires-connect-into")
   //         .body(Body::empty())
   //         .unwrap();
   //     let response = app.ready().await.unwrap().call(request).await.unwrap();
   //     assert_eq!(response.status(), StatusCode::OK);
   // }
}