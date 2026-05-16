use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AppState {
    links: Mutex<HashMap<String, String>>,
    counter: AtomicU64,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            links: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(0),
        }
    }

    pub fn shorten(&self, url: String) -> String {
        let code = format!("{:x}", self.counter.fetch_add(1, Ordering::Relaxed));
        self.links.lock().unwrap().insert(code.clone(), url);
        code
    }

    pub fn resolve(&self, code: &str) -> Option<String> {
        self.links.lock().unwrap().get(code).cloned()
    }

    pub fn list(&self) -> Vec<(String, String)> {
        let map = self.links.lock().unwrap();
        let mut entries: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_generates_hex_codes() {
        let state = AppState::new();
        let a = state.shorten("https://a.com".into());
        let b = state.shorten("https://b.com".into());
        assert_eq!(a, "0");
        assert_eq!(b, "1");
    }

    #[test]
    fn resolve_returns_original_url() {
        let state = AppState::new();
        let code = state.shorten("https://example.com".into());
        assert_eq!(state.resolve(&code), Some("https://example.com".into()));
    }

    #[test]
    fn resolve_unknown_returns_none() {
        let state = AppState::new();
        assert_eq!(state.resolve("nope"), None);
    }

    #[test]
    fn list_returns_sorted_entries() {
        let state = AppState::new();
        state.shorten("https://c.com".into());
        state.shorten("https://a.com".into());
        state.shorten("https://b.com".into());
        let entries = state.list();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0, "0");
        assert_eq!(entries[1].0, "1");
        assert_eq!(entries[2].0, "2");
    }
}
