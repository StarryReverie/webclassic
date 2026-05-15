use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::Controller;

#[derive(Debug, Clone)]
pub struct ConstantHandler {
    status: StatusCode,
    body: String,
    headers: Vec<(String, String)>,
}

impl ConstantHandler {
    pub fn new(body: String) -> Self {
        Self {
            status: StatusCode::OK,
            body,
            headers: Vec::new(),
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, name: &str, value: String) -> Self {
        self.headers.push((name.to_string(), value));
        self
    }
}

impl Controller for ConstantHandler {
    fn process(&self, _request: HttpRequest, _interrupt: &Interrupt) -> Option<HttpResponse> {
        let mut response = HttpResponse::new(self.status).with_body(self.body.clone().into_bytes());
        for (name, value) in &self.headers {
            response = response.with_header(name, value.clone());
        }
        Some(response)
    }
}
