use std::fs;
use std::path::{Path, PathBuf};

use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

use crate::mime::guess_content_type;

#[derive(Debug, Clone)]
pub struct StaticDirectoryHandler {
    root: PathBuf,
    index_file: Option<String>,
}

impl StaticDirectoryHandler {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            index_file: None,
        }
    }

    pub fn with_index_file(mut self, name: &str) -> Self {
        self.index_file = Some(name.to_string());
        self
    }
}

fn is_under_root(path: &Path, root: &Path) -> bool {
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    match path.canonicalize() {
        Ok(canonical_path) => canonical_path.starts_with(canonical_root),
        Err(_) => {
            let cleaned = path.components().collect::<PathBuf>();
            let mut candidate = canonical_root.clone();
            for comp in cleaned.components() {
                if comp == std::path::Component::ParentDir {
                    if !candidate.pop() {
                        return false;
                    }
                } else if comp != std::path::Component::CurDir {
                    candidate.push(comp);
                }
            }
            candidate.starts_with(canonical_root)
        }
    }
}

fn serve_file(path: &Path) -> Option<HttpResponse> {
    match fs::read(path) {
        Ok(body) => {
            let content_type = guess_content_type(path);
            let response = HttpResponse::new(StatusCode::OK)
                .with_header("content-type", content_type.to_string())
                .with_body(body);
            Some(response)
        }
        Err(_) => Some(HttpResponse::new(StatusCode::NOT_FOUND)),
    }
}

impl Controller for StaticDirectoryHandler {
    fn process(&self, context: Context, _interrupt: &Interrupt) -> Option<HttpResponse> {
        let relative = context.route_tail();

        let resolved = self.root.join(relative);

        if !is_under_root(&resolved, &self.root) {
            return Some(HttpResponse::new(StatusCode::FORBIDDEN));
        }

        if resolved.is_dir() {
            let request_path = context.request().uri().path();
            if !request_path.ends_with('/') {
                let response = HttpResponse::new(StatusCode::FOUND)
                    .with_header("location", format!("{}/", request_path));
                return Some(response);
            }

            if let Some(ref index) = self.index_file {
                let index_path = resolved.join(index);
                if !is_under_root(&index_path, &self.root) {
                    return Some(HttpResponse::new(StatusCode::FORBIDDEN));
                }
                return serve_file(&index_path);
            }

            return Some(HttpResponse::new(StatusCode::NOT_FOUND));
        }

        serve_file(&resolved)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use webclassic_http::request::HttpRequest;
    use webclassic_service::interrupt::InterruptSource;
    use webclassic_service::request::Request;
    use webclassic_web::controller::Controller;

    use super::*;

    fn make_context(path: &str, route_tail: &str) -> Context {
        let raw = format!("GET {} HTTP/1.0\r\n\r\n", path);
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        Context::with_tail(request, route_tail.to_string())
    }

    fn make_interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    fn setup_dir(tmp: &Path) {
        fs::create_dir_all(tmp.join("css")).unwrap();
        fs::write(tmp.join("css/style.css"), b"body { color: red; }").unwrap();
        fs::write(tmp.join("index.html"), b"<h1>Hello</h1>").unwrap();
        fs::create_dir_all(tmp.join("secret")).unwrap();
    }

    #[test]
    fn serve_file_from_directory() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler = StaticDirectoryHandler::new(tmp.path().to_path_buf());
        let ctx = make_context("/static/css/style.css", "css/style.css");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert_eq!(body, "body { color: red; }");
    }

    #[test]
    fn directory_without_trailing_slash_redirects() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler =
            StaticDirectoryHandler::new(tmp.path().to_path_buf()).with_index_file("index.html");
        let ctx = make_context("/static/css", "css");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get("location").unwrap()[0],
            "/static/css/"
        );
    }

    #[test]
    fn directory_with_trailing_slash_serves_index() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler =
            StaticDirectoryHandler::new(tmp.path().to_path_buf()).with_index_file("index.html");
        let ctx = make_context("/static/", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert_eq!(body, "<h1>Hello</h1>");
    }

    #[test]
    fn root_path_serves_index() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler =
            StaticDirectoryHandler::new(tmp.path().to_path_buf()).with_index_file("index.html");
        let ctx = make_context("/", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert_eq!(body, "<h1>Hello</h1>");
    }

    #[test]
    fn directory_without_index_returns_404() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler = StaticDirectoryHandler::new(tmp.path().to_path_buf());
        let ctx = make_context("/static/", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn path_traversal_returns_403() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler = StaticDirectoryHandler::new(tmp.path().to_path_buf());
        let ctx = make_context("/static/../../../etc/passwd", "../../../etc/passwd");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn missing_file_returns_404() {
        let tmp = tempfile::tempdir().unwrap();
        setup_dir(tmp.path());

        let handler = StaticDirectoryHandler::new(tmp.path().to_path_buf());
        let ctx = make_context("/static/nope.txt", "nope.txt");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::NOT_FOUND);
    }
}
