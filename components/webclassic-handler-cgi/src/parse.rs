use webclassic_http::response::HttpResponse;
use webclassic_http::util::StatusCode;

pub fn parse_cgi_output(output: &[u8]) -> HttpResponse {
    if output.is_empty() {
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let (header_bytes, body_bytes) = match find_header_end(output) {
        Some(pos) => (
            &output[..pos],
            &output[pos + separator_len(&output[pos..])..],
        ),
        None => return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let header_str = match std::str::from_utf8(header_bytes) {
        Ok(s) => s,
        Err(_) => return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut status = StatusCode::OK;
    let mut response = HttpResponse::new(status);

    for line in header_str.lines() {
        let (name, value) = match line.split_once(':') {
            Some((n, v)) => (n.trim(), v.trim()),
            None => continue,
        };

        if name.eq_ignore_ascii_case("status") {
            if let Some(code) = parse_status_value(value) {
                status = code;
                response = HttpResponse::new(status);
            }
            continue;
        }

        response = response.with_header(name, value.to_string());
    }

    response = response.with_body(body_bytes.to_vec());
    response
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(1) {
        if data[i..].starts_with(b"\r\n\r\n") {
            return Some(i);
        }
        if data[i..].starts_with(b"\n\n") {
            return Some(i);
        }
    }
    None
}

fn separator_len(data: &[u8]) -> usize {
    if data.starts_with(b"\r\n\r\n") { 4 } else { 2 }
}

fn parse_status_value(value: &str) -> Option<StatusCode> {
    let code_str = value.split_whitespace().next()?;
    let code: u16 = code_str.parse().ok()?;
    Some(StatusCode::new(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_output_with_crlf() {
        let output = b"Content-Type: text/html\r\n\r\n<body>hello</body>";
        let resp = parse_cgi_output(output);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), b"<body>hello</body>");
    }

    #[test]
    fn standard_output_with_lf() {
        let output = b"Content-Type: text/html\n\n<body>hello</body>";
        let resp = parse_cgi_output(output);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), b"<body>hello</body>");
    }

    #[test]
    fn status_header() {
        let output = b"Status: 404 Not Found\r\nContent-Type: text/html\r\n\r\nNot found";
        let resp = parse_cgi_output(output);
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.body(), b"Not found");
    }

    #[test]
    fn multiple_headers() {
        let output = b"Content-Type: text/html\r\nX-Custom: foo\r\n\r\nok";
        let resp = parse_cgi_output(output);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap()[0], "text/html");
        assert_eq!(resp.headers().get("x-custom").unwrap()[0], "foo");
    }

    #[test]
    fn empty_output_returns_500() {
        let resp = parse_cgi_output(b"");
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn no_header_separator_returns_500() {
        let resp = parse_cgi_output(b"Content-Type: text/html");
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn empty_body() {
        let resp = parse_cgi_output(b"Content-Type: text/plain\r\n\r\n");
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.body().is_empty());
    }

    #[test]
    fn status_without_reason() {
        let resp = parse_cgi_output(b"Status: 201\r\n\r\ncreated");
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
