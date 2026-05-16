use webclassic_http::response::HttpResponse;
use webclassic_http::util::Method;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller, Filter};

#[derive(Debug, Clone)]
pub struct HeadFilter;

impl Default for HeadFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl HeadFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Filter for HeadFilter {
    fn filter<C>(
        &self,
        controller: &C,
        context: Context,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse>
    where
        C: Controller,
    {
        if context.request().method() == Method::Head {
            let request = context.into_request().with_method(Method::Get);
            let ctx = Context::new(request);
            controller.process(ctx, interrupt).map(|r| {
                let body_len = r.body().len();
                r.with_body(Vec::new())
                    .with_header("Content-Length", body_len.to_string())
            })
        } else {
            controller.process(context, interrupt)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use webclassic_http::request::HttpRequest;
    use webclassic_http::util::{StatusCode, Uri};
    use webclassic_service::interrupt::InterruptSource;

    use super::*;

    struct FixedController(StatusCode, Vec<u8>);

    impl Controller for FixedController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            let response = HttpResponse::new(self.0)
                .with_header("Content-Type", "text/html".to_string())
                .with_body(self.1.clone());
            Some(response)
        }
    }

    fn interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    #[test]
    fn head_strips_body_preserves_content_length() {
        let controller = FixedController(StatusCode::OK, b"hello world".to_vec());
        let filter = HeadFilter::new();
        let request = HttpRequest::new(Method::Head, Uri::from_str("/page").unwrap());
        let context = Context::new(request);

        let response = filter.filter(&controller, context, &interrupt()).unwrap();

        assert_eq!(response.status(), &StatusCode::OK);
        assert!(response.body().is_empty());
        assert_eq!(
            response.headers().get("content-length"),
            Some(["11".to_string()].as_slice())
        );
        assert_eq!(
            response.headers().get("content-type"),
            Some(["text/html".to_string()].as_slice())
        );
    }

    #[test]
    fn get_passes_through() {
        let body = b"hello world".to_vec();
        let controller = FixedController(StatusCode::OK, body.clone());
        let filter = HeadFilter::new();
        let request = HttpRequest::new(Method::Get, Uri::from_str("/page").unwrap());
        let context = Context::new(request);

        let response = filter.filter(&controller, context, &interrupt()).unwrap();

        assert_eq!(response.body(), body);
    }

    #[test]
    fn head_returns_none_when_controller_returns_none() {
        struct NoneController;
        impl Controller for NoneController {
            fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
                None
            }
        }

        let filter = HeadFilter::new();
        let request = HttpRequest::new(Method::Head, Uri::from_str("/page").unwrap());
        let context = Context::new(request);

        let result = filter.filter(&NoneController, context, &interrupt());
        assert!(result.is_none());
    }
}
