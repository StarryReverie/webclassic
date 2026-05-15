use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;

use crate::controller::{Context, Controller};

#[derive(Default)]
pub struct EmptyController {}

impl EmptyController {
    pub fn new() -> Self {
        Self {}
    }
}

impl Controller for EmptyController {
    fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
        Some(HttpResponse::new(StatusCode::NOT_FOUND))
    }
}
