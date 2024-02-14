use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HttpResponse {
    body: String
}

#[derive(Debug, Serialize)]
pub struct HttpRequest {
    pub(crate) route: String,
    body: String,
}

trait RequestBuilder {
    fn build_request(&self, route: &str, body: &str) -> HttpRequest;
}

struct HttpRequestBuilder;

impl RequestBuilder for HttpRequestBuilder {
    fn build_request(&self, route: &str, body: &str) -> HttpRequest {
        HttpRequest {
            route: route.to_string(),
            body: body.to_string(),
        }
    }
}

struct RequestDirector<B: RequestBuilder> {
    builder: B,
}

impl<B: RequestBuilder> RequestDirector<B> {
    fn new(builder: B) -> Self {
        RequestDirector { builder }
    }

    fn construct(&self, route: &str, body: &str) -> HttpRequest {
        self.builder.build_request(route, body)
    }
}
