use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for DebugLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugLevel::Trace => write!(f, "TRACE"),
            DebugLevel::Debug => write!(f, "DEBUG"),
            DebugLevel::Info => write!(f, "INFO"),
            DebugLevel::Warn => write!(f, "WARN"),
            DebugLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugCategory {
    Network,
    Sync,
    State,
    Auth,
    Peer,
    Email,
    Thread,
    UI,
    Other,
}

impl fmt::Display for DebugCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugCategory::Network => write!(f, "NET"),
            DebugCategory::Sync => write!(f, "SYNC"),
            DebugCategory::State => write!(f, "STATE"),
            DebugCategory::Auth => write!(f, "AUTH"),
            DebugCategory::Peer => write!(f, "PEER"),
            DebugCategory::Email => write!(f, "EMAIL"),
            DebugCategory::Thread => write!(f, "THREAD"),
            DebugCategory::UI => write!(f, "UI"),
            DebugCategory::Other => write!(f, "OTHER"),
        }
    }
}

#[derive(Clone)]
pub struct DebugEntry {
    pub timestamp: String,
    pub level: DebugLevel,
    pub category: DebugCategory,
    pub thread_id: String,
    pub message: String,
    pub context: Option<String>,
}

impl fmt::Display for DebugEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let context_str = self
            .context
            .as_ref()
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        write!(
            f,
            "{} [{}] {} {} {}{} {}",
            self.timestamp,
            self.level,
            self.category,
            self.thread_id,
            self.message,
            context_str,
            if self.level == DebugLevel::Error {
                "❌"
            } else if self.level == DebugLevel::Warn {
                "⚠️"
            } else if self.level == DebugLevel::Info {
                "ℹ️"
            } else {
                ""
            }
        )
    }
}

pub struct DebugLogger {
    entries: Arc<Mutex<Vec<DebugEntry>>>,
    max_entries: usize,
}

impl DebugLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max_entries,
        }
    }

    fn get_thread_name() -> String {
        std::thread::current()
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                format!("TID:{:?}", std::thread::current().id())
            })
    }

    fn get_timestamp() -> String {
        use std::time::UNIX_EPOCH;
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        let hours = (secs / 3600) % 24;
        let mins = (secs / 60) % 60;
        let sec = secs % 60;
        format!("{:02}:{:02}:{:02}.{:03}", hours, mins, sec, millis)
    }

    pub fn log(
        &self,
        level: DebugLevel,
        category: DebugCategory,
        message: impl Into<String>,
        context: Option<String>,
    ) {
        let entry = DebugEntry {
            timestamp: Self::get_timestamp(),
            level,
            category,
            thread_id: Self::get_thread_name(),
            message: message.into(),
            context,
        };

        eprintln!("{}", entry);

        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);
            if entries.len() > self.max_entries {
                entries.remove(0);
            }
        }
    }

    pub fn trace(&self, category: DebugCategory, msg: impl Into<String>) {
        self.log(DebugLevel::Trace, category, msg, None);
    }

    pub fn debug(&self, category: DebugCategory, msg: impl Into<String>) {
        self.log(DebugLevel::Debug, category, msg, None);
    }

    pub fn info(&self, category: DebugCategory, msg: impl Into<String>) {
        self.log(DebugLevel::Info, category, msg, None);
    }

    pub fn warn(&self, category: DebugCategory, msg: impl Into<String>) {
        self.log(DebugLevel::Warn, category, msg, None);
    }

    pub fn error(&self, category: DebugCategory, msg: impl Into<String>) {
        self.log(DebugLevel::Error, category, msg, None);
    }

    pub fn info_ctx(
        &self,
        category: DebugCategory,
        msg: impl Into<String>,
        ctx: impl Into<String>,
    ) {
        self.log(DebugLevel::Info, category, msg, Some(ctx.into()));
    }

    pub fn error_ctx(
        &self,
        category: DebugCategory,
        msg: impl Into<String>,
        ctx: impl Into<String>,
    ) {
        self.log(DebugLevel::Error, category, msg, Some(ctx.into()));
    }

    pub fn debug_ctx(
        &self,
        category: DebugCategory,
        msg: impl Into<String>,
        ctx: impl Into<String>,
    ) {
        self.log(DebugLevel::Debug, category, msg, Some(ctx.into()));
    }

    pub fn warn_ctx(
        &self,
        category: DebugCategory,
        msg: impl Into<String>,
        ctx: impl Into<String>,
    ) {
        self.log(DebugLevel::Warn, category, msg, Some(ctx.into()));
    }

    pub fn get_entries(&self) -> Vec<DebugEntry> {
        self.entries.lock().map(|e| e.clone()).unwrap_or_default()
    }

    pub fn get_entries_by_category(&self, category: DebugCategory) -> Vec<DebugEntry> {
        self.entries
            .lock()
            .map(|e| {
                e.iter()
                    .filter(|entry| entry.category == category)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_entries_by_level(&self, level: DebugLevel) -> Vec<DebugEntry> {
        self.entries
            .lock()
            .map(|e| {
                e.iter()
                    .filter(|entry| entry.level == level)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_recent(&self, count: usize) -> Vec<DebugEntry> {
        self.entries
            .lock()
            .map(|e| {
                e.iter()
                    .rev()
                    .take(count)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn count(&self) -> usize {
        self.entries
            .lock()
            .map(|e| e.len())
            .unwrap_or_default()
    }
}

impl Clone for DebugLogger {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            max_entries: self.max_entries,
        }
    }
}
