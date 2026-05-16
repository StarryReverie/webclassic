use std::sync::{Arc, Mutex};

use webclassic::web::filter::{LogBackend, LogEntry};

pub struct MemoryLogBackend {
    lines: Mutex<Vec<String>>,
}

impl MemoryLogBackend {
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(Vec::new()),
        }
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
}

impl LogBackend for MemoryLogBackend {
    fn log(&self, entry: &LogEntry) {
        let ip = entry.remote_addr.as_deref().unwrap_or("-");
        let ts = entry.timestamp.format("%d/%b/%Y:%H:%M:%S %z");
        let referer = entry.referer.as_deref().unwrap_or("-");
        let ua = entry.user_agent.as_deref().unwrap_or("-");
        let line = format!(
            "{} - - [{}] \"{} {} HTTP/1.0\" {} {} \"{}\" \"{}\"",
            ip, ts, entry.method, entry.path, entry.status, entry.size, referer, ua,
        );
        self.lines.lock().unwrap().push(line);
    }
}

pub fn create_log_backend() -> Arc<MemoryLogBackend> {
    Arc::new(MemoryLogBackend::new())
}
