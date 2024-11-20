use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image {
        text: String,
        url: String,
    },
    Function {
        name: String,
        arguments: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
}

// 新增：流式消息类型
#[derive(Debug, Clone)]
pub enum StreamMessage {
    Chunk(String),         // 部分响应
    Done(Message),         // 完整消息
    Error(String),         // 错误信息
}
