use std::fs;
use std::path::PathBuf;

use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

use crate::mime::guess_content_type;

pub struct StaticFileHandler {
    path: PathBuf,
}

impl StaticFileHandler {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Controller for StaticFileHandler {
    fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
        match fs::read(&self.path) {
            Ok(body) => {
                let content_type = guess_content_type(&self.path);
                let response = HttpResponse::new(StatusCode::OK)
                    .with_header("content-type", content_type.to_string())
                    .with_body(body);
                Some(response)
            }
            Err(_) => Some(HttpResponse::new(StatusCode::NOT_FOUND)),
        }
    }
}
