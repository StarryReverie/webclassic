use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::Controller;

type HandlerFn = Box<dyn Fn(HttpRequest, &Interrupt) -> Option<HttpResponse> + Send + Sync>;

pub struct FunctionHandler {
    f: HandlerFn,
}

impl FunctionHandler {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(HttpRequest, &Interrupt) -> Option<HttpResponse> + Send + Sync + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl Controller for FunctionHandler {
    fn process(&self, request: HttpRequest, interrupt: &Interrupt) -> Option<HttpResponse> {
        (self.f)(request, interrupt)
    }
}
