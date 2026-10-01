//! OpenAI-compatible data models.
//!
//! These types follow the OpenAI API specification exactly to ensure
//! compatibility with frameworks like LangChain, CrewAI, and n8n.

use serde::{Deserialize, Serialize};

// ============================================================================
// Chat Completions
// ============================================================================

/// Chat completion request (OpenAI format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier
    pub model: String,

    /// Array of messages in the conversation
    pub messages: Vec<ChatMessage>,

    /// Sampling temperature (0.0 - 2.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Enable streaming responses
    #[serde(default)]
    pub stream: bool,

    /// Nucleus sampling parameter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Frequency penalty (-2.0 - 2.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty (-2.0 - 2.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Stop sequences
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// User identifier for abuse detection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Number of completions to generate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Log probabilities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Seed for deterministic sampling
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,

    /// Message content
    pub content: String,

    /// Optional name for the participant
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Function call result
    Function,
    /// Tool call result
    Tool,
}

/// Chat completion response (OpenAI format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique identifier
    pub id: String,

    /// Object type (always "chat.completion")
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Model used
    pub model: String,

    /// Array of completion choices
    pub choices: Vec<ChatChoice>,

    /// Token usage statistics
    pub usage: Usage,

    /// System fingerprint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Chat choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,

    /// Generated message
    pub message: ChatMessage,

    /// Reason for finishing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u64,

    /// Tokens in the completion
    pub completion_tokens: u64,

    /// Total tokens used
    pub total_tokens: u64,
}

// ============================================================================
// Streaming
// ============================================================================

/// Streaming chat completion chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier
    pub id: String,

    /// Object type (always "chat.completion.chunk")
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Model used
    pub model: String,

    /// Array of delta choices
    pub choices: Vec<ChatChunkChoice>,

    /// System fingerprint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Streaming choice delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    /// Choice index
    pub index: u32,

    /// Delta content
    pub delta: ChatDelta,

    /// Reason for finishing (only in last chunk)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta content in streaming
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatDelta {
    /// Role (only in first chunk)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// Content delta
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

// ============================================================================
// Embeddings
// ============================================================================

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model identifier
    pub model: String,

    /// Input text (string or array)
    pub input: EmbeddingInput,

    /// Encoding format
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// User identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Dimensions for the output
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// Embedding input (string or array)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single string input
    Single(String),
    /// Multiple string inputs
    Multiple(Vec<String>),
}

impl EmbeddingInput {
    /// Get the number of inputs
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multiple(v) => v.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.is_empty(),
            Self::Multiple(v) => v.is_empty(),
        }
    }

    /// Convert to vector of strings
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s.clone()],
            Self::Multiple(v) => v.clone(),
        }
    }
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type (always "list")
    pub object: String,

    /// Array of embeddings
    pub data: Vec<EmbeddingData>,

    /// Model used
    pub model: String,

    /// Usage statistics
    pub usage: EmbeddingUsage,
}

/// Embedding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// Object type (always "embedding")
    pub object: String,

    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Index of the input
    pub index: u32,
}

/// Embedding usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u64,

    /// Total tokens used
    pub total_tokens: u64,
}

// ============================================================================
// Models
// ============================================================================

/// Models list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    /// Object type (always "list")
    pub object: String,

    /// Array of model objects
    pub data: Vec<ModelObject>,
}

/// Model object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelObject {
    /// Model identifier
    pub id: String,

    /// Object type (always "model")
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Owner/organization
    pub owned_by: String,

    /// Pool-specific information (extension)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool_info: Option<PoolModelInfo>,
}

/// Pool-specific model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolModelInfo {
    /// Number of nodes supporting this model
    pub node_count: u32,

    /// Total capacity available
    pub total_capacity: u32,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Average success rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_success_rate: Option<f64>,
}

// ============================================================================
// Validation
// ============================================================================

impl ChatCompletionRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.model.is_empty() {
            errors.push("model is required".to_string());
        }

        if self.messages.is_empty() {
            errors.push("messages array cannot be empty".to_string());
        }

        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                errors.push("temperature must be between 0 and 2".to_string());
            }
        }

        if let Some(top_p) = self.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                errors.push("top_p must be between 0 and 1".to_string());
            }
        }

        if let Some(freq) = self.frequency_penalty {
            if !(-2.0..=2.0).contains(&freq) {
                errors.push("frequency_penalty must be between -2 and 2".to_string());
            }
        }

        if let Some(pres) = self.presence_penalty {
            if !(-2.0..=2.0).contains(&pres) {
                errors.push("presence_penalty must be between -2 and 2".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl EmbeddingRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.model.is_empty() {
            errors.push("model is required".to_string());
        }

        if self.input.is_empty() {
            errors.push("input cannot be empty".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ============================================================================
// Builders
// ============================================================================

impl ChatCompletionResponse {
    /// Create a new chat completion response
    pub fn new(model: String, message: ChatMessage, usage: Usage) -> Self {
        Self {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model,
            choices: vec![ChatChoice {
                index: 0,
                message,
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage,
            system_fingerprint: None,
        }
    }
}

impl EmbeddingResponse {
    /// Create a new embedding response
    pub fn new(model: String, embeddings: Vec<Vec<f32>>, usage: EmbeddingUsage) -> Self {
        Self {
            object: "list".to_string(),
            data: embeddings
                .into_iter()
                .enumerate()
                .map(|(i, embedding)| EmbeddingData {
                    object: "embedding".to_string(),
                    embedding,
                    index: i as u32,
                })
                .collect(),
            model,
            usage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: "Hello!".to_string(),
                    name: None,
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
            n: None,
            logprobs: None,
            seed: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));

        let parsed: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, "gpt-4");
        assert_eq!(parsed.messages.len(), 2);
    }

    #[test]
    fn test_message_role_serialization() {
        assert_eq!(
            serde_json::to_string(&MessageRole::System).unwrap(),
            "\"system\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Assistant).unwrap(),
            "\"assistant\""
        );
    }

    #[test]
    fn test_embedding_input_variants() {
        // Single string
        let single = EmbeddingInput::Single("Hello".to_string());
        let json = serde_json::to_string(&single).unwrap();
        assert_eq!(json, "\"Hello\"");

        // Multiple strings
        let multiple = EmbeddingInput::Multiple(vec!["Hello".to_string(), "World".to_string()]);
        let json = serde_json::to_string(&multiple).unwrap();
        assert_eq!(json, "[\"Hello\",\"World\"]");

        // Deserialization
        let parsed: EmbeddingInput = serde_json::from_str("\"test\"").unwrap();
        assert_eq!(parsed, EmbeddingInput::Single("test".to_string()));

        let parsed: EmbeddingInput = serde_json::from_str("[\"a\",\"b\"]").unwrap();
        assert_eq!(
            parsed,
            EmbeddingInput::Multiple(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_chat_request_validation() {
        // Valid request
        let valid = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            max_tokens: None,
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
            n: None,
            logprobs: None,
            seed: None,
        };
        assert!(valid.validate().is_ok());

        // Invalid: empty model
        let invalid = ChatCompletionRequest {
            model: "".to_string(),
            messages: vec![],
            temperature: Some(3.0), // Invalid
            ..valid.clone()
        };
        let errors = invalid.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("model")));
        assert!(errors.iter().any(|e| e.contains("messages")));
        assert!(errors.iter().any(|e| e.contains("temperature")));
    }

    #[test]
    fn test_chat_response_builder() {
        let response = ChatCompletionResponse::new(
            "gpt-4".to_string(),
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hello! How can I help?".to_string(),
                name: None,
            },
            Usage {
                prompt_tokens: 10,
                completion_tokens: 8,
                total_tokens: 18,
            },
        );

        assert!(response.id.starts_with("chatcmpl-"));
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.usage.total_tokens, 18);
    }
}







