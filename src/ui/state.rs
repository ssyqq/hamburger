use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        }
    }
}

#[derive(Default)]
pub struct ChatState {
    pub is_sending: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct UIState {
    pub show_settings: bool,
    pub current_chat_id: Option<String>,
    pub chat_input: String,
    pub settings: SettingsState,
    pub chat_state: ChatState,
    pub new_chat_requested: bool,
    pub delete_chat_requested: Option<String>,
}
