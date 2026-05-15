use std::convert::Infallible;

use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;
use webclassic_service::service::Service;

use crate::controller::Controller;

pub struct WebService {
    controller: Box<dyn Controller + Send + Sync>,
}

impl WebService {
    pub fn new<C>(controller: C) -> Self
    where
        C: Controller + Send + Sync + 'static,
    {
        Self::boxed(Box::new(controller))
    }

    pub fn boxed(controller: Box<dyn Controller + Send + Sync>) -> Self {
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
        Ok(self.controller.process(request, interrupt))
    }
}
