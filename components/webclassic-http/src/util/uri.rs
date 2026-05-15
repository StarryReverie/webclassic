use std::str::FromStr;

use snafu::Snafu;

use crate::util::QueryMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    path: String,
    query: Option<QueryMap>,
}

impl Uri {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> Option<&QueryMap> {
        self.query.as_ref()
    }
}

#[derive(Debug, Snafu)]
pub enum ParseUriError {
    #[snafu(display("URI path must start with /"))]
    MissingLeadingSlash,
    #[snafu(display("invalid query string"))]
    InvalidQuery {
        source: crate::util::ParseQueryMapError,
    },
}

impl FromStr for Uri {
    type Err = ParseUriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (path, query) = match s.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (s, None),
        };

        if !path.starts_with('/') {
            return Err(ParseUriError::MissingLeadingSlash);
        }

        let query = match query {
            Some(q) => Some(
                q.parse()
                    .map_err(|e| ParseUriError::InvalidQuery { source: e })?,
            ),
            None => None,
        };

        Ok(Uri {
            path: path.to_string(),
            query,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_only() {
        let uri: Uri = "/foo/bar".parse().unwrap();
        assert_eq!(uri.path(), "/foo/bar");
        assert!(uri.query().is_none());
    }

    #[test]
    fn path_with_query() {
        let uri: Uri = "/search?q=hello&lang=en".parse().unwrap();
        assert_eq!(uri.path(), "/search");
        let query = uri.query().unwrap();
        assert_eq!(query.get("q"), Some(["hello".to_string()].as_slice()));
        assert_eq!(query.get("lang"), Some(["en".to_string()].as_slice()));
    }

    #[test]
    fn empty_query() {
        let uri: Uri = "/path?".parse().unwrap();
        assert_eq!(uri.path(), "/path");
        let query = uri.query().unwrap();
        assert!(query.is_empty());
    }

    #[test]
    fn root_path() {
        let uri: Uri = "/".parse().unwrap();
        assert_eq!(uri.path(), "/");
        assert!(uri.query().is_none());
    }

    #[test]
    fn missing_leading_slash() {
        assert!("foo/bar".parse::<Uri>().is_err());
    }
}
