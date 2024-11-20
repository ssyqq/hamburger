use super::session::ChatSession;
use crate::llm::message::Message;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SessionManager {
    sessions: HashMap<String, ChatSession>,
    current_session_id: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_session_id: None,
        }
    }

    pub fn create_session(&mut self, title: String) -> String {
        let id = Uuid::new_v4().to_string();
        let mut session = ChatSession::new(title);
        session.id = id.clone();
        self.sessions.insert(id.clone(), session);
        self.current_session_id = Some(id.clone());
        id
    }

    pub fn get_current_session(&self) -> Option<&ChatSession> {
        self.current_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    pub fn get_current_session_mut(&mut self) -> Option<&mut ChatSession> {
        self.current_session_id
            .as_ref()
            .and_then(|id| self.sessions.get_mut(id))
    }

    pub fn switch_session(&mut self, id: String) -> Result<()> {
        if self.sessions.contains_key(&id) {
            self.current_session_id = Some(id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    pub fn get_all_sessions(&self) -> Vec<&ChatSession> {
        self.sessions.values().collect()
    }

    pub fn add_message_to_current(&mut self, message: Message) -> Result<()> {
        if let Some(session) = self.get_current_session_mut() {
            session.add_message(message);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No active session"))
        }
    }

    pub fn delete_session(&mut self, id: &str) -> Result<()> {
        if self.sessions.remove(id).is_some() {
            if Some(id.to_string()) == self.current_session_id {
                self.current_session_id = self.sessions.keys().next().cloned();
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
}
