use std::convert::Infallible;

use webclassic_http::request::{HttpRequest, ParseHttpRequestError};
use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_service::service::Service;

use crate::controller::{Context, Controller};

pub struct WebService {
    controller: Box<dyn Controller>,
}

impl WebService {
    pub fn new<C>(controller: C) -> Self
    where
        C: Controller + 'static,
    {
        Self::boxed(Box::new(controller))
    }

    pub fn boxed(controller: Box<dyn Controller>) -> Self {
        Self { controller }
    }
}

impl Service for WebService {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = Infallible;

    fn process(
        &self,
        request: Self::Request,
        interrupt: &Interrupt,
    ) -> Result<Option<Self::Response>, Self::Error> {
        let context = Context::new(request);
        Ok(self.controller.process(context, interrupt))
    }

    fn on_invalid(&self, _error: &ParseHttpRequestError) -> Option<HttpResponse> {
        Some(HttpResponse::new(StatusCode::BAD_REQUEST))
    }
}
