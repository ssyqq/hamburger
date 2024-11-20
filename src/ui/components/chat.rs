use eframe::egui::{self, ScrollArea, Ui};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use crate::{
    chat::ChatSession,
    llm::{
        client::LLMClient,
        message::{Message, MessageContent, Role, StreamMessage},
    },
    ui::state::UIState,
};

pub enum ChatMessage {
    StreamChunk(String),
    // 其他消息类型...
}

pub struct Chat {
    messages: Vec<Message>,
    streaming_content: Option<String>,
    response_rx: Option<mpsc::Receiver<StreamMessage>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            streaming_content: None,
            response_rx: None,
            runtime: Arc::new(tokio::runtime::Runtime::new().expect("Failed to create runtime")),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        state: &mut UIState,
        client: LLMClient,
        session: &mut ChatSession,
    ) {
        let available_height = ui.available_height();
        let input_area_height = 100.0;

        ui.vertical(|ui| {
            // 聊天历史记录区域
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .max_height(available_height - input_area_height)
                .show(ui, |ui| {
                    // 显示历史消息
                    for message in &session.messages {
                        self.render_message(ui, message);
                        ui.add_space(8.0);
                    }

                    // 显示正在流式传输的消息
                    if let Some(content) = &self.streaming_content {
                        self.render_message(
                            ui,
                            &Message {
                                role: Role::Assistant,
                                content: MessageContent::Text(content.clone()),
                                timestamp: chrono::Utc::now(),
                            },
                        );
                    }
                });

            ui.separator();

            // 输入区域容器
            egui::Frame::none()
                .fill(ui.style().visuals.window_fill())
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let input_area = ui.available_width() - 60.0;

                        let text_edit = egui::TextEdit::multiline(&mut state.chat_input)
                            .desired_width(input_area)
                            .desired_rows(3)
                            .hint_text("Type a message...")
                            .margin(egui::vec2(8.0, 8.0))
                            .frame(true);

                        let response = ui.add(text_edit);

                        ui.vertical(|ui| {
                            let send_button = ui.add_enabled(
                                !state.chat_input.is_empty() && !state.chat_state.is_sending,
                                egui::Button::new(if state.chat_state.is_sending {
                                    "⏳"
                                } else {
                                    "Send"
                                }),
                            );

                            let mut should_send = false;
                            if response.lost_focus() {
                                if ui.input(|i| {
                                    i.modifiers.command && i.key_pressed(egui::Key::Enter)
                                }) {
                                    state.chat_input.push('\n');
                                    response.request_focus();
                                } else if ui.input(|i| {
                                    !i.modifiers.command && i.key_pressed(egui::Key::Enter)
                                }) {
                                    should_send = !state.chat_input.is_empty();
                                }
                            }

                            should_send |= send_button.clicked();

                            if should_send {
                                info!("Preparing to send message");
                                state.chat_state.is_sending = true;
                                let message = Message {
                                    role: Role::User,
                                    content: MessageContent::Text(state.chat_input.clone()),
                                    timestamp: chrono::Utc::now(),
                                };

                                debug!(?message, "Created user message");
                                state.chat_input.clear();
                                session.add_message(message.clone());
                                info!("Added message to session");

                                let (tx, rx) = mpsc::channel(10);
                                self.response_rx = Some(rx);
                                self.streaming_content = Some(String::new());
                                debug!("Set up streaming channel");

                                let client_clone = client.clone();
                                let ctx = ui.ctx().clone();
                                self.runtime.spawn(async move {
                                    client_clone.set_response_tx(tx.clone()).await;

                                    info!("Starting async message processing");
                                    match client_clone.send_message_streaming(message).await {
                                        Ok(mut stream_rx) => {
                                            info!("Successfully created message stream");
                                            while let Some(message) = stream_rx.recv().await {
                                                debug!(?message, "Received stream message");
                                                if let Err(e) = tx.send(message).await {
                                                    error!(
                                                        ?e,
                                                        "Failed to send message through channel"
                                                    );
                                                    break;
                                                }
                                                ctx.request_repaint();
                                            }
                                            info!("Stream processing completed");
                                        }
                                        Err(e) => {
                                            error!(?e, "Failed to create message stream");
                                            let _ =
                                                tx.send(StreamMessage::Error(e.to_string())).await;
                                        }
                                    }
                                });
                            }
                        });
                    });

                    if let Some(error) = &state.chat_state.error {
                        error!(?error, "Displaying error message");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                        });
                    }
                });
        });

        // 处理流式响应
        if let Some(rx) = &mut self.response_rx {
            while let Ok(message) = rx.try_recv() {
                match message {
                    StreamMessage::Chunk(chunk) => {
                        if let Some(content) = &mut self.streaming_content {
                            content.push_str(&chunk);
                        }
                    }
                    StreamMessage::Done(message) => {
                        session.add_message(message);
                        self.streaming_content = None;
                        state.chat_state.is_sending = false;
                    }
                    StreamMessage::Error(error) => {
                        error!(?error, "Stream error");
                        self.streaming_content = None;
                        state.chat_state.is_sending = false;
                        state.chat_state.error = Some(error);
                    }
                }
            }
        }
    }

    fn render_message(&self, ui: &mut Ui, message: &Message) {
        ui.horizontal(|ui| {
            match message.role {
                Role::User => {
                    ui.label("You: ");
                }
                Role::Assistant => {
                    ui.label("AI: ");
                }
                Role::System => {
                    ui.label("System: ");
                }
            }

            match &message.content {
                MessageContent::Text(text) => {
                    ui.label(text);
                }
                MessageContent::Image { text, .. } => {
                    debug!("Rendering image message");
                    ui.label(text);
                    // TODO: 显示图片
                }
                MessageContent::Function { name, arguments } => {
                    debug!(name, "Rendering function call message");
                    ui.label(format!("Function call: {} with args: {}", name, arguments));
                }
            }
        });
    }
}
