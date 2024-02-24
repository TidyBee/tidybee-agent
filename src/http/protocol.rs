use axum::{async_trait};
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use tracing::{info};
use crate::configuration::HttpConfig;
use crate::server::Protocol;
use serde_json::{json, Value};

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub(crate) uuid: String
}

#[derive(Debug, Serialize, Clone)]
pub struct HttpRequest {
    pub(crate) path: String,
    pub(crate) body: String,
}

pub trait RequestBuilder {
    fn build_request(&self, path: String, body: String) -> HttpRequest;
}

#[derive(Clone)]
pub struct HttpRequestBuilder;

impl RequestBuilder for HttpRequestBuilder {
    fn build_request(&self, path: String, body: String) -> HttpRequest {
        HttpRequest {
            path,
            body,
        }
    }
}

impl Default for HttpRequestBuilder {
    fn default() -> Self {
        HttpRequestBuilder
    }
}

#[derive(Clone)]
pub struct RequestDirector<B: RequestBuilder> {
    builder: B,
}

impl<B: RequestBuilder> RequestDirector<B> {
    pub fn new(builder: B) -> Self {
        RequestDirector { builder }
    }

    pub fn construct(&self, path: String, body: String) -> HttpRequest {
        self.builder.build_request(path, body)
    }
}

#[derive(Clone)]
pub struct HttpProtocol {
    http_request_director: RequestDirector<HttpRequestBuilder>,
    config: HttpConfig,
    client: Client
}

pub struct HttpProtocolBuilder {
    http_request_builder: HttpRequestBuilder
}

impl Default for HttpProtocolBuilder {
    fn default() -> Self {
        HttpProtocolBuilder {
            http_request_builder: HttpRequestBuilder::default(),
        }
    }
}

#[async_trait]
impl Protocol for HttpProtocol {
    async fn handle_post(&self, body_request: String) {
        info!("handle post called");
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let http_request = http_request_director.construct("http://localhost:7001/gateway/auth/aoth".to_string(), body_request.clone());
        let response = self.client.post("http://localhost:7001/gateway/auth/aoth").json(&http_request).send().await;

        info!("Sending : {:?}", http_request);
        info!("Response: {:?}", response);
    }
    fn dump(&self) -> Value {
        let request = self.http_request_director.construct("http://localhost:7001/gateway/auth/aoth".to_string(), "test".to_string());
        let json_data = json!({
            "path": self.config.auth_path.clone(),
            "host": self.config.host.clone(),
            "request": request
        });
        return json_data;
    }
}

impl HttpProtocolBuilder {
    pub fn new() -> Self {HttpProtocolBuilder::default()}

    pub fn build(self, config: HttpConfig) -> HttpProtocol{
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let client = Client::new();

        HttpProtocol {
            http_request_director,
            config,
            client
        }
    }
}