use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client as OpenAIClient,
};
use chrono::Utc;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use super::{
    config::LLMConfig,
    message::{Message, MessageContent, Role, StreamMessage},
};

#[derive(Clone)]
pub struct LLMClient {
    client: OpenAIClient<OpenAIConfig>,
    config: Arc<RwLock<LLMConfig>>,
    response_tx: Arc<RwLock<Option<tokio::sync::mpsc::Sender<StreamMessage>>>>,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = OpenAIClient::with_config(
            OpenAIConfig::new()
                .with_api_key(&config.api_key)
                .with_api_base(&config.api_base),
        );

        Self {
            client,
            config: Arc::new(RwLock::new(config)),
            response_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_response_tx(&self, tx: tokio::sync::mpsc::Sender<StreamMessage>) {
        let mut response_tx = self.response_tx.write().await;
        *response_tx = Some(tx);
    }

    pub async fn send_message(&self, messages: Vec<Message>) -> Result<String> {
        let config = self.config.read().await;

        let request = match &messages[0].content {
            MessageContent::Text(text) => CreateChatCompletionRequestArgs::default()
                .model(&config.model)
                .temperature(config.temperature)
                .max_tokens(config.max_tokens)
                .messages([ChatCompletionRequestUserMessageArgs::default()
                    .content(text.clone())
                    .build()?
                    .into()])
                .build()?,
            MessageContent::Image { text, url } => {
                // 实现图片消息处理
                todo!()
            }
            MessageContent::Function { name, arguments } => {
                // 实现函数调用
                todo!()
            }
        };

        let mut response_text = String::new();

        let mut stream = self.client.chat().create_stream(request).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(text) = response
                        .choices
                        .get(0)
                        .and_then(|choice| choice.delta.content.as_ref())
                    {
                        response_text.push_str(text);
                        if let Some(tx) = &*self.response_tx.read().await {
                            tx.send(StreamMessage::Chunk(text.clone())).await.ok();
                        }
                    }
                }
                Err(e) => {
                    error!(?e, "Stream error");
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        Ok(response_text)
    }

    pub async fn send_message_streaming(
        &self,
        message: Message,
    ) -> Result<mpsc::Receiver<StreamMessage>> {
        info!("Starting streaming request");
        let config = self.config.read().await;
        debug!(?config, "Using configuration");

        let request = match &message.content {
            MessageContent::Text(text) => {
                debug!(text_length = text.len(), "Creating text request");
                CreateChatCompletionRequestArgs::default()
                    .model(&config.model)
                    .temperature(config.temperature)
                    .max_tokens(config.max_tokens)
                    .messages([ChatCompletionRequestUserMessageArgs::default()
                        .content(text.clone())
                        .build()?
                        .into()])
                    .build()?
            }
            MessageContent::Image { text, url } => {
                // 实现图片消息处理
                todo!()
            }
            MessageContent::Function { name, arguments } => {
                // 实现函数调用
                todo!()
            }
        };

        debug!("Creating stream");
        let mut stream = self.client.chat().create_stream(request).await?;
        info!("Stream created successfully");

        let (tx, rx) = mpsc::channel(100);
        debug!("Channel created");

        tokio::spawn(async move {
            info!("Starting stream processing");
            let mut content = String::new();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for chat_choice in response.choices {
                            if let Some(delta_content) = chat_choice.delta.content {
                                if !delta_content.is_empty() {
                                    content.push_str(&delta_content);
                                    if let Err(e) =
                                        tx.send(StreamMessage::Chunk(delta_content)).await
                                    {
                                        error!(?e, "Failed to send content");
                                        return;
                                    }
                                }
                            }

                            if let Some(reason) = chat_choice.finish_reason {
                                info!(reason = ?reason, "Stream finished with reason");
                                let final_message = Message {
                                    role: Role::Assistant,
                                    content: MessageContent::Text(content),
                                    timestamp: Utc::now(),
                                };
                                let _ = tx.send(StreamMessage::Done(final_message)).await;
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!(?e, "Stream error");
                        let _ = tx.send(StreamMessage::Error(e.to_string())).await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }
}
