use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;

use crate::controller::Controller;

pub trait Filter {
    fn filter<C>(
        &self,
        controller: &C,
        request: HttpRequest,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse>
    where
        C: Controller;
}

pub struct FilteredController<F, C> {
    filter: F,
    controller: C,
}

impl<F, C> FilteredController<F, C> {
    pub fn new(filter: F, controller: C) -> Self {
        Self { filter, controller }
    }
}

impl<F, C> Controller for FilteredController<F, C>
where
    F: Filter,
    C: Controller,
{
    fn process(&self, request: HttpRequest, interrupt: &Interrupt) -> Option<HttpResponse> {
        self.filter.filter(&self.controller, request, interrupt)
    }
}

pub trait FilterExt: Controller {
    fn filtered<F>(self, filter: F) -> FilteredController<F, Self>
    where
        Self: Sized,
    {
        FilteredController::new(filter, self)
    }
}

impl<C> FilterExt for C where C: Controller {}
