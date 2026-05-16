use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::num::NonZero;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use webclassic::web::protocol::util::Method;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: NonZero<usize>,
    #[serde(default = "default_max_pending")]
    pub max_pending: NonZero<usize>,
    pub content: ContentConfig,
    #[serde(default)]
    pub cgi: Vec<CgiEntry>,
    #[serde(default)]
    pub error_pages: HashMap<String, String>,
}

fn default_max_connections() -> NonZero<usize> {
    NonZero::new(32).unwrap()
}

fn default_max_pending() -> NonZero<usize> {
    NonZero::new(128).unwrap()
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
    fn load_full_config_with_resolved_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let public = tmp.path().join("public");
        fs::create_dir_all(&public).unwrap();
        let cgi_bin = tmp.path().join("cgi-bin");
        fs::create_dir_all(&cgi_bin).unwrap();

        fs::write(
            tmp.path().join("server.toml"),
            r#"
listen = "0.0.0.0:3000"
max_connections = 64
max_pending = 256

[content]
root = "./public"
index = "index.html"

[[cgi]]
methods = ["GET", "POST"]
prefix = "/cgi-bin/hello"
program = "./cgi-bin/hello.sh"

[[cgi]]
methods = ["GET"]
prefix = "/cgi-bin/search"
program = "python"
args = ["/opt/cgi/search.py"]

[error_pages]
"404" = "404.html"
"403" = "403.html"
"#,
        )
        .unwrap();

        let config = load(&tmp.path().join("server.toml")).unwrap();

        assert_eq!(config.listen, "0.0.0.0:3000");
        assert_eq!(config.max_connections, NonZero::new(64).unwrap());
        assert_eq!(config.max_pending, NonZero::new(256).unwrap());

        assert_eq!(config.content.root, public);
        assert_eq!(config.content.index.as_deref(), Some("index.html"));

        assert_eq!(config.cgi.len(), 2);
        assert_eq!(config.cgi[0].methods, vec!["GET", "POST"]);
        assert_eq!(config.cgi[0].prefix, "/cgi-bin/hello");
        assert_eq!(config.cgi[0].program, tmp.path().join("cgi-bin/hello.sh"));
        assert!(config.cgi[0].args.is_empty());
        assert_eq!(config.cgi[1].methods, vec!["GET"]);
        assert_eq!(config.cgi[1].args, vec!["/opt/cgi/search.py"]);

        let methods = config.cgi[0].parse_methods().unwrap();
        assert_eq!(methods, vec![Method::Get, Method::Post]);

        assert_eq!(config.error_pages.get("404"), Some(&"404.html".to_string()));
        assert_eq!(config.error_pages.get("403"), Some(&"403.html".to_string()));
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
}
