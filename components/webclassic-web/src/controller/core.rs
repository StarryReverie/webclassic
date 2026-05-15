use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;

pub struct Context {
    request: HttpRequest,
    route_tail: String,
}

impl Context {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request,
            route_tail: String::new(),
        }
    }

    pub fn with_tail(request: HttpRequest, route_tail: String) -> Self {
        Self {
            request,
            route_tail,
        }
    }

    pub fn request(&self) -> &HttpRequest {
        &self.request
    }

    pub fn route_tail(&self) -> &str {
        &self.route_tail
    }

    pub fn into_request(self) -> HttpRequest {
        self.request
    }
}

pub trait Controller: Send + Sync {
    fn process(&self, context: Context, interrupt: &Interrupt) -> Option<HttpResponse>;
}
