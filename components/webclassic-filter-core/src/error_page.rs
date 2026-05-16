use std::collections::HashMap;

use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller, Filter};

pub struct ErrorPageFilter {
    pages: HashMap<StatusCode, Box<dyn Fn() -> HttpResponse + Send + Sync>>,
}

impl Default for ErrorPageFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorPageFilter {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }

    pub fn with_page(self, status: StatusCode, html: String) -> Self {
        let response = HttpResponse::new(status)
            .with_header("Content-Type", "text/html".to_string())
            .with_body(html.into_bytes());
        let mut pages = self.pages;
        pages.insert(status, Box::new(move || response.clone()));
        Self { pages }
    }

    pub fn with_handler<F>(self, status: StatusCode, handler: F) -> Self
    where
        F: Fn() -> HttpResponse + Send + Sync + 'static,
    {
        let mut pages = self.pages;
        pages.insert(status, Box::new(handler));
        Self { pages }
    }
}

impl Filter for ErrorPageFilter {
    fn filter<C>(
        &self,
        controller: &C,
        context: Context,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse>
    where
        C: Controller,
    {
        let response = controller.process(context, interrupt);
        match response {
            Some(ref r) if r.status().code() >= 400 => match self.pages.get(&r.status()) {
                Some(handler) => Some(handler()),
                None => response,
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use webclassic_http::request::HttpRequest;
    use webclassic_http::util::{Method, Uri};
    use webclassic_service::interrupt::InterruptSource;

    use super::*;

    struct FixedController(StatusCode);

    impl Controller for FixedController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            Some(HttpResponse::new(self.0))
        }
    }

    struct NoneController;

    impl Controller for NoneController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            None
        }
    }

    fn interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    fn context() -> Context {
        use std::str::FromStr;
        let request = HttpRequest::new(Method::Get, Uri::from_str("/").unwrap());
        Context::new(request)
    }

    #[test]
    fn unmapped_error_passes_through() {
        let controller = FixedController(StatusCode::INTERNAL_SERVER_ERROR);
        let filter = ErrorPageFilter::new();
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.body().is_empty());
    }

    #[test]
    fn success_passes_through() {
        let controller = FixedController(StatusCode::OK);
        let filter = ErrorPageFilter::new();
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.body().is_empty());
    }

    #[test]
    fn none_passes_through() {
        let filter = ErrorPageFilter::new();
        let result = filter.filter(&NoneController, context(), &interrupt());
        assert!(result.is_none());
    }

    #[test]
    fn redirect_not_intercepted() {
        let controller = FixedController(StatusCode::MOVED_PERMANENTLY);
        let filter = ErrorPageFilter::new();
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert!(response.body().is_empty());
    }

    #[test]
    fn with_page_overrides_specific_status() {
        let controller = FixedController(StatusCode::NOT_FOUND);
        let filter = ErrorPageFilter::new().with_page(
            StatusCode::NOT_FOUND,
            "<html><body>custom 404</body></html>".to_string(),
        );
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.body(), b"<html><body>custom 404</body></html>");
    }

    #[test]
    fn with_page_falls_back_to_pass_through() {
        let controller = FixedController(StatusCode::INTERNAL_SERVER_ERROR);
        let filter =
            ErrorPageFilter::new().with_page(StatusCode::NOT_FOUND, "custom 404".to_string());
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.body().is_empty());
    }

    #[test]
    fn with_handler_custom_closure() {
        let filter = ErrorPageFilter::new().with_handler(StatusCode::NOT_FOUND, || {
            HttpResponse::new(StatusCode::NOT_FOUND)
                .with_header("Content-Type", "text/plain".to_string())
                .with_body(b"custom error".to_vec())
        });
        let controller = FixedController(StatusCode::NOT_FOUND);
        let response = filter.filter(&controller, context(), &interrupt()).unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.body(), b"custom error");
        assert_eq!(
            response.headers().get("content-type"),
            Some(["text/plain".to_string()].as_slice())
        );
    }

    #[test]
    fn with_page_chains_multiple() {
        let filter = ErrorPageFilter::new()
            .with_page(StatusCode::NOT_FOUND, "custom 404".to_string())
            .with_page(StatusCode::FORBIDDEN, "custom 403".to_string());

        let resp404 = filter
            .filter(
                &FixedController(StatusCode::NOT_FOUND),
                context(),
                &interrupt(),
            )
            .unwrap();
        assert_eq!(resp404.body(), b"custom 404");

        let resp403 = filter
            .filter(
                &FixedController(StatusCode::FORBIDDEN),
                context(),
                &interrupt(),
            )
            .unwrap();
        assert_eq!(resp403.body(), b"custom 403");

        let resp500 = filter
            .filter(
                &FixedController(StatusCode::INTERNAL_SERVER_ERROR),
                context(),
                &interrupt(),
            )
            .unwrap();
        assert_eq!(resp500.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(resp500.body().is_empty());
    }
}
