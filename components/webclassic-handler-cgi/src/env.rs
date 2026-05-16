use std::collections::HashMap;

use webclassic_http::request::HttpRequest;

pub fn build_cgi_env(
    request: &HttpRequest,
    script_name: &str,
    path_info: &str,
    query_string: &str,
) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("GATEWAY_INTERFACE".to_string(), "CGI/1.1".to_string());
    env.insert("SERVER_PROTOCOL".to_string(), "HTTP/1.0".to_string());
    env.insert("SERVER_SOFTWARE".to_string(), "WebClassic".to_string());
    env.insert("REQUEST_METHOD".to_string(), request.method().to_string());
    env.insert("SCRIPT_NAME".to_string(), script_name.to_string());
    env.insert("PATH_INFO".to_string(), path_info.to_string());
    env.insert("QUERY_STRING".to_string(), query_string.to_string());

    if let Some(content_type) = request
        .headers()
        .get("content-type")
        .and_then(|v| v.first())
    {
        env.insert("CONTENT_TYPE".to_string(), content_type.to_string());
    }

    if !request.body().is_empty() {
        env.insert(
            "CONTENT_LENGTH".to_string(),
            request.body().len().to_string(),
        );
    }

    if let Some(host_values) = request.headers().get("host")
        && let Some(host) = host_values.first()
    {
        let (server_name, server_port) = match host.split_once(':') {
            Some((name, port)) => (name.to_string(), port.to_string()),
            None => (host.to_string(), "80".to_string()),
        };
        env.insert("SERVER_NAME".to_string(), server_name);
        env.insert("SERVER_PORT".to_string(), server_port);
    }

    for (name, values) in request.headers().iter() {
        let name = name.as_str().to_uppercase();
        if name == "CONTENT-TYPE" || name == "CONTENT-LENGTH" || name == "HOST" {
            continue;
        }
        let var_name = format!("HTTP_{}", name.replace('-', "_"));
        if let Some(value) = values.first() {
            env.insert(var_name, value.to_string());
        }
    }

    env
}

#[cfg(test)]
mod tests {
    use webclassic_http::request::HttpRequest;
    use webclassic_service::request::Request;

    use super::*;

    fn parse_request(raw: &str) -> HttpRequest {
        let (req, _) = HttpRequest::deserialize(raw.as_bytes()).unwrap().unwrap();
        req
    }

    #[test]
    fn basic_get_request() {
        let request = parse_request("GET /hello HTTP/1.0\r\nHost: localhost:8080\r\n\r\n");
        let env = build_cgi_env(&request, "/hello", "", "");

        assert_eq!(env.get("GATEWAY_INTERFACE").unwrap(), "CGI/1.1");
        assert_eq!(env.get("SERVER_PROTOCOL").unwrap(), "HTTP/1.0");
        assert_eq!(env.get("SERVER_SOFTWARE").unwrap(), "WebClassic");
        assert_eq!(env.get("REQUEST_METHOD").unwrap(), "GET");
        assert_eq!(env.get("SCRIPT_NAME").unwrap(), "/hello");
        assert_eq!(env.get("PATH_INFO").unwrap(), "");
        assert_eq!(env.get("QUERY_STRING").unwrap(), "");
        assert_eq!(env.get("SERVER_NAME").unwrap(), "localhost");
        assert_eq!(env.get("SERVER_PORT").unwrap(), "8080");
        assert!(!env.contains_key("CONTENT_TYPE"));
        assert!(!env.contains_key("CONTENT_LENGTH"));
    }

    #[test]
    fn post_with_body() {
        let body = b"name=hello";
        let raw = format!(
            "POST /submit HTTP/1.0\r\nHost: example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let mut data = raw.into_bytes();
        data.extend_from_slice(body);
        let (request, _) = HttpRequest::deserialize(&data).unwrap().unwrap();

        let env = build_cgi_env(&request, "/submit", "", "");

        assert_eq!(env.get("REQUEST_METHOD").unwrap(), "POST");
        assert_eq!(
            env.get("CONTENT_TYPE").unwrap(),
            "application/x-www-form-urlencoded"
        );
        assert_eq!(env.get("CONTENT_LENGTH").unwrap(), "10");
    }

    #[test]
    fn query_string() {
        let request = parse_request("GET /search?q=hello&lang=en HTTP/1.0\r\n\r\n");
        let env = build_cgi_env(&request, "/search", "", "q=hello&lang=en");
        assert_eq!(env.get("QUERY_STRING").unwrap(), "q=hello&lang=en");
    }

    #[test]
    fn path_info() {
        let request = parse_request("GET /article/2024/hi HTTP/1.0\r\nHost: localhost\r\n\r\n");
        let env = build_cgi_env(&request, "/article", "/2024/hi", "");
        assert_eq!(env.get("SCRIPT_NAME").unwrap(), "/article");
        assert_eq!(env.get("PATH_INFO").unwrap(), "/2024/hi");
    }

    #[test]
    fn host_without_port() {
        let request = parse_request("GET / HTTP/1.0\r\nHost: example.com\r\n\r\n");
        let env = build_cgi_env(&request, "/", "", "");
        assert_eq!(env.get("SERVER_NAME").unwrap(), "example.com");
        assert_eq!(env.get("SERVER_PORT").unwrap(), "80");
    }

    #[test]
    fn http_headers_mapped() {
        let request = parse_request(
            "GET / HTTP/1.0\r\nHost: localhost\r\nAccept: text/html\r\nX-Custom: foobar\r\n\r\n",
        );
        let env = build_cgi_env(&request, "/", "", "");

        assert_eq!(env.get("HTTP_ACCEPT").unwrap(), "text/html");
        assert_eq!(env.get("HTTP_X_CUSTOM").unwrap(), "foobar");
    }

    #[test]
    fn content_type_and_host_not_duplicated_as_http() {
        let request =
            parse_request("GET / HTTP/1.0\r\nHost: localhost\r\nContent-Type: text/html\r\n\r\n");
        let env = build_cgi_env(&request, "/", "", "");

        assert!(!env.contains_key("HTTP_HOST"));
        assert!(!env.contains_key("HTTP_CONTENT_TYPE"));
        assert!(env.contains_key("CONTENT_TYPE"));
    }
}
