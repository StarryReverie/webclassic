use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

use crate::env::build_cgi_env;
use crate::parse::parse_cgi_output;

pub struct CgiHandler {
    script: PathBuf,
}

impl CgiHandler {
    pub fn new(script: PathBuf) -> Self {
        Self { script }
    }
}

fn build_query_string(request: &HttpRequest) -> String {
    match request.uri().query() {
        Some(query) => {
            let mut pairs = Vec::new();
            for (key, values) in query.iter() {
                for value in values {
                    pairs.push(format!("{}={}", key, value));
                }
            }
            pairs.join("&")
        }
        None => String::new(),
    }
}

fn build_script_name(request: &HttpRequest, route_tail: &str) -> String {
    let path = request.uri().path();
    if route_tail.is_empty() {
        path.to_string()
    } else {
        path.strip_suffix(route_tail).unwrap_or(path).to_string()
    }
}

impl Controller for CgiHandler {
    fn process(&self, context: Context, interrupt: &Interrupt) -> Option<HttpResponse> {
        if !self.script.exists() {
            return Some(HttpResponse::new(StatusCode::NOT_FOUND));
        }

        let request = context.request();
        let path_info = context.route_tail().to_string();
        let script_name = build_script_name(request, &path_info);
        let query_string = build_query_string(request);
        let env = build_cgi_env(request, &script_name, &path_info, &query_string);

        let mut child = match Command::new(&self.script)
            .envs(&env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)),
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(request.body());
            drop(stdin);
        }

        loop {
            if interrupt.is_interrupted() {
                let _ = child.kill();
                let _ = child.wait();
                return Some(HttpResponse::new(StatusCode::SERVICE_UNAVAILABLE));
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
                    }
                    break;
                }
                Ok(None) => std::thread::yield_now(),
                Err(_) => return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)),
            }
        }

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(_) => return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)),
        };

        Some(parse_cgi_output(&output.stdout))
    }
}

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;

    use webclassic_http::request::HttpRequest;
    use webclassic_service::interrupt::InterruptSource;
    use webclassic_service::request::Request;
    use webclassic_web::controller::Controller;

    use super::*;

    fn make_context(path: &str, route_tail: &str) -> Context {
        let raw = format!("GET {} HTTP/1.0\r\nHost: localhost\r\n\r\n", path);
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        Context::with_tail(request, route_tail.to_string())
    }

    fn make_interrupt() -> Interrupt {
        InterruptSource::new().subscribe()
    }

    fn write_script(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let script_path = dir.join(name);
        fs::write(&script_path, content).unwrap();
        fs::set_permissions(&script_path, Permissions::from_mode(0o755)).unwrap();
        script_path
    }

    #[test]
    fn script_returns_200() {
        let tmp = tempfile::tempdir().unwrap();
        let script = write_script(
            tmp.path(),
            "hello.sh",
            "#!/usr/bin/env sh\necho 'Content-Type: text/plain'\necho ''\necho 'Hello CGI'",
        );

        let handler = CgiHandler::new(script);
        let ctx = make_context("/hello", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::OK);
        assert_eq!(response.body(), b"Hello CGI\n");
    }

    #[test]
    fn script_returns_custom_status() {
        let tmp = tempfile::tempdir().unwrap();
        let script = write_script(
            tmp.path(),
            "notfound.sh",
            "#!/usr/bin/env sh\necho 'Status: 404 Not Found'\necho 'Content-Type: text/plain'\necho ''\necho 'gone'",
        );

        let handler = CgiHandler::new(script);
        let ctx = make_context("/notfound", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn script_not_found_returns_404() {
        let handler = CgiHandler::new(PathBuf::from("/nonexistent/script"));
        let ctx = make_context("/test", "");
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn script_receives_query_string() {
        let tmp = tempfile::tempdir().unwrap();
        let script = write_script(
            tmp.path(),
            "echo_qs.sh",
            "#!/usr/bin/env sh\necho 'Content-Type: text/plain'\necho ''\necho \"$QUERY_STRING\"",
        );

        let handler = CgiHandler::new(script);
        let raw = "GET /echo_qs?foo=bar HTTP/1.0\r\nHost: localhost\r\n\r\n";
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        let ctx = Context::with_tail(request, String::new());

        let response = handler.process(ctx, &make_interrupt()).unwrap();
        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("foo=bar"));
    }

    #[test]
    fn script_receives_path_info() {
        let tmp = tempfile::tempdir().unwrap();
        let script = write_script(
            tmp.path(),
            "echo_pi.sh",
            "#!/usr/bin/env sh\necho 'Content-Type: text/plain'\necho ''\necho \"$PATH_INFO\"",
        );

        let handler = CgiHandler::new(script);
        let raw = "GET /article/2024/hi HTTP/1.0\r\nHost: localhost\r\n\r\n";
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        let ctx = Context::with_tail(request, "/2024/hi".to_string());

        let response = handler.process(ctx, &make_interrupt()).unwrap();
        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("/2024/hi"));
    }
}
