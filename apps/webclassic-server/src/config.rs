use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use webclassic::web::protocol::util::Method;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen: String,
    pub content: ContentConfig,
    #[serde(default)]
    pub cgi: Vec<CgiEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ContentConfig {
    pub root: PathBuf,
    pub index: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CgiEntry {
    pub methods: Vec<String>,
    pub prefix: String,
    pub program: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
}

impl CgiEntry {
    pub fn parse_methods(&self) -> Result<Vec<Method>, Box<dyn Error + Send + Sync>> {
        self.methods
            .iter()
            .map(|s| {
                s.parse::<Method>().map_err(|e| {
                    format!(
                        "invalid method '{}' in CGI entry '{}': {}",
                        s, self.prefix, e
                    )
                    .into()
                })
            })
            .collect()
    }
}

pub fn load(path: &Path) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut config = toml::from_str::<Config>(&content)?;

    let base = path.parent().unwrap_or(Path::new("."));
    config.content.root = resolve_path(base, &config.content.root);
    for entry in &mut config.cgi {
        entry.program = resolve_path(base, &entry.program);
    }

    Ok(config)
}

fn resolve_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_relative() {
        base.join(path)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let config: Config = toml::from_str(
            r#"
            listen = "127.0.0.1:8080"
            [content]
            root = "/var/www/html"
        "#,
        )
        .unwrap();
        assert_eq!(config.listen, "127.0.0.1:8080");
        assert_eq!(config.content.root, PathBuf::from("/var/www/html"));
        assert!(config.content.index.is_none());
        assert!(config.cgi.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let config: Config = toml::from_str(
            r#"
            listen = "0.0.0.0:3000"
            [content]
            root = "/var/www"
            index = "index.html"
            [[cgi]]
            methods = ["GET", "POST"]
            prefix = "/cgi-bin/hello"
            program = "/usr/local/bin/hello.sh"
            [[cgi]]
            methods = ["GET"]
            prefix = "/cgi-bin/search"
            program = "python"
            args = ["/opt/cgi/search.py"]
        "#,
        )
        .unwrap();
        assert_eq!(config.listen, "0.0.0.0:3000");
        assert_eq!(config.content.index.as_deref(), Some("index.html"));
        assert_eq!(config.cgi.len(), 2);

        let entry = &config.cgi[0];
        assert_eq!(entry.methods, vec!["GET", "POST"]);
        assert_eq!(entry.prefix, "/cgi-bin/hello");
        assert_eq!(entry.program, PathBuf::from("/usr/local/bin/hello.sh"));
        assert!(entry.args.is_empty());

        let entry = &config.cgi[1];
        assert_eq!(entry.args, vec!["/opt/cgi/search.py"]);
    }

    #[test]
    fn parse_methods_valid() {
        let entry = CgiEntry {
            methods: vec!["GET".to_string(), "POST".to_string()],
            prefix: "/test".to_string(),
            program: PathBuf::from("/bin/test"),
            args: vec![],
        };
        let methods = entry.parse_methods().unwrap();
        assert_eq!(methods, vec![Method::Get, Method::Post]);
    }

    #[test]
    fn parse_methods_case_insensitive() {
        let entry = CgiEntry {
            methods: vec!["get".to_string(), "post".to_string()],
            prefix: "/test".to_string(),
            program: PathBuf::from("/bin/test"),
            args: vec![],
        };
        let methods = entry.parse_methods().unwrap();
        assert_eq!(methods, vec![Method::Get, Method::Post]);
    }

    #[test]
    fn parse_methods_invalid() {
        let entry = CgiEntry {
            methods: vec!["PATCH".to_string()],
            prefix: "/test".to_string(),
            program: PathBuf::from("/bin/test"),
            args: vec![],
        };
        assert!(entry.parse_methods().is_err());
    }

    #[test]
    fn resolve_relative_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("server.toml");
        fs::write(
            &config_path,
            r#"
listen = "127.0.0.1:8080"
[content]
root = "./public"
index = "index.html"
[[cgi]]
methods = ["GET"]
prefix = "/cgi-bin/hello"
program = "./cgi-bin/hello.sh"
"#,
        )
        .unwrap();

        let config = load(&config_path).unwrap();
        assert_eq!(config.content.root, tmp.path().join("public"));
        assert_eq!(config.cgi[0].program, tmp.path().join("cgi-bin/hello.sh"));
    }

    #[test]
    fn keep_absolute_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("server.toml");
        fs::write(
            &config_path,
            r#"
listen = "127.0.0.1:8080"
[content]
root = "/var/www/html"
[[cgi]]
methods = ["GET"]
prefix = "/cgi-bin/hello"
program = "/usr/local/bin/hello.sh"
"#,
        )
        .unwrap();

        let config = load(&config_path).unwrap();
        assert_eq!(config.content.root, PathBuf::from("/var/www/html"));
        assert_eq!(config.cgi[0].program, PathBuf::from("/usr/local/bin/hello.sh"));
    }
}
