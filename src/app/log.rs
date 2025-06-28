use std::{fmt::Display, time::SystemTime};

use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: SystemTime,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        LogEntry {
            level,
            message: message.into(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn new_d(message: impl Into<String>) -> Self {
        LogEntry::new(LogLevel::Debug, message)
    }
    pub fn new_i(message: impl Into<String>) -> Self {
        LogEntry::new(LogLevel::Info, message)
    }
    pub fn new_w(message: impl Into<String>) -> Self {
        LogEntry::new(LogLevel::Warning, message)
    }
    pub fn new_e(message: impl Into<String>) -> Self {
        LogEntry::new(LogLevel::Error, message)
    }
    pub fn fmt_timestamp(&self) -> String {
        let now = chrono::DateTime::<chrono::Local>::from(SystemTime::now());
        let ts = chrono::DateTime::<chrono::Local>::from(self.timestamp);
        if now.date_naive() == ts.date_naive() {
            format!("{}", ts.format("%H:%M:%S"))
        } else {
            format!("{}", ts.format("%Y/%m/%d %H:%M:%S"))
        }
    }
}

pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Default for Log {
    fn default() -> Self {
        Log { entries: Vec::new() }
    }
}

impl Log {
    pub fn log_d(&mut self, message: impl Into<String>) {
        self.entries.push(LogEntry::new_d(message));
    }
    pub fn log_i(&mut self, message: impl Into<String>) {
        self.entries.push(LogEntry::new_i(message));
    }
    pub fn log_w(&mut self, message: impl Into<String>) {
        self.entries.push(LogEntry::new_w(message));
    }
    pub fn log_e(&mut self, message: impl Into<String>) {
        self.entries.push(LogEntry::new_e(message));
    }
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}


#[component]
pub fn log_to_component(log: SyncSignal<Log>, last_log_message: Signal<Option<Event<MountedData>>>) -> Element {
    rsx!{
        for (i, entry) in log.read().iter().enumerate() {
            div {
                class: "log-entry",
                div {
                    class: "log-content",
                    span {
                        class: match entry.level {
                            LogLevel::Debug => "log-level debug",
                            LogLevel::Info => "log-level info",
                            LogLevel::Warning => "log-level warning",
                            LogLevel::Error => "log-level error",
                        },
                        onmounted: move |e| {
                            if i == log.read().len() - 1 {
                                last_log_message.set(Some(e));
                            }
                        },
                        "{entry.level}"
                    }
                    span {
                        class: "log-message",
                        "{entry.message}"
                    }
                }
                span {
                    class: "log-timestamp",
                    "{entry.fmt_timestamp()}"
                }
            }
        }
    }
}