use std::{fmt::Display, str::FromStr, time::SystemTime};

use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
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

impl FromStr for LogLevel {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warning" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid log level: {}", s),
            )),
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
    pub notification: Option<LogLevel>,
    pub level: LogLevel,
}

impl Default for Log {
    fn default() -> Self {
        Log { 
            entries: Vec::new(),
            notification: None,
            level: LogLevel::Debug,
        }
    }
}

impl Log {
    fn log_to_term(entry: &LogEntry) {
        let color = match entry.level {
            LogLevel::Debug => "32", // Blue
            LogLevel::Info => "34", // Green
            LogLevel::Warning => "33", // Yellow
            LogLevel::Error => "31", // Red
        };
        println!("\x1b[90m{}\x1b[0m \x1b[{}m[{}]\x1b[0m {}", entry.fmt_timestamp(), color, entry.level, entry.message);
    }

    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let entry = LogEntry::new(level, message);
        Self::log_to_term(&entry);
        if (self.notification.is_none() || self.notification.unwrap() < level) && level >= self.level {
            self.notification = Some(level);
        }
        self.entries.push(entry);
    }
        

    pub fn log_d(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }
    pub fn log_i(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }
    pub fn log_w(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }
    pub fn log_e(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().filter(|e| e.level >= self.level)
    }
    pub fn len(&self) -> usize {
        if self.level == LogLevel::Debug {
            return self.entries.len();
        }
        self.entries.iter().filter(|e| e.level >= self.level).count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get_last_message(&self) -> Option<String> {
        if let Some(last_entry) = self.entries
            .iter().rev()
            .filter(|e| e.level >= self.level)
            .next() {
            Some(format!("{}: {}", last_entry.level, last_entry.message))
        } else {
            None
        }
    }
    pub fn reset_notification(&mut self) {
        self.notification = None;
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.notification = None;
    }
}


#[component]
pub fn log_to_component(
    log: SyncSignal<Log>,
    mut last_log_message: Signal<Option<Event<MountedData>>>
) -> Element {
    let log_read = log.read();
    let log_len = log_read.len();
    rsx!{
        for (i, entry) in log_read.iter().enumerate() {
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
                                log.write().reset_notification();
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