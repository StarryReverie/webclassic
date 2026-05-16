use std::sync::Arc;

use chrono::Local;
use webclassic_http::response::HttpResponse;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller, Filter};

use crate::backend::{LogBackend, LogEntry};

pub struct LogFilter {
    backend: Arc<dyn LogBackend>,
}

impl LogFilter {
    pub fn new(backend: Arc<dyn LogBackend>) -> Self {
        Self { backend }
    }
}

impl Filter for LogFilter {
    fn filter<C>(
        &self,
        controller: &C,
        context: Context,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse>
    where
        C: Controller,
    {
        let timestamp = Local::now();
        let request = context.request();
        let method = request.method();
        let path = request.uri().path().to_string();
        let referer = request
            .headers()
            .get("referer")
            .and_then(|v| v.first())
            .cloned();
        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.first())
            .cloned();

        let response = controller.process(context, interrupt);

        if let Some(ref r) = response {
            let entry = LogEntry {
                remote_addr: None,
                timestamp,
                method,
                path,
                status: r.status().code(),
                size: r.body().len(),
                referer,
                user_agent,
            };
            self.backend.log(&entry);
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Mutex;

    use webclassic_http::request::HttpRequest;
    use webclassic_http::response::HttpResponse;
    use webclassic_http::util::{Method, StatusCode, Uri};
    use webclassic_service::interrupt::InterruptSource;
    use webclassic_web::controller::{Context, Controller};

    use super::*;

    struct FixedController(StatusCode);

    impl Controller for FixedController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            Some(HttpResponse::new(self.0).with_body(b"hello".to_vec()))
        }
    }

    struct NoneController;

    impl Controller for NoneController {
        fn process(&self, _context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
            None
        }
    }

    struct CaptureBackend {
        entries: Mutex<Vec<LogEntry>>,
    }

    impl CaptureBackend {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
            }
        }
    }

    impl LogBackend for CaptureBackend {
        fn log(&self, entry: &LogEntry) {
            self.entries.lock().unwrap().push(entry.clone());
        }
    }

    fn interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    fn make_filter(backend: &Arc<CaptureBackend>) -> LogFilter {
        let dyn_backend: Arc<dyn LogBackend> = Arc::clone(backend) as Arc<dyn LogBackend>;
        LogFilter::new(dyn_backend)
    }

    #[test]
    fn logs_successful_response() {
        let backend = Arc::new(CaptureBackend::new());
        let filter = make_filter(&backend);
        let request = HttpRequest::new(Method::Get, Uri::from_str("/index.html").unwrap())
            .with_header("Referer", "http://example.com/".to_string());
        let context = Context::new(request);

        let response = filter
            .filter(&FixedController(StatusCode::OK), context, &interrupt())
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let entries = backend.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].method, Method::Get);
        assert_eq!(entries[0].path, "/index.html");
        assert_eq!(entries[0].status, 200);
        assert_eq!(entries[0].size, 5);
        assert_eq!(entries[0].referer.as_deref(), Some("http://example.com/"));
        assert!(entries[0].user_agent.is_none());
    }

    #[test]
    fn logs_error_response() {
        let backend = Arc::new(CaptureBackend::new());
        let filter = make_filter(&backend);
        let request = HttpRequest::new(Method::Get, Uri::from_str("/missing").unwrap());
        let context = Context::new(request);

        filter.filter(
            &FixedController(StatusCode::NOT_FOUND),
            context,
            &interrupt(),
        );

        let entries = backend.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, 404);
    }

    #[test]
    fn none_response_not_logged() {
        let backend = Arc::new(CaptureBackend::new());
        let filter = make_filter(&backend);
        let request = HttpRequest::new(Method::Get, Uri::from_str("/").unwrap());
        let context = Context::new(request);

        let result = filter.filter(&NoneController, context, &interrupt());
        assert!(result.is_none());

        let entries = backend.entries.lock().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn captures_user_agent() {
        let backend = Arc::new(CaptureBackend::new());
        let filter = make_filter(&backend);
        let request = HttpRequest::new(Method::Get, Uri::from_str("/").unwrap())
            .with_header("User-Agent", "TestBot/1.0".to_string());
        let context = Context::new(request);

        filter.filter(&FixedController(StatusCode::OK), context, &interrupt());

        let entries = backend.entries.lock().unwrap();
        assert_eq!(entries[0].user_agent.as_deref(), Some("TestBot/1.0"));
    }

    #[test]
    fn passes_response_through() {
        let backend = Arc::new(CaptureBackend::new());
        let filter = make_filter(&backend);
        let request = HttpRequest::new(Method::Get, Uri::from_str("/").unwrap());
        let context = Context::new(request);

        let response = filter
            .filter(&FixedController(StatusCode::OK), context, &interrupt())
            .unwrap();

        assert_eq!(response.body(), b"hello");
    }
}
