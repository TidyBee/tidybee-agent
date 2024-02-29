use std::env;
use crate::configuration::HttpConfig;
use axum::async_trait;
use futures::task::Spawn;
use log::{debug, error};
use reqwest::{Client, Response};
use serde::ser::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

const MAX_ATTEMPTS: u32 = 30;

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
    pub async fn connect(&self) -> Result<Result<String, reqwest::Error>, dyn Error> {
        let mut attempts = 0;
        let uuid = env::var("AGENT_UUID").unwrap_or_else(|_| "default_value".to_string());

        let url = if uuid == "default_value" {
            self.config.host.clone() + &*self.config.auth_path
        } else {
            self.config.host.clone() + &*self.config.auth_path + "/" + &*uuid
        };

        debug!("Defined url : {:?}", url);
        while attempts < MAX_ATTEMPTS {
            match self.handle_post("test".to_string(), url.clone()).await {
                Ok(response) => {
                    if response.status().is_success() {
                        let body_text = response.text().await;
                        info!("Request successfully sent, Response : {:?}", body_text);
                        return Ok(body_text);
                    } else {
                        error!("Error sending request : {:?}", response.status());
                    }
                }
                Err(err) => {
                    error!("Error sending request : {}", err);
                }
            }
            attempts += 1;
        }
        Err(Error::custom("Maximum number of attempts reached without success."))
    }

    async fn handle_post(&self, body_request: String, url: String) -> Result<Response, dyn Error> {
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let http_request = http_request_director.construct(url.clone(), body_request.clone());

        info!("Sending request: {:?}", http_request);

        let response = match self.client.post(&url)
            .json(&http_request)
            .send()
            .await {
            Ok(res) => res,
            Err(err) => {
                error!("Error sending request: {}", err);
                return Err(err.into());
            }
        };

        if response.status().is_ok() {
            info!("Request sent successfully!");
        } else {
            info!("Error sending request: {}", response.status());
        }

        Ok(response)
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
