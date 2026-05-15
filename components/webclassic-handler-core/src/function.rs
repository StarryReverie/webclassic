use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

type HandlerFn = Box<dyn Fn(Context, &Interrupt) -> Option<HttpResponse> + Send + Sync>;

pub struct FunctionHandler {
    f: HandlerFn,
}

impl FunctionHandler {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Context, &Interrupt) -> Option<HttpResponse> + Send + Sync + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl Controller for FunctionHandler {
    fn process(&self, context: Context, interrupt: &Interrupt) -> Option<HttpResponse> {
        (self.f)(context, interrupt)
    }
}
