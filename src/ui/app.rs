use super::components::{chat::Chat, settings::Settings, sidebar::Sidebar};
use super::state::UIState;
use crate::chat::SessionManager;
use crate::llm::{LLMClient, LLMConfig};
use eframe::egui;
use std::sync::Arc;

pub struct App {
    llm_client: LLMClient,
    state: UIState,
    sidebar: Sidebar,
    chat: Chat,
    settings: Settings,
    runtime: Arc<tokio::runtime::Runtime>,
    session_manager: SessionManager,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let runtime = Arc::new(tokio::runtime::Runtime::new()?);
        let config = LLMConfig::default();

        // 创建一个默认会话
        let mut session_manager = SessionManager::new();
        let _default_session_id = session_manager.create_session("New Chat".to_string());

        Ok(Self {
            llm_client: LLMClient::new(config),
            state: UIState::default(),
            sidebar: Sidebar::new(),
            chat: Chat::new(),
            settings: Settings::default(),
            runtime,
            session_manager,
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理会话管理事件
        if self.state.new_chat_requested {
            self.session_manager.create_session("New Chat".to_string());
            self.state.new_chat_requested = false;
        }

        if let Some(id) = self.state.delete_chat_requested.take() {
            let _ = self.session_manager.delete_session(&id);
        }

        egui::SidePanel::left("sidebar")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.sidebar.ui(
                    ui,
                    &mut self.state,
                    &self.session_manager.get_all_sessions(),
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(session) = self.session_manager.get_current_session_mut() {
                self.chat
                    .ui(ui, &mut self.state, self.llm_client.clone(), session);
            }
        });

        // 设置窗口
        if self.state.show_settings {
            let mut show_settings = self.state.show_settings;
            egui::Window::new("Settings")
                .open(&mut show_settings)
                .show(ctx, |ui| {
                    self.settings.ui(ui, &mut self.state);
                });
            self.state.show_settings = show_settings;
        }
    }
}
