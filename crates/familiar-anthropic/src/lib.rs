//! One-shot Anthropic Messages client. The only allowed base URL is
//! `https://api.anthropic.com`. Redirects are refused. TLS is required.
//! Response and error bodies are never logged; user-facing errors do not
//! include the provider body.

use familiar_credentials::SecretStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
pub const ANTHROPIC_MESSAGES_PATH: &str = "/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const DEFAULT_MODEL: &str = "claude-sonnet-4-5";
pub const DEFAULT_MAX_TOKENS: u32 = 1024;

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum AnthropicError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("rate limited")]
    RateLimit,
    #[error("request timed out")]
    Timeout,
    #[error("request cancelled")]
    Cancelled,
    #[error("invalid request")]
    InvalidRequest,
    #[error("provider request failed")]
    ProcessFailed,
    #[error("provider output was empty or invalid")]
    InvalidOutput,
    #[error("provider output exceeded the size limit")]
    OutputTooLarge,
}

#[derive(Debug, Clone)]
pub struct AnthropicRequest {
    pub prompt: String,
    pub model: String,
    pub timeout: Duration,
    pub max_response_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnthropicResponse {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

pub trait Transport: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        timeout: Duration,
    ) -> Result<HttpResponse, AnthropicError>;
}

#[derive(Clone, Default)]
pub struct FakeTransport {
    pub url: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    pub headers: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>>,
    pub body: std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    pub response: HttpResponse,
    pub error: Option<AnthropicError>,
}

impl FakeTransport {
    pub fn with_response(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            response: HttpResponse {
                status,
                body: body.into(),
            },
            ..Self::default()
        }
    }
}

impl Transport for FakeTransport {
    fn post_json(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        _timeout: Duration,
    ) -> Result<HttpResponse, AnthropicError> {
        *self.url.lock().expect("url lock") = Some(url.to_string());
        *self.headers.lock().expect("header lock") = headers;
        *self.body.lock().expect("body lock") = Some(body);
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(self.response.clone())
    }
}

pub fn messages_url() -> String {
    format!("{ANTHROPIC_API_BASE}{ANTHROPIC_MESSAGES_PATH}")
}

pub fn validate_endpoint(url: &str) -> Result<(), AnthropicError> {
    if !url.starts_with("https://api.anthropic.com/") {
        return Err(AnthropicError::InvalidRequest);
    }
    if url.contains('@') || url.contains('?') {
        return Err(AnthropicError::InvalidRequest);
    }
    Ok(())
}

#[derive(Serialize)]
struct MessagesBody<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Option<Vec<ContentBlock>>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

pub fn complete_one_shot<S: SecretStore, T: Transport>(
    secrets: &S,
    transport: &T,
    request: AnthropicRequest,
) -> Result<AnthropicResponse, AnthropicError> {
    if request.prompt.trim().is_empty() {
        return Err(AnthropicError::InvalidRequest);
    }
    let key = secrets
        .get_secret("anthropic")
        .map_err(|_| AnthropicError::ProcessFailed)?
        .ok_or(AnthropicError::NotAuthenticated)?;
    let url = messages_url();
    validate_endpoint(&url)?;
    let model = if request.model.trim().is_empty() {
        DEFAULT_MODEL
    } else {
        request.model.trim()
    };
    let payload = serde_json::to_vec(&MessagesBody {
        model,
        max_tokens: DEFAULT_MAX_TOKENS,
        messages: vec![Message {
            role: "user",
            content: request.prompt.trim(),
        }],
    })
    .map_err(|_| AnthropicError::InvalidRequest)?;
    let mut headers = HashMap::new();
    headers.insert("x-api-key".into(), key);
    headers.insert("anthropic-version".into(), ANTHROPIC_VERSION.into());
    headers.insert("content-type".into(), "application/json".into());
    let response = transport.post_json(&url, headers, payload, request.timeout)?;
    map_response(response, request.max_response_bytes)
}

fn map_response(
    response: HttpResponse,
    max_response_bytes: usize,
) -> Result<AnthropicResponse, AnthropicError> {
    match response.status {
        200 => parse_success(response.body, max_response_bytes),
        401 | 403 => Err(AnthropicError::NotAuthenticated),
        429 => Err(AnthropicError::RateLimit),
        _ => Err(AnthropicError::ProcessFailed),
    }
}

fn parse_success(body: Vec<u8>, max_response_bytes: usize) -> Result<AnthropicResponse, AnthropicError> {
    if body.len() > max_response_bytes {
        return Err(AnthropicError::OutputTooLarge);
    }
    let parsed: MessagesResponse =
        serde_json::from_slice(&body).map_err(|_| AnthropicError::InvalidOutput)?;
    let text = parsed
        .content
        .into_iter()
        .flatten()
        .filter(|block| block.kind.as_deref().unwrap_or("text") == "text")
        .filter_map(|block| block.text)
        .collect::<Vec<_>>()
        .join("");
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(AnthropicError::InvalidOutput);
    }
    if text.len() > max_response_bytes {
        return Err(AnthropicError::OutputTooLarge);
    }
    Ok(AnthropicResponse { text })
}

/// Error messages safe to show or log: no provider body, no key, no prompt.
pub fn public_error_message(error: &AnthropicError) -> &'static str {
    match error {
        AnthropicError::NotAuthenticated => "Anthropic is not authenticated",
        AnthropicError::RateLimit => "Anthropic rate-limited the request",
        AnthropicError::Timeout => "The Anthropic request timed out",
        AnthropicError::Cancelled => "The Anthropic request was cancelled",
        AnthropicError::InvalidRequest => "The Anthropic request was invalid",
        AnthropicError::ProcessFailed => "The Anthropic request failed",
        AnthropicError::InvalidOutput => "Anthropic returned an invalid answer",
        AnthropicError::OutputTooLarge => "The Anthropic answer exceeded the size limit",
    }
}


#[cfg(feature = "http")]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl ReqwestTransport {
    pub fn new(timeout: Duration) -> Result<Self, AnthropicError> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| AnthropicError::ProcessFailed)?;
        Ok(Self { client })
    }

    pub async fn post_json(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        timeout: Duration,
    ) -> Result<HttpResponse, AnthropicError> {
        validate_endpoint(url)?;
        let mut request = self
            .client
            .post(url)
            .timeout(timeout)
            .body(body);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        let response = request.send().await.map_err(|error| {
            if error.is_timeout() {
                AnthropicError::Timeout
            } else {
                AnthropicError::ProcessFailed
            }
        })?;
        let status = response.status().as_u16();
        let body = response
            .bytes()
            .await
            .map_err(|_| AnthropicError::ProcessFailed)?
            .to_vec();
        Ok(HttpResponse { status, body })
    }
}

#[cfg(feature = "http")]
pub async fn complete_one_shot_reqwest<S: SecretStore>(
    secrets: &S,
    request: AnthropicRequest,
) -> Result<AnthropicResponse, AnthropicError> {
    if request.prompt.trim().is_empty() {
        return Err(AnthropicError::InvalidRequest);
    }
    let key = secrets
        .get_secret("anthropic")
        .map_err(|_| AnthropicError::ProcessFailed)?
        .ok_or(AnthropicError::NotAuthenticated)?;
    let url = messages_url();
    validate_endpoint(&url)?;
    let model = if request.model.trim().is_empty() {
        DEFAULT_MODEL
    } else {
        request.model.trim()
    };
    let payload = serde_json::to_vec(&MessagesBody {
        model,
        max_tokens: DEFAULT_MAX_TOKENS,
        messages: vec![Message {
            role: "user",
            content: request.prompt.trim(),
        }],
    })
    .map_err(|_| AnthropicError::InvalidRequest)?;
    let mut headers = HashMap::new();
    headers.insert("x-api-key".into(), key);
    headers.insert("anthropic-version".into(), ANTHROPIC_VERSION.into());
    headers.insert("content-type".into(), "application/json".into());
    let transport = ReqwestTransport::new(request.timeout)?;
    let response = transport
        .post_json(&url, headers, payload, request.timeout)
        .await?;
    map_response(response, request.max_response_bytes)
}


#[cfg(test)]
mod tests {
    use super::*;
    use familiar_credentials::{MemorySecretStore, SecretStore, redact_secrets};

    fn ok_body() -> Vec<u8> {
        serde_json::json!({
            "content": [{"type": "text", "text": "Hola desde un servidor falso."}]
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn one_shot_uses_fixed_https_endpoint_and_returns_text() {
        let secrets = MemorySecretStore::new();
        secrets
            .set_secret("anthropic", "sk-ant-testfixture-not-real")
            .unwrap();
        let transport = FakeTransport::with_response(200, ok_body());
        let response = complete_one_shot(
            &secrets,
            &transport,
            AnthropicRequest {
                prompt: "Hola".to_string(),
                model: DEFAULT_MODEL.to_string(),
                timeout: Duration::from_secs(10),
                max_response_bytes: 65_536,
            },
        )
        .unwrap();
        assert_eq!(response.text, "Hola desde un servidor falso.");
        let url = transport.url.lock().unwrap().clone().unwrap();
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
        assert!(url.starts_with("https://"));
        validate_endpoint(&url).unwrap();
        let headers = transport.headers.lock().unwrap().clone();
        assert_eq!(
            headers.get("x-api-key").map(String::as_str),
            Some("sk-ant-testfixture-not-real")
        );
    }

    #[test]
    fn missing_key_is_not_authenticated_and_does_not_call_transport() {
        let secrets = MemorySecretStore::new();
        let transport = FakeTransport::with_response(200, ok_body());
        let error = complete_one_shot(
            &secrets,
            &transport,
            AnthropicRequest {
                prompt: "Hola".to_string(),
                model: DEFAULT_MODEL.to_string(),
                timeout: Duration::from_secs(10),
                max_response_bytes: 65_536,
            },
        )
        .unwrap_err();
        assert_eq!(error, AnthropicError::NotAuthenticated);
        assert!(transport.url.lock().unwrap().is_none());
        assert_eq!(public_error_message(&error), "Anthropic is not authenticated");
    }

    #[test]
    fn error_body_is_not_surfaced_or_left_in_public_message() {
        let secrets = MemorySecretStore::new();
        secrets
            .set_secret("anthropic", "sk-ant-testfixture-not-real")
            .unwrap();
        let leaked = br#"{"error":{"message":"internal boom with sk-ant-testfixture-not-real"}}"#;
        let transport = FakeTransport::with_response(500, leaked.to_vec());
        let error = complete_one_shot(
            &secrets,
            &transport,
            AnthropicRequest {
                prompt: "Hola".to_string(),
                model: DEFAULT_MODEL.to_string(),
                timeout: Duration::from_secs(10),
                max_response_bytes: 65_536,
            },
        )
        .unwrap_err();
        assert_eq!(error, AnthropicError::ProcessFailed);
        let public = public_error_message(&error);
        assert!(!public.contains("internal boom"));
        assert!(!public.contains("sk-ant"));
        let logged = redact_secrets(public);
        assert_eq!(logged, public);
    }

    #[test]
    fn rejects_non_https_or_foreign_hosts() {
        assert!(validate_endpoint("http://api.anthropic.com/v1/messages").is_err());
        assert!(validate_endpoint("https://evil.example/v1/messages").is_err());
        assert!(validate_endpoint("https://api.anthropic.com.evil/v1/messages").is_err());
    }
}
