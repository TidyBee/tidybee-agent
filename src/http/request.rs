use serde_derive::{Deserialize, Serialize};

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
    fn build_request(&self, route: &str, body: &str) -> HttpRequest;
}

pub struct HttpRequestBuilder;

impl RequestBuilder for HttpRequestBuilder {
    fn build_request(&self, route: &str, body: &str) -> HttpRequest {
        HttpRequest {
            route: route.to_string(),
            body: body.to_string(),
        }
    }
}

pub struct RequestDirector<B: RequestBuilder> {
    builder: B,
}

impl<B: RequestBuilder> RequestDirector<B> {
    pub fn new(builder: B) -> Self {
        RequestDirector { builder }
    }

    pub fn construct(&self, route: &str, body: &str) -> HttpRequest {
        self.builder.build_request(route, body)
    }
}
