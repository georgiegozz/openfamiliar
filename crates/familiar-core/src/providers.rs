//! Provider router and adapters (Ollama, OpenAI-compatible, Gemini, mock).

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("unknown provider: {0}")]
    Unknown(String),
    #[error("config invalid: {0}")]
    Config(String),
    #[error("http: {0}")]
    Http(String),
    #[error("cancelled")]
    Cancelled,
    #[error("timeout")]
    Timeout,
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessageDto>,
    pub max_tokens: Option<u32>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub ok: bool,
    pub message: String,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn validate_configuration(&self) -> Result<ValidationResult>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn stream_collect(&self, request: ChatRequest) -> Result<String>;
    async fn cancel(&self, _session_id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct MockProvider;

#[async_trait]
impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        "mock"
    }

    async fn validate_configuration(&self) -> Result<ValidationResult> {
        Ok(ValidationResult {
            ok: true,
            message: "mock ready".into(),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: "mock-model".into(),
            name: "Mock Model".into(),
        }])
    }

    async fn stream_collect(&self, request: ChatRequest) -> Result<String> {
        let last = request
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        let text = last.to_lowercase();
        let reply = if text.contains("benito") || text.contains("juarez") || text.contains("juárez") {
            "Benito Juárez nació el 21 de marzo de 1806 en Guelatao, Oaxaca. Fue un célebre presidente de México conocido como el «Benemérito de las Américas». ¡Una gran figura de la historia! 🐾".to_string()
        } else if text.contains("hola") || text.contains("hello") || text.contains("buenos dias") {
            "¡Hola! Soy Perrito Tech. Estoy aquí flotando para ayudarte en lo que necesites. ¿En qué programamos hoy? 🐶".to_string()
        } else if text.contains("cómo estás") || text.contains("como estas") || text.contains("tal") {
            "¡De maravilla! Listo y atento a tus comandos. ¿Y tú, qué tal va el día? 🐾".to_string()
        } else {
            format!("¡Guau! Recibí tu mensaje: «{}». (Nota: Estoy en modo Demo Offline. Configura tu API key de Gemini o OpenAI en el panel inferior para chatear con IA real). 🐶", last)
        };
        
        Ok(reply)
    }
}

pub struct OllamaProvider {
    pub base_url: String,
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama-local"
    }

    async fn validate_configuration(&self) -> Result<ValidationResult> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        match reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => Ok(ValidationResult {
                ok: true,
                message: "ollama reachable".into(),
            }),
            Ok(resp) => Ok(ValidationResult {
                ok: false,
                message: format!("status {}", resp.status()),
            }),
            Err(e) => Ok(ValidationResult {
                ok: false,
                message: e.to_string(),
            }),
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let resp = reqwest::Client::new()
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let mut models = Vec::new();
        if let Some(arr) = body.get("models").and_then(|m| m.as_array()) {
            for m in arr {
                if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                    models.push(ModelInfo {
                        id: name.to_string(),
                        name: name.to_string(),
                    });
                }
            }
        }
        Ok(models)
    }

    async fn stream_collect(&self, request: ChatRequest) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let payload = serde_json::json!({
            "model": request.model,
            "stream": true,
            "messages": request.messages,
            "options": { "num_predict": request.max_tokens.unwrap_or(512) }
        });
        let resp = reqwest::Client::new()
            .post(url)
            .json(&payload)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Http(format!("status {}", resp.status())));
        }
        let mut stream = resp.bytes_stream();
        let mut out = String::new();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| ProviderError::Http(e.to_string()))?;
            buf.push_str(&String::from_utf8_lossy(&bytes));
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();
                if line.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(content) = v.pointer("/message/content").and_then(|c| c.as_str()) {
                        out.push_str(content);
                    }
                }
            }
        }
        if out.is_empty() {
            Err(ProviderError::Message("empty ollama response".into()))
        } else {
            Ok(out)
        }
    }
}

pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub api_key: String,
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &str {
        "openai-compatible"
    }

    async fn validate_configuration(&self) -> Result<ValidationResult> {
        if self.api_key.trim().is_empty() {
            return Ok(ValidationResult {
                ok: false,
                message: "api key missing".into(),
            });
        }
        Ok(ValidationResult {
            ok: true,
            message: "api key present (connection not probed)".into(),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let resp = reqwest::Client::new()
            .get(url)
            .bearer_auth(&self.api_key)
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let mut models = Vec::new();
        if let Some(arr) = body.get("data").and_then(|d| d.as_array()) {
            for m in arr {
                if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                    models.push(ModelInfo {
                        id: id.to_string(),
                        name: id.to_string(),
                    });
                }
            }
        }
        Ok(models)
    }

    async fn stream_collect(&self, request: ChatRequest) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let payload = serde_json::json!({
            "model": request.model,
            "stream": false,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(512)
        });
        let resp = reqwest::Client::new()
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Http(format!("{status}: {body}")));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let content = body
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        if content.is_empty() {
            Err(ProviderError::Message("empty completion".into()))
        } else {
            Ok(content)
        }
    }
}

pub struct GeminiProvider {
    pub api_key: String,
    pub base_url: String,
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini-native"
    }

    async fn validate_configuration(&self) -> Result<ValidationResult> {
        if self.api_key.trim().is_empty() {
            return Ok(ValidationResult {
                ok: false,
                message: "gemini api key missing".into(),
            });
        }
        Ok(ValidationResult {
            ok: true,
            message: "api key present".into(),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "gemini-2.0-flash".into(),
                name: "Gemini 2.0 Flash".into(),
            },
            ModelInfo {
                id: "gemini-1.5-flash".into(),
                name: "Gemini 1.5 Flash".into(),
            },
        ])
    }

    async fn stream_collect(&self, request: ChatRequest) -> Result<String> {
        let model = request.model;
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            model,
            self.api_key
        );
        let mut contents = Vec::new();
        for m in request.messages {
            let role = if m.role == "assistant" {
                "model"
            } else {
                "user"
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }]
            }));
        }
        let payload = serde_json::json!({ "contents": contents });
        let resp = reqwest::Client::new()
            .post(url)
            .json(&payload)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Http(format!("{status}: {body}")));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;
        let content = body
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if content.is_empty() {
            Err(ProviderError::Message("empty gemini response".into()))
        } else {
            Ok(content)
        }
    }
}

pub struct ProviderRouter {
    providers: HashMap<String, std::sync::Arc<dyn ModelProvider>>,
}

impl ProviderRouter {
    pub fn with_defaults() -> Self {
        let mut providers: HashMap<String, std::sync::Arc<dyn ModelProvider>> = HashMap::new();
        let mock: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::new(MockProvider);
        let ollama: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::new(OllamaProvider {
            base_url: std::env::var("OPENFAMILIAR_OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
        });
        let openai: std::sync::Arc<dyn ModelProvider> =
            std::sync::Arc::new(OpenAiCompatibleProvider {
                base_url: std::env::var("OPENFAMILIAR_OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                api_key: std::env::var("OPENFAMILIAR_OPENAI_API_KEY").unwrap_or_default(),
            });
        let gemini: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::new(GeminiProvider {
            api_key: std::env::var("OPENFAMILIAR_GEMINI_API_KEY").unwrap_or_default(),
            base_url: std::env::var("OPENFAMILIAR_GEMINI_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".into()),
        });
        providers.insert(mock.id().into(), mock);
        providers.insert(ollama.id().into(), ollama);
        providers.insert(openai.id().into(), openai);
        providers.insert(gemini.id().into(), gemini);
        Self { providers }
    }

    pub fn ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn stream_collect(&self, provider_id: &str, request: ChatRequest) -> Result<String> {
        let p = self
            .providers
            .get(provider_id)
            .ok_or_else(|| ProviderError::Unknown(provider_id.into()))?;
        p.stream_collect(request).await
    }

    pub async fn validate(&self, provider_id: &str) -> Result<ValidationResult> {
        let p = self
            .providers
            .get(provider_id)
            .ok_or_else(|| ProviderError::Unknown(provider_id.into()))?;
        p.validate_configuration().await
    }

    pub async fn list_models(&self, provider_id: &str) -> Result<Vec<ModelInfo>> {
        let p = self
            .providers
            .get(provider_id)
            .ok_or_else(|| ProviderError::Unknown(provider_id.into()))?;
        p.list_models().await
    }
}
