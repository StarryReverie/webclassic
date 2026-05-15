use std::collections::HashMap;
use std::str::FromStr;

use crate::util::HeaderName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderMap {
    inner: HashMap<HeaderName, Vec<String>>,
    len: usize,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            len: 0,
        }
    }

    pub fn insert(&mut self, name: &str, value: String) {
        let name = match HeaderName::from_str(name) {
            Ok(n) => n,
            Err(_) => return,
        };
        self.inner.entry(name).or_default().push(value);
        self.len += 1;
    }

    pub fn with(mut self, name: &str, value: String) -> Self {
        self.insert(name, value);
        self
    }

    pub fn get(&self, name: &str) -> Option<&[String]> {
        let name = HeaderName::from_str(name).ok()?;
        self.inner.get(&name).map(|v| v.as_slice())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &[String])> {
        self.inner.iter().map(|(k, v)| (k, v.as_slice()))
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Default for HeaderMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut map = HeaderMap::new();
        map.insert("Content-Type", "text/html".to_string());
        assert_eq!(
            map.get("Content-Type"),
            Some(["text/html".to_string()].as_slice())
        );
    }

    #[test]
    fn get_case_insensitive() {
        let mut map = HeaderMap::new();
        map.insert("Content-Type", "text/html".to_string());
        assert_eq!(
            map.get("content-type"),
            Some(["text/html".to_string()].as_slice())
        );
        assert_eq!(
            map.get("CONTENT-TYPE"),
            Some(["text/html".to_string()].as_slice())
        );
    }

    #[test]
    fn multiple_values() {
        let mut map = HeaderMap::new();
        map.insert("Accept", "text/html".to_string());
        map.insert("Accept", "application/json".to_string());
        let values = map.get("Accept").unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "text/html");
        assert_eq!(values[1], "application/json");
    }

    #[test]
    fn get_missing() {
        let map = HeaderMap::new();
        assert!(map.get("Content-Type").is_none());
    }

    #[test]
    fn with_chaining() {
        let map = HeaderMap::new()
            .with("Content-Type", "text/html".to_string())
            .with("Accept", "text/plain".to_string());
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn insert_invalid_name_is_ignored() {
        let mut map = HeaderMap::new();
        map.insert("Bad Name", "value".to_string());
        assert!(map.is_empty());
    }
}
