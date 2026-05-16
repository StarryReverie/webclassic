use std::str::FromStr;

use snafu::Snafu;
use webclassic_service::request::Request;

use crate::util::{
    HeaderMap, HeaderName, Method, ParseHeaderNameError, ParseMethodError, ParseUriError, Uri,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(method: Method, uri: Uri) -> Self {
        Self {
            method,
            uri,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn with_header(mut self, name: &str, value: String) -> Self {
        self.headers.insert(name, value);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

#[derive(Debug, Snafu)]
pub enum ParseHttpRequestError {
    #[snafu(display("invalid request line"))]
    RequestLine,
    #[snafu(display("invalid HTTP method"))]
    Method { source: ParseMethodError },
    #[snafu(display("invalid URI"))]
    Uri { source: ParseUriError },
    #[snafu(display("invalid header name"))]
    HeaderName { source: ParseHeaderNameError },
    #[snafu(display("missing Content-Length for request body"))]
    MissingContentLength,
    #[snafu(display("invalid Content-Length"))]
    InvalidContentLength,
}

impl Request for HttpRequest {
    type Error = ParseHttpRequestError;

    fn deserialize(data: &[u8]) -> Result<Option<(Self, usize)>, Self::Error> {
        let header_end = match find_header_end(data) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let header_bytes = &data[..header_end];
        let header_str = match std::str::from_utf8(header_bytes) {
            Ok(s) => s,
            Err(_) => return Err(ParseHttpRequestError::RequestLine),
        };

        let consumed = header_end + 4;
        let (method, uri, headers) = parse_head(header_str)?;

        let body_len = headers
            .get("content-length")
            .and_then(|v| v.first())
            .map(|v| v.trim())
            .unwrap_or("0");
        let body_len: usize = body_len.parse().unwrap_or(0);

        if data.len() < consumed + body_len {
            return Ok(None);
        }

        let body = data[consumed..consumed + body_len].to_vec();

        Ok(Some((
            HttpRequest {
                method,
                uri,
                headers,
                body,
            },
            consumed + body_len,
        )))
    }
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if data[i..i + 4] == [b'\r', b'\n', b'\r', b'\n'] {
            return Some(i);
        }
    }
    None
}

fn parse_head(head: &str) -> Result<(Method, Uri, HeaderMap), ParseHttpRequestError> {
    let mut lines = head.split("\r\n");

    let request_line = lines.next().ok_or(ParseHttpRequestError::RequestLine)?;
    let mut parts = request_line.splitn(3, ' ');
    let method_str = parts.next().ok_or(ParseHttpRequestError::RequestLine)?;
    let uri_str = parts.next().ok_or(ParseHttpRequestError::RequestLine)?;

    let method =
        Method::from_str(method_str).map_err(|e| ParseHttpRequestError::Method { source: e })?;
    let uri = Uri::from_str(uri_str).map_err(|e| ParseHttpRequestError::Uri { source: e })?;

    let mut headers = HeaderMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = match line.split_once(':') {
            Some((n, v)) => (n, v.trim()),
            None => continue,
        };
        let _validated = HeaderName::from_str(name)
            .map_err(|e| ParseHttpRequestError::HeaderName { source: e })?;
        headers.insert(name, value.to_string());
    }

    Ok((method, uri, headers))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_request() {
        let raw = b"GET /index.html HTTP/1.0\r\nHost: localhost\r\n\r\n";
        let (req, consumed) = HttpRequest::deserialize(raw).unwrap().unwrap();
        assert_eq!(consumed, raw.len());
        assert_eq!(req.method(), Method::Get);
        assert_eq!(req.uri().path(), "/index.html");
        assert_eq!(
            req.headers().get("host"),
            Some(["localhost".to_string()].as_slice())
        );
        assert!(req.body().is_empty());
    }

    #[test]
    fn parse_post_with_body() {
        let body = b"name=hello";
        let raw = format!(
            "POST /submit HTTP/1.0\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let mut data = raw.into_bytes();
        data.extend_from_slice(body);

        let (req, consumed) = HttpRequest::deserialize(&data).unwrap().unwrap();
        assert_eq!(consumed, data.len());
        assert_eq!(req.method(), Method::Post);
        assert_eq!(req.uri().path(), "/submit");
        assert_eq!(req.body(), body);
    }

    #[test]
    fn parse_with_query() {
        let raw = b"GET /search?q=hello HTTP/1.0\r\n\r\n";
        let (req, _) = HttpRequest::deserialize(raw).unwrap().unwrap();
        let query = req.uri().query().unwrap();
        assert_eq!(query.get("q"), Some(["hello".to_string()].as_slice()));
    }

    #[test]
    fn incomplete_headers() {
        let raw = b"GET / HTTP/1.0\r\nHost: localhost\r\n";
        let result = HttpRequest::deserialize(raw).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn incomplete_body() {
        let raw = b"POST / HTTP/1.0\r\nContent-Length: 100\r\n\r\nshort";
        let result = HttpRequest::deserialize(raw).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn multiple_headers() {
        let raw = b"GET / HTTP/1.0\r\nAccept: text/html\r\nAccept: application/json\r\n\r\n";
        let (req, _) = HttpRequest::deserialize(raw).unwrap().unwrap();
        let values = req.headers().get("accept").unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "text/html");
        assert_eq!(values[1], "application/json");
    }

    #[test]
    fn builder_basic() {
        let req = HttpRequest::new(Method::Get, Uri::from_str("/index.html").unwrap());
        assert_eq!(req.method(), Method::Get);
        assert_eq!(req.uri().path(), "/index.html");
        assert!(req.headers().is_empty());
        assert!(req.body().is_empty());
    }

    #[test]
    fn builder_with_header_and_body() {
        let req = HttpRequest::new(Method::Post, Uri::from_str("/submit").unwrap())
            .with_header("Host", "localhost".to_string())
            .with_header("Content-Type", "application/json".to_string())
            .with_body(br#"{"key":"value"}"#.to_vec());
        assert_eq!(req.method(), Method::Post);
        assert_eq!(req.uri().path(), "/submit");
        assert_eq!(
            req.headers().get("host"),
            Some(["localhost".to_string()].as_slice())
        );
        assert_eq!(
            req.headers().get("content-type"),
            Some(["application/json".to_string()].as_slice())
        );
        assert_eq!(req.body(), br#"{"key":"value"}"#);
    }

    #[test]
    fn builder_with_method() {
        let req = HttpRequest::new(Method::Head, Uri::from_str("/page").unwrap())
            .with_method(Method::Get);
        assert_eq!(req.method(), Method::Get);
        assert_eq!(req.uri().path(), "/page");
    }
}
