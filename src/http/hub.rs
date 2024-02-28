use std::env;
use crate::configuration::HttpConfig;
use axum::async_trait;
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub(crate) uuid: String,
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
        HttpRequest { path, body }
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
pub struct Hub {
    http_request_director: RequestDirector<HttpRequestBuilder>,
    config: HttpConfig,
    client: Client,
}

pub struct HubBuilder {
    http_request_builder: HttpRequestBuilder,
}

impl Default for HubBuilder {
    fn default() -> Self {
        HubBuilder {
            http_request_builder: HttpRequestBuilder::default(),
        }
    }
}

impl Hub {
    async fn connect(&self) {
        let uuid = env::var("AGENT_UUID").unwrap_or_else(|_| "default_value".to_string());

        let url = if uuid != "default_value" {
            self.config.host.clone() + &*self.config.auth_path + "/" + &*uuid
        } else {
            self.config.host.clone() + &*self.config.auth_path
        };

        let client = Client::new();
        let response = client.post(url)
            .body("body_req")
            .send()
            .await?;

        if response.status().is_success() {
            info!("Request successfully send");
        } else {
            info!("Error sending request");
        }
    }

    async fn handle_post(&self, body_request: String) {
        info!("handle post called");
        let host_auth_path = self.config.host.clone() + &*self.config.auth_path.clone();
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let http_request = http_request_director.construct(host_auth_path.clone(),
                                                           body_request.clone(),
        );
        let response = self
            .client
            .post(host_auth_path)
            .json(&http_request)
            .send()
            .await;
        info!("Sending : {:?}", http_request);
        info!("Response: {:?}", response);
    }

    fn config_dump(&self) -> Value {
        json!({
            "path": self.config.auth_path.clone(),
            "host": self.config.host.clone(),
        })
    }
}

impl HubBuilder {
    pub fn new() -> Self {
        HubBuilder::default()
    }

    pub fn build(self, config: HttpConfig) -> Hub {
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let client = Client::new();

        Hub {
            http_request_director,
            config,
            client,
        }
    }
}
