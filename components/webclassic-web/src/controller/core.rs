use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;

pub trait Controller {
    fn process(&self, request: HttpRequest, interrupt: &Interrupt) -> Option<HttpResponse>;
}
