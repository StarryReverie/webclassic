use std::path::Path;

pub fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_extensions() {
        assert_eq!(
            guess_content_type(Path::new("index.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            guess_content_type(Path::new("style.css")),
            "text/css; charset=utf-8"
        );
        assert_eq!(
            guess_content_type(Path::new("app.js")),
            "application/javascript"
        );
        assert_eq!(
            guess_content_type(Path::new("data.json")),
            "application/json"
        );
        assert_eq!(guess_content_type(Path::new("photo.jpg")), "image/jpeg");
        assert_eq!(guess_content_type(Path::new("photo.jpeg")), "image/jpeg");
        assert_eq!(guess_content_type(Path::new("image.png")), "image/png");
        assert_eq!(guess_content_type(Path::new("icon.svg")), "image/svg+xml");
        assert_eq!(guess_content_type(Path::new("favicon.ico")), "image/x-icon");
        assert_eq!(guess_content_type(Path::new("font.woff2")), "font/woff2");
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            guess_content_type(Path::new("INDEX.HTML")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            guess_content_type(Path::new("Style.CSS")),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn unknown_extension() {
        assert_eq!(
            guess_content_type(Path::new("binary.dat")),
            "application/octet-stream"
        );
        assert_eq!(
            guess_content_type(Path::new("noext")),
            "application/octet-stream"
        );
    }
}
