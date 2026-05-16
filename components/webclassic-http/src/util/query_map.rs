use std::collections::HashMap;
use std::str::FromStr;

use snafu::Snafu;

use crate::util::percent_decode::decode_percent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryMap {
    inner: HashMap<String, Vec<String>>,
    len: usize,
}

impl QueryMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            len: 0,
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.inner.entry(key).or_default().push(value);
        self.len += 1;
    }

    pub fn with(mut self, key: String, value: String) -> Self {
        self.insert(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&[String]> {
        self.inner.get(key).map(|v| v.as_slice())
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &[String])> {
        self.inner.iter().map(|(k, v)| (k, v.as_slice()))
    }
}

impl Default for QueryMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Snafu)]
pub enum ParseQueryMapError {
    #[snafu(display("invalid percent encoding in query string"))]
    InvalidEncoding,
}

impl FromStr for QueryMap {
    type Err = ParseQueryMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = QueryMap::new();
        if s.is_empty() {
            return Ok(map);
        }
        for pair in s.split('&') {
            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };
            map.insert(decode_percent(key), decode_percent(value));
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let map: QueryMap = "a=1&b=2".parse().unwrap();
        assert_eq!(map.get("a"), Some(["1".to_string()].as_slice()));
        assert_eq!(map.get("b"), Some(["2".to_string()].as_slice()));
    }

    #[test]
    fn parse_empty_value() {
        let map: QueryMap = "key=".parse().unwrap();
        assert_eq!(map.get("key"), Some(["".to_string()].as_slice()));
    }

    #[test]
    fn parse_no_value() {
        let map: QueryMap = "key".parse().unwrap();
        assert_eq!(map.get("key"), Some(["".to_string()].as_slice()));
    }

    #[test]
    fn parse_empty_string() {
        let map: QueryMap = "".parse().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn multiple_values() {
        let map: QueryMap = "a=1&a=2&a=3".parse().unwrap();
        let values = map.get("a").unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "2");
        assert_eq!(values[2], "3");
    }

    #[test]
    fn percent_decoded() {
        let map: QueryMap = "name=hello%20world&msg=%E4%BD%A0%E5%A5%BD".parse().unwrap();
        assert_eq!(
            map.get("name"),
            Some(["hello world".to_string()].as_slice())
        );
        assert_eq!(map.get("msg"), Some(["你好".to_string()].as_slice()));
    }

    #[test]
    fn with_chaining() {
        let map = QueryMap::new()
            .with("a".to_string(), "1".to_string())
            .with("b".to_string(), "2".to_string());
        assert_eq!(map.len(), 2);
    }
}
