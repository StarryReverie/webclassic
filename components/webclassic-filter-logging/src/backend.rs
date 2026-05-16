use chrono::Local;
use webclassic_http::util::Method;

pub trait LogBackend: Send + Sync {
    fn log(&self, entry: &LogEntry);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub remote_addr: Option<String>,
    pub timestamp: chrono::DateTime<Local>,
    pub method: Method,
    pub path: String,
    pub status: u16,
    pub size: usize,
    pub referer: Option<String>,
    pub user_agent: Option<String>,
}
