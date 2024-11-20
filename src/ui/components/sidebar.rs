use crate::chat::session::ChatSession;
use crate::ui::state::UIState;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Ui};

#[derive(Debug, Clone, Default)]
pub struct ChatInfo {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct Sidebar {
    chats: Vec<ChatInfo>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &mut UIState, sessions: &[&ChatSession]) {
        ui.vertical(|ui| {
            if ui.button("New Chat").clicked() {
                state.new_chat_requested = true;
            }

            ui.separator();

            for session in sessions {
                let is_current = state.current_chat_id == Some(session.id.clone());
                if ui.selectable_label(is_current, &session.title).clicked() {
                    state.current_chat_id = Some(session.id.clone());
                }

                if ui.small_button("🗑").clicked() {
                    state.delete_chat_requested = Some(session.id.clone());
                }
            }

            ui.separator();

            if ui.button("Settings").clicked() {
                state.show_settings = true;
            }
        });
    }
}
