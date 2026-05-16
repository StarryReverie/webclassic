use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;
use webclassic_http::request::HttpRequest;
use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller};

use crate::env::build_cgi_env;
use crate::parse::parse_cgi_output;

const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone)]
pub struct CgiHandler {
    command: PathBuf,
    args: Vec<String>,
}

impl CgiHandler {
    pub fn new(command: PathBuf, args: Vec<String>) -> Self {
        Self { command, args }
    }

    fn script_path(&self) -> &Path {
        match self.args.first() {
            Some(arg) => Path::new(arg),
            None => &self.command,
        }
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

fn build_script_name(request: &HttpRequest, segments: &[String]) -> String {
    let path = request.uri().path();
    if segments.is_empty() {
        path.to_string()
    } else {
        let tail = format!("/{}", segments.join("/"));
        path.strip_suffix(&tail).unwrap_or(path).to_string()
    }
}

impl Controller for CgiHandler {
    fn process(&self, context: Context, interrupt: &Interrupt) -> Option<HttpResponse> {
        if !self.script_path().exists() {
            return Some(HttpResponse::new(StatusCode::NOT_FOUND));
        }

        let request = context.request();
        let segments = context.route_tail();
        let path_info = if segments.is_empty() {
            String::new()
        } else {
            format!("/{}", segments.join("/"))
        };
        let script_name = build_script_name(request, segments);
        let query_string = build_query_string(request);
        let env = build_cgi_env(request, &script_name, &path_info, &query_string);

        let mut child = match Command::new(&self.command)
            .args(&self.args)
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
            match child.wait_timeout(POLL_INTERVAL) {
                Ok(Some(status)) => {
                    if !status.success() {
                        return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
                    }
                    break;
                }
                Ok(None) => {
                    if interrupt.is_interrupted() {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Some(HttpResponse::new(StatusCode::SERVICE_UNAVAILABLE));
                    }
                }
                Err(_) => return Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)),
            }
        }

        let mut output = Vec::new();
        if let Some(mut out) = child.stdout.take() {
            let _ = out.read_to_end(&mut output);
        }

        Some(parse_cgi_output(&output))
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

    fn make_context(path: &str, route_tail: &[&str]) -> Context {
        let raw = format!("GET {} HTTP/1.0\r\nHost: localhost\r\n\r\n", path);
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        Context::with_tail(
            request,
            route_tail.iter().map(|s| (*s).to_string()).collect(),
        )
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

        let handler = CgiHandler::new(script, vec![]);
        let ctx = make_context("/hello", &[]);
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

        let handler = CgiHandler::new(script, vec![]);
        let ctx = make_context("/notfound", &[]);
        let response = handler.process(ctx, &make_interrupt()).unwrap();

        assert_eq!(*response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn script_not_found_returns_404() {
        let handler = CgiHandler::new(PathBuf::from("/nonexistent/script"), vec![]);
        let ctx = make_context("/test", &[]);
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

        let handler = CgiHandler::new(script, vec![]);
        let raw = "GET /echo_qs?foo=bar HTTP/1.0\r\nHost: localhost\r\n\r\n";
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        let ctx = Context::with_tail(request, vec![]);

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

        let handler = CgiHandler::new(script, vec![]);
        let raw = "GET /article/2024/hi HTTP/1.0\r\nHost: localhost\r\n\r\n";
        let (request, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        let ctx = Context::with_tail(request, vec!["2024".to_string(), "hi".to_string()]);

        let response = handler.process(ctx, &make_interrupt()).unwrap();
        assert_eq!(*response.status(), StatusCode::OK);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("/2024/hi"));
    }
}
