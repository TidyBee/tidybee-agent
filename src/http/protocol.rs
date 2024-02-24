use crate::configuration::HttpConfig;
use crate::server::Protocol;
use axum::async_trait;
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub uuid: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct HttpRequest {
    pub path: String,
    pub body: String,
}

pub trait RequestBuilder {
    fn build_request(&self, path: String, body: String) -> HttpRequest;
}

#[derive(Clone)]
pub struct HttpRequestBuilder;

impl RequestBuilder for HttpRequestBuilder {
    fn build_request(&self, path: String, body: String) -> HttpRequest {
        HttpRequest { path, body }
    }
}

impl Default for HttpRequestBuilder {
    fn default() -> Self {
        Self
    }
}

#[derive(Clone)]
pub struct RequestDirector<B: RequestBuilder> {
    builder: B,
}

impl<B: RequestBuilder> RequestDirector<B> {
    pub fn new(builder: B) -> Self {
        Self { builder }
    }

    pub fn construct(&self, path: String, body: String) -> HttpRequest {
        self.builder.build_request(path, body)
    }
}

#[derive(Clone)]
pub struct HttpProtocol {
    #[allow(dead_code)]
    http_request_director: RequestDirector<HttpRequestBuilder>,
    config: HttpConfig,
    client: Client,
}

pub struct HttpProtocolBuilder {
    #[allow(dead_code)]
    http_request_builder: HttpRequestBuilder,
}

impl Default for HttpProtocolBuilder {
    fn default() -> Self {
        Self {
            http_request_builder: HttpRequestBuilder,
        }
    }
}

#[async_trait]
impl Protocol for HttpProtocol {
    async fn handle_post(&self, body_request: String) {
        info!("handle post called");
        let host_auth_path = self.config.host.clone() + &*self.config.auth_path.clone();
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let http_request =
            http_request_director.construct(host_auth_path.clone(), body_request.clone());
        let response = self
            .client
            .post(host_auth_path)
            .json(&http_request)
            .send()
            .await;
        info!("Sending : {:?}", http_request);
        info!("Response: {:?}", response);
    }

    fn dump(&self) -> Value {
        json!({
            "path": self.config.auth_path.clone(),
            "host": self.config.host.clone(),
        })
    }
}

impl HttpProtocolBuilder {
    pub fn new() -> Self {
        HttpProtocolBuilder::default()
    }

    pub fn build(self, config: HttpConfig) -> HttpProtocol {
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let client = Client::new();

        HttpProtocol {
            http_request_director,
            config,
            client,
        }
    }
}
