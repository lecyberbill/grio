//! # Native LLM Connectors & Streaming Engine (`grio::ai`)
//!
//! Out-of-the-box connectors for modern local and cloud LLM providers:
//! - **LM Studio** (`http://localhost:1234/v1`)
//! - **Ollama** (`http://localhost:11434/v1`)
//! - **OpenAI / vLLM / Groq / Mistral**
//!
//! Provides zero-boilerplate streaming directly into `Chatbot` or `Output` components.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Supported LLM provider presets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    /// Local LM Studio instance (default `http://localhost:1234/v1`).
    LmStudio,
    /// Local Ollama instance (default `http://localhost:11434/v1`).
    Ollama,
    /// OpenAI official API or custom OpenAI-compatible endpoint (vLLM, TGI, Groq).
    OpenAi,
    /// Custom HTTP endpoint.
    Custom(String),
}

/// LLM Client configuration and executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Llm {
    /// Base URL endpoint.
    pub endpoint: String,
    /// Target model identifier.
    pub model: String,
    /// Optional API authentication key.
    pub api_key: Option<String>,
    /// Sampling temperature (0.0 - 2.0).
    pub temperature: f64,
    /// Top-p nucleus sampling (0.0 - 1.0).
    pub top_p: f64,
    /// Maximum tokens to generate.
    pub max_tokens: Option<usize>,
}

impl Default for Llm {
    fn default() -> Self {
        Self::lm_studio()
    }
}

impl Llm {
    /// Creates a preconfigured connector for local LM Studio.
    pub fn lm_studio() -> Self {
        Self {
            endpoint: "http://localhost:1234/v1/chat/completions".to_string(),
            model: "default".to_string(),
            api_key: None,
            temperature: 0.7,
            top_p: 0.95,
            max_tokens: Some(2048),
        }
    }

    /// Creates a preconfigured connector for local Ollama.
    pub fn ollama() -> Self {
        Self {
            endpoint: "http://localhost:11434/v1/chat/completions".to_string(),
            model: "llama3".to_string(),
            api_key: None,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: Some(2048),
        }
    }

    /// Creates a connector for OpenAI or OpenAI-compatible servers (vLLM, Groq).
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: Some(api_key.into()),
            temperature: 0.7,
            top_p: 1.0,
            max_tokens: Some(2048),
        }
    }

    /// Sets the target model name.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets custom base endpoint URL.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Sets sampling temperature.
    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    /// Sets nucleus sampling top-p.
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = top_p;
        self
    }

    /// Sets maximum tokens.
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Builds the standard OpenAI-compatible JSON payload.
    pub fn build_payload(
        &self,
        messages: &[crate::components::ChatMessage],
        stream: bool,
    ) -> serde_json::Value {
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        json!({
            "model": self.model,
            "messages": msgs,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "stream": stream,
            "max_tokens": self.max_tokens,
        })
    }
}
