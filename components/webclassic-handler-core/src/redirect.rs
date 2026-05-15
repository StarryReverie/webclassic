use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

#[derive(Debug, Clone)]
pub struct RedirectHandler {
    status: StatusCode,
    location: String,
}

impl RedirectHandler {
    pub fn permanent(location: String) -> Self {
        Self {
            status: StatusCode::MOVED_PERMANENTLY,
            location,
        }
    }

    pub fn temporary(location: String) -> Self {
        Self {
            status: StatusCode::FOUND,
            location,
        }
    }
}

impl Controller for RedirectHandler {
    fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
        Some(
            HttpResponse::new(self.status)
                .with_header("location", self.location.clone())
                .with_body(Vec::new()),
        )
    }
}
