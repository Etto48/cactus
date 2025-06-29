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
    fn log_to_term(level: LogLevel, message: impl Into<String>) {
        let entry = LogEntry::new(level, message);
        let color = match level {
            LogLevel::Debug => "32", // Blue
            LogLevel::Info => "34", // Green
            LogLevel::Warning => "33", // Yellow
            LogLevel::Error => "31", // Red
        };
        println!("\x1b[90m{}\x1b[0m \x1b[{}m[{}]\x1b[0m {}", entry.fmt_timestamp(), color, entry.level.to_string(), entry.message);
    }

    pub fn log_d(&mut self, message: impl Into<String>) {
        let message = message.into();
        Self::log_to_term(LogLevel::Debug, message.clone());
        self.entries.push(LogEntry::new_d(message));
    }
    pub fn log_i(&mut self, message: impl Into<String>) {
        let message = message.into();
        Self::log_to_term(LogLevel::Info, message.clone());
        self.entries.push(LogEntry::new_i(message));
    }
    pub fn log_w(&mut self, message: impl Into<String>) {
        let message = message.into();
        Self::log_to_term(LogLevel::Warning, message.clone());
        self.entries.push(LogEntry::new_w(message));
    }
    pub fn log_e(&mut self, message: impl Into<String>) {
        let message = message.into();
        Self::log_to_term(LogLevel::Error, message.clone());
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
pub fn log_to_component(
    log: SyncSignal<Log>,
    mut last_log_message: Signal<Option<Event<MountedData>>>
) -> Element {
    let log = log.read();
    let log_len = log.len();
    rsx!{
        for (i, entry) in log.iter().enumerate() {
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
                            if i == log_len - 1 {
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