use axum::Json;
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use tracing::{error, info};
use crate::configuration::HttpConfig;
use crate::server::Protocol;
use std::future::Future;
use std::ops::Add;
use futures::future::BoxFuture;

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub(crate) uuid: String
}

#[derive(Debug, Serialize, Clone)]
pub struct HttpRequest {
    pub(crate) route: String,
    pub(crate) body: String,
}

pub trait RequestBuilder {
    fn build_request(&self, route: String, body: String) -> HttpRequest;
}

#[derive(Clone)]
pub struct HttpRequestBuilder;

impl RequestBuilder for HttpRequestBuilder {
    fn build_request(&self, route: String, body: String) -> HttpRequest {
        HttpRequest {
            route,
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

    pub fn construct(&self, route: String, body: String) -> HttpRequest {
        self.builder.build_request(route, body)
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

impl Protocol for HttpProtocol {
    fn handle_post(&self, body: String) -> BoxFuture<'static, Json<HttpResponse>> {
        let http_request_builder = HttpRequestBuilder;
        let http_request_director = RequestDirector::new(http_request_builder);
        let http_request = http_request_director.construct("http://localhost:7001/gateway/auth/aoth".to_string(), body.clone());

        let client = self.client.clone();
        let config = self.config.clone();
        let mut url = config.host.clone();
        url.push_str(&*config.auth_route.clone());
        info!("Sending request to this url : {:?}", url);

        Box::pin(async move {
            info!("Sending HttpRequest: {:?}", http_request);
            let response = client.post(url).json(&http_request).send().await;

            info!("Sending HttpRequest: {:?}", http_request);
            let _ = match response {
                Ok(response) => {
                    let body = response.json::<HttpResponse>().await;
                    match body {
                        Ok(body) => Json(body),
                        Err(_) => {
                            error!("Error reading response body");
                            panic!("Error reading response body")
                        }
                    }
                }
                Err(_) => {
                    error!("Error sending POST request");
                    panic!("Error sending POST request")
                }
            };
            Json(HttpResponse { uuid: body })
        })
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