use std::collections::HashMap;

use webclassic_http::response::HttpResponse;
use webclassic_http::util::Method;
use webclassic_service::interrupt::Interrupt;

use crate::controller::{Context, Controller, EmptyController};
use crate::dispatcher::Route;

type RouteEntry = (Route, Box<dyn Controller>);

pub struct Dispatcher {
    routes: HashMap<Method, Vec<RouteEntry>>,
    fallback: Box<dyn Controller>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            fallback: Box::new(EmptyController::new()),
        }
    }

    pub fn with<C>(mut self, route: Route, controller: C) -> Self
    where
        C: Controller + 'static,
    {
        self.insert(route, controller);
        self
    }

    pub fn fallback<C>(mut self, controller: C) -> Self
    where
        C: Controller + 'static,
    {
        self.set_fallback(controller);
        self
    }

    pub fn insert<C>(&mut self, route: Route, controller: C)
    where
        C: Controller + 'static,
    {
        let controller = Box::new(controller);
        self.routes
            .entry(route.method())
            .or_default()
            .push((route, controller));
    }

    pub fn set_fallback<C>(&mut self, controller: C)
    where
        C: Controller + 'static,
    {
        self.fallback = Box::new(controller);
    }
}

impl Controller for Dispatcher {
    fn process(&self, context: Context, interrupt: &Interrupt) -> Option<HttpResponse> {
        let path = context.request().uri().path();
        let method = context.request().method();

        let matched = self.routes.get(&method).and_then(|routes| {
            routes
                .iter()
                .filter_map(|(route, controller)| {
                    route
                        .test(method, path)
                        .map(|metric| (metric, route, controller))
                })
                .reduce(|ra, rb| if ra.0.is_better_than(rb.0) { ra } else { rb })
        });

        if interrupt.is_interrupted() {
            return None;
        }

        match matched {
            Some((_, route, controller)) => {
                let tail = route.path().tail_for(path);
                let ctx = Context::with_tail(context.into_request(), tail);
                controller.process(ctx, interrupt)
            }
            None => self.fallback.process(context, interrupt),
        }
    }
}

#[cfg(test)]
mod tests {
    use webclassic_http::request::HttpRequest;
    use webclassic_http::response::HttpResponse;
    use webclassic_http::util::{Method, StatusCode};
    use webclassic_service::interrupt::InterruptSource;
    use webclassic_service::request::Request;

    use crate::controller::{Context, Controller};

    use super::*;

    struct FixedController(StatusCode);

    impl Controller for FixedController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            Some(HttpResponse::new(self.0))
        }
    }

    fn parse_request(raw: &str) -> HttpRequest {
        let (req, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        req
    }

    fn interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    #[test]
    fn route_matched() {
        let dispatcher = Dispatcher::new().with(
            Route::by(Method::Get).equal("/api"),
            FixedController(StatusCode::OK),
        );

        let req = parse_request("GET /api HTTP/1.0\r\n\r\n");
        let resp = dispatcher.process(Context::new(req), &interrupt()).unwrap();
        assert_eq!(resp.status().code(), 200);
    }

    #[test]
    fn longest_prefix_wins() {
        let dispatcher = Dispatcher::new()
            .with(
                Route::by(Method::Get).prefix("/api"),
                FixedController(StatusCode::OK),
            )
            .with(
                Route::by(Method::Get).prefix("/api/v1"),
                FixedController(StatusCode::CREATED),
            );

        let req = parse_request("GET /api/v1/users HTTP/1.0\r\n\r\n");
        let resp = dispatcher.process(Context::new(req), &interrupt()).unwrap();
        assert_eq!(resp.status().code(), 201);
    }

    #[test]
    fn no_match_fallback() {
        let dispatcher = Dispatcher::new()
            .with(
                Route::by(Method::Get).prefix("/api"),
                FixedController(StatusCode::OK),
            )
            .fallback(FixedController(StatusCode::NOT_FOUND));

        let req = parse_request("GET /other HTTP/1.0\r\n\r\n");
        let resp = dispatcher.process(Context::new(req), &interrupt()).unwrap();
        assert_eq!(resp.status().code(), 404);
    }

    #[test]
    fn method_filter() {
        let dispatcher = Dispatcher::new()
            .with(
                Route::by(Method::Get).prefix("/api"),
                FixedController(StatusCode::OK),
            )
            .fallback(FixedController(StatusCode::METHOD_NOT_ALLOWED));

        let req = parse_request("POST /api HTTP/1.0\r\n\r\n");
        let resp = dispatcher.process(Context::new(req), &interrupt()).unwrap();
        assert_eq!(resp.status().code(), 405);
    }
}
