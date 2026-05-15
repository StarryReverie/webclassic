use std::path::{Path, PathBuf};

use webclassic_http::util::Method;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    method: Method,
    path: PathPattern,
}

impl Route {
    pub fn new(method: Method, path: PathPattern) -> Self {
        Self { method, path }
    }

    pub fn by(method: Method) -> RouteBuilder {
        RouteBuilder { method }
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn path(&self) -> &PathPattern {
        &self.path
    }

    pub fn test(&self, method: Method, path: &str) -> Option<MatchMetric> {
        if self.method == method {
            self.path.test(path)
        } else {
            None
        }
    }
}

pub struct RouteBuilder {
    method: Method,
}

impl RouteBuilder {
    pub fn prefix<P>(self, path: P) -> Route
    where
        P: Into<PathBuf>,
    {
        Route::new(self.method, PathPattern::Prefix(path.into()))
    }

    pub fn equal<P>(self, path: P) -> Route
    where
        P: Into<PathBuf>,
    {
        Route::new(self.method, PathPattern::Equal(path.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathPattern {
    Prefix(PathBuf),
    Equal(PathBuf),
}

impl PathPattern {
    pub fn test(&self, path: &str) -> Option<MatchMetric> {
        match &self {
            PathPattern::Prefix(pattern) => {
                if Path::new(path).starts_with(pattern) {
                    Some(MatchMetric(pattern.iter().count()))
                } else {
                    None
                }
            }
            PathPattern::Equal(pattern) => {
                if path == pattern {
                    Some(MatchMetric(pattern.iter().count()))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MatchMetric(usize);

impl MatchMetric {
    pub fn is_better_than(self, other: Self) -> bool {
        self.0 > other.0
    }

    pub fn best(self, other: Self) -> Self {
        if self.is_better_than(other) {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use webclassic_http::util::Method;

    use super::*;

    #[test]
    fn prefix_match() {
        let route = Route::by(Method::Get).prefix("/api");
        assert!(route.test(Method::Get, "/api").is_some());
        assert!(route.test(Method::Get, "/api/v1").is_some());
        assert!(route.test(Method::Get, "/v1").is_none());
        assert!(route.test(Method::Get, "/apis").is_none());
    }

    #[test]
    fn equal_match() {
        let route = Route::by(Method::Get).equal("/api");
        assert!(route.test(Method::Get, "/api").is_some());
        assert!(route.test(Method::Get, "/api/v1").is_none());
    }

    #[test]
    fn route_method_filter() {
        let route = Route::by(Method::Get).prefix("/api");
        assert!(route.test(Method::Get, "/api").is_some());
        assert!(route.test(Method::Post, "/api").is_none());
    }

    #[test]
    fn longer_prefix_is_better() {
        let short = Route::by(Method::Get).prefix("/api");
        let long = Route::by(Method::Get).prefix("/api/v1");
        let short_metric = short.test(Method::Get, "/api/v1/users").unwrap();
        let long_metric = long.test(Method::Get, "/api/v1/users").unwrap();
        assert!(long_metric.is_better_than(short_metric));
    }
}
