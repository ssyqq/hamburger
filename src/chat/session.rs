use crate::llm::message::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub messages: VecDeque<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
}

impl ChatSession {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            messages: VecDeque::new(),
            created_at: now,
            updated_at: now,
            stream_tx: None,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.updated_at = Utc::now();
    }
}
