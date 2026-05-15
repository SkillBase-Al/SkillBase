use serde::{Deserialize, Serialize};

/// Represents a completion response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCompletion {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// A message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// Supported LLM provider types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
}

impl LlmProviderType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anthropic" => LlmProviderType::Anthropic,
            _ => LlmProviderType::OpenAI,
        }
    }
}

/// A client for making LLM API calls.
#[derive(Debug, Clone)]
pub struct LlmClient {
    provider: LlmProviderType,
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(
        provider: LlmProviderType,
        api_key: String,
        model: String,
        base_url: String,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create reqwest client");

        Self {
            provider,
            api_key,
            model,
            base_url,
            client,
        }
    }

    pub fn from_env() -> Self {
        let provider = LlmProviderType::from_str(
            &std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
        );
        let api_key = std::env::var("LLM_API_KEY")
            .expect("LLM_API_KEY environment variable is required");
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        Self::new(provider, api_key, model, base_url)
    }

    pub fn provider_type(&self) -> LlmProviderType {
        self.provider
    }

    /// Send a completion request to the configured LLM provider.
    pub async fn complete(
        &self,
        system_prompt: &str,
        messages: Vec<LlmMessage>,
        max_tokens: u32,
    ) -> Result<LlmCompletion, Box<dyn std::error::Error + Send + Sync>> {
        match self.provider {
            LlmProviderType::OpenAI => {
                self.complete_openai(system_prompt, messages, max_tokens).await
            }
            LlmProviderType::Anthropic => {
                self.complete_anthropic(system_prompt, messages, max_tokens).await
            }
        }
    }

    async fn complete_openai(
        &self,
        system_prompt: &str,
        messages: Vec<LlmMessage>,
        max_tokens: u32,
    ) -> Result<LlmCompletion, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        #[derive(Serialize, Deserialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            max_tokens: u32,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
        }

        #[derive(Deserialize)]
        struct OpenAIUsage {
            prompt_tokens: u32,
            completion_tokens: u32,
            total_tokens: u32,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
            usage: Option<OpenAIUsage>,
        }

        let mut api_messages = vec![OpenAIMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        }];

        for msg in messages {
            api_messages.push(OpenAIMessage {
                role: msg.role,
                content: msg.content,
            });
        }

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: api_messages,
            max_tokens,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error ({}): {}", status, body).into());
        }

        let data: OpenAIResponse = response.json().await?;
        let choice = data.choices.into_iter().next().ok_or("No completion choices returned")?;
        let usage = data.usage.unwrap_or(OpenAIUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        Ok(LlmCompletion {
            content: choice.message.content,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        })
    }

    async fn complete_anthropic(
        &self,
        system_prompt: &str,
        messages: Vec<LlmMessage>,
        max_tokens: u32,
    ) -> Result<LlmCompletion, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));

        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            system: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: u32,
        }

        #[derive(Deserialize)]
        struct AnthropicContent {
            text: Option<String>,
        }

        #[derive(Deserialize)]
        struct AnthropicUsage {
            input_tokens: u32,
            output_tokens: u32,
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
            usage: Option<AnthropicUsage>,
        }

        let api_messages: Vec<AnthropicMessage> = messages
            .into_iter()
            .map(|msg| AnthropicMessage {
                role: msg.role,
                content: msg.content,
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            system: system_prompt.to_string(),
            messages: api_messages,
            max_tokens,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Anthropic API error ({}): {}", status, body).into());
        }

        let data: AnthropicResponse = response.json().await?;
        let content = data
            .content
            .into_iter()
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n");

        let usage = data.usage.unwrap_or(AnthropicUsage {
            input_tokens: 0,
            output_tokens: 0,
        });

        Ok(LlmCompletion {
            content,
            prompt_tokens: usage.input_tokens,
            completion_tokens: usage.output_tokens,
            total_tokens: usage.input_tokens + usage.output_tokens,
        })
    }
}
