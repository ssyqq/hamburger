use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "synmeS7JUMYHDSucuvtJkwJ0djDbvvQB".to_string()),
            api_base: std::env::var("OPENAI_API_BASE")
                .unwrap_or_else(|_| "https://api.mistral.ai/v1".to_string()),
            model: "mistral-large-latest".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        }
    }
}
