use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    Hello(String),
    Text(String)
}

impl Message {
    pub fn hello(name: impl Into<String>) -> Self {
        Message::Hello(name.into())
    }

    pub fn text(text: impl Into<String>) -> Self {
        Message::Text(text.into())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let bytes = bitcode::serialize(self).expect("Failed to serialize message");
        let len = (bytes.len() as u64).to_le_bytes();
        let mut result = Vec::with_capacity(len.len() + bytes.len());
        result.extend_from_slice(&len);
        result.extend_from_slice(&bytes);
        result
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None; // Not enough data for length prefix
        }
        let len = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
        if data.len() < len + 8 {
            return None; // Not enough data for the full message
        }
        let message_data = &data[8..8 + len];
        bitcode::deserialize(message_data).ok()
    }
    
}