use webclassic_service::response::Response;

use crate::util::{HeaderMap, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.body
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

impl Response for HttpResponse {
    fn serialize(&self) -> Vec<u8> {
        let mut output = Vec::new();

        output.extend_from_slice(format!("HTTP/1.0 {}\r\n", self.status).as_bytes());

        if self.headers.get("content-length").is_none() && !self.body.is_empty() {
            output.extend_from_slice(format!("content-length: {}\r\n", self.body.len()).as_bytes());
        }

        for (name, values) in self.headers.iter() {
            for value in values {
                output.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
            }
        }

        output.extend_from_slice(b"\r\n");
        output.extend_from_slice(&self.body);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_200_empty() {
        let resp = HttpResponse::new(StatusCode::OK);
        let raw = String::from_utf8(resp.serialize()).unwrap();
        assert_eq!(raw, "HTTP/1.0 200 OK\r\n\r\n");
    }

    #[test]
    fn serialize_200_with_body() {
        let resp = HttpResponse::new(StatusCode::OK).with_body(b"hello".to_vec());
        let raw = String::from_utf8(resp.serialize()).unwrap();
        assert_eq!(raw, "HTTP/1.0 200 OK\r\ncontent-length: 5\r\n\r\nhello");
    }

    #[test]
    fn serialize_404() {
        let resp = HttpResponse::new(StatusCode::NOT_FOUND).with_body(b"Not Found".to_vec());
        let raw = String::from_utf8(resp.serialize()).unwrap();
        assert!(raw.starts_with("HTTP/1.0 404 Not Found\r\n"));
        assert!(raw.contains("content-length: 9"));
        assert!(raw.ends_with("Not Found"));
    }

    #[test]
    fn serialize_custom_header() {
        let resp = HttpResponse::new(StatusCode::OK)
            .with_header("Content-Type", "text/html".to_string())
            .with_body(b"<h1>Hi</h1>".to_vec());
        let raw = String::from_utf8(resp.serialize()).unwrap();
        assert!(raw.contains("content-type: text/html\r\n"));
        assert!(raw.contains("content-length: 11\r\n"));
    }

    #[test]
    fn serialize_manual_content_length_not_duplicated() {
        let resp = HttpResponse::new(StatusCode::OK)
            .with_header("Content-Length", "3".to_string())
            .with_body(b"abc".to_vec());
        let raw = String::from_utf8(resp.serialize()).unwrap();
        assert_eq!(raw, "HTTP/1.0 200 OK\r\ncontent-length: 3\r\n\r\nabc");
    }
}
