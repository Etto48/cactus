use std::{collections::HashMap, net::SocketAddr, time::SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Sent,
    Received,
}

impl MessageDirection {
    pub fn is_received(&self) -> bool {
        matches!(self, MessageDirection::Received)
    }

    pub fn is_sent(&self) -> bool {
        matches!(self, MessageDirection::Sent)
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            MessageDirection::Sent => "sent",
            MessageDirection::Received => "received",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub timestamp: SystemTime,
    pub direction: MessageDirection,
    pub content: String,
}

impl ChatMessage {
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

#[derive(Debug, Clone)]
pub struct Chats {
    pub chats: HashMap<SocketAddr, Vec<ChatMessage>>
}

impl Default for Chats {
    fn default() -> Self {
        Chats {
            chats: HashMap::new(),
        }
    }
}

impl Chats {
    pub fn add_message(&mut self, address: SocketAddr, direction: MessageDirection, content: String) {
        if !self.chats.contains_key(&address) {
            self.chats.insert(address, Vec::new());
        }
        if let Some(messages) = self.chats.get_mut(&address) {
            messages.push(ChatMessage {
                timestamp: SystemTime::now(),
                direction,
                content,
            });
        }
    }

    pub fn clear_chat(&mut self, address: &SocketAddr) {
        self.chats.remove(address);
    }

    pub fn get_messages(&self, address: &SocketAddr) -> Option<&Vec<ChatMessage>> {
        self.chats.get(address)
    }
}
