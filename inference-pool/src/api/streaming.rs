//! Server-Sent Events (SSE) streaming support.
//!
//! Implements OpenAI-compatible streaming responses for chat completions.
//!
//! ## Protocol
//!
//! - Content-Type: text/event-stream
//! - Each chunk is a JSON object prefixed with "data: "
//! - Stream ends with "data: [DONE]"

use crate::models::{MessageRole, Usage};
use axum::{body::Body, response::Response};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Streaming chat completion chunk (OpenAI format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique chunk ID
    pub id: String,

    /// Object type (always "chat.completion.chunk")
    pub object: String,

    /// Unix timestamp
    pub created: i64,

    /// Model used
    pub model: String,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,

    /// Choices (usually just one)
    pub choices: Vec<ChunkChoice>,

    /// Usage statistics (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Choice in a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// Choice index
    pub index: u32,

    /// Delta content
    pub delta: ChunkDelta,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,

    /// Finish reason (null until complete)
    pub finish_reason: Option<String>,
}

/// Delta content in a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkDelta {
    /// Role (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Tool calls (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
}

impl ChatCompletionChunk {
    /// Create a new chunk with role only (first chunk)
    pub fn first(id: &str, model: &str) -> Self {
        Self {
            id: id.to_string(),
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            system_fingerprint: None,
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: Some(MessageRole::Assistant),
                    content: None,
                    tool_calls: None,
                },
                logprobs: None,
                finish_reason: None,
            }],
            usage: None,
        }
    }

    /// Create a content chunk
    pub fn content(id: &str, model: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            system_fingerprint: None,
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: None,
                    content: Some(content.to_string()),
                    tool_calls: None,
                },
                logprobs: None,
                finish_reason: None,
            }],
            usage: None,
        }
    }

    /// Create a final chunk with finish reason
    pub fn finish(id: &str, model: &str, reason: &str, usage: Option<Usage>) -> Self {
        Self {
            id: id.to_string(),
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            system_fingerprint: None,
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta::default(),
                logprobs: None,
                finish_reason: Some(reason.to_string()),
            }],
            usage,
        }
    }

    /// Serialize to SSE data format
    pub fn to_sse_data(&self) -> String {
        format!("data: {}\n\n", serde_json::to_string(self).unwrap_or_default())
    }
}

/// SSE event for streaming
#[derive(Debug)]
pub enum StreamEvent {
    /// Chat completion chunk
    Chunk(ChatCompletionChunk),

    /// Stream done marker
    Done,

    /// Error occurred
    Error(String),
}

impl StreamEvent {
    /// Convert to SSE data string
    pub fn to_sse_string(&self) -> String {
        match self {
            StreamEvent::Chunk(chunk) => chunk.to_sse_data(),
            StreamEvent::Done => "data: [DONE]\n\n".to_string(),
            StreamEvent::Error(msg) => {
                let error_json = serde_json::json!({
                    "error": {
                        "message": msg,
                        "type": "server_error",
                        "code": "stream_error"
                    }
                });
                format!("data: {}\n\n", error_json)
            }
        }
    }
}

/// Create a streaming response from a channel receiver
pub fn create_sse_response(rx: mpsc::Receiver<StreamEvent>) -> Response {
    let stream = ReceiverStream::new(rx);
    let body_stream = stream.map(|event| Ok::<_, Infallible>(event.to_sse_string()));

    Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .header("x-accel-buffering", "no")
        .body(Body::from_stream(body_stream))
        .unwrap()
}

/// Stream sender for sending chunks
#[derive(Clone)]
pub struct StreamSender {
    tx: mpsc::Sender<StreamEvent>,
    id: String,
    model: String,
}

impl StreamSender {
    /// Create a new stream sender
    pub fn new(tx: mpsc::Sender<StreamEvent>, id: &str, model: &str) -> Self {
        Self {
            tx,
            id: id.to_string(),
            model: model.to_string(),
        }
    }

    /// Send the first chunk (with role)
    pub async fn send_first(&self) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        let chunk = ChatCompletionChunk::first(&self.id, &self.model);
        self.tx.send(StreamEvent::Chunk(chunk)).await
    }

    /// Send a content chunk
    pub async fn send_content(&self, content: &str) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        let chunk = ChatCompletionChunk::content(&self.id, &self.model, content);
        self.tx.send(StreamEvent::Chunk(chunk)).await
    }

    /// Send the final chunk
    pub async fn send_finish(&self, usage: Option<Usage>) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        let chunk = ChatCompletionChunk::finish(&self.id, &self.model, "stop", usage);
        self.tx.send(StreamEvent::Chunk(chunk)).await
    }

    /// Send done marker
    pub async fn send_done(&self) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        self.tx.send(StreamEvent::Done).await
    }

    /// Send error
    pub async fn send_error(&self, msg: &str) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        self.tx.send(StreamEvent::Error(msg.to_string())).await
    }
}

/// Create a stream channel
pub fn create_stream_channel(buffer_size: usize) -> (StreamSender, mpsc::Receiver<StreamEvent>) {
    let (tx, rx) = mpsc::channel(buffer_size);
    // Note: id and model will be set when using the sender
    (StreamSender { tx, id: String::new(), model: String::new() }, rx)
}

/// Create a configured stream sender and receiver
pub fn create_configured_stream(id: &str, model: &str, buffer_size: usize) -> (StreamSender, mpsc::Receiver<StreamEvent>) {
    let (tx, rx) = mpsc::channel(buffer_size);
    (StreamSender::new(tx, id, model), rx)
}

use tokio_stream::StreamExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_first() {
        let chunk = ChatCompletionChunk::first("chatcmpl-123", "gpt-4");

        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.role, Some(MessageRole::Assistant));
        assert!(chunk.choices[0].delta.content.is_none());
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_chunk_content() {
        let chunk = ChatCompletionChunk::content("chatcmpl-123", "gpt-4", "Hello");

        assert!(chunk.choices[0].delta.role.is_none());
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_chunk_finish() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        let chunk = ChatCompletionChunk::finish("chatcmpl-123", "gpt-4", "stop", Some(usage));

        assert!(chunk.choices[0].delta.role.is_none());
        assert!(chunk.choices[0].delta.content.is_none());
        assert_eq!(chunk.choices[0].finish_reason, Some("stop".to_string()));
        assert!(chunk.usage.is_some());
    }

    #[test]
    fn test_sse_format() {
        let chunk = ChatCompletionChunk::content("chatcmpl-123", "gpt-4", "Hi");
        let sse = chunk.to_sse_data();

        assert!(sse.starts_with("data: "));
        assert!(sse.ends_with("\n\n"));
        assert!(sse.contains("\"content\":\"Hi\""));
    }

    #[test]
    fn test_done_event() {
        let event = StreamEvent::Done;
        assert_eq!(event.to_sse_string(), "data: [DONE]\n\n");
    }

    #[tokio::test]
    async fn test_stream_sender() {
        let (sender, mut rx) = create_configured_stream("chatcmpl-123", "gpt-4", 10);

        // Spawn sender task
        let sender_task = tokio::spawn(async move {
            // Send first chunk
            sender.send_first().await.unwrap();

            // Send content
            sender.send_content("Hello").await.unwrap();
            sender.send_content(" world").await.unwrap();

            // Send finish
            sender.send_finish(None).await.unwrap();
            sender.send_done().await.unwrap();

            // Drop sender to close channel
            drop(sender);
        });

        // Receive and verify
        let mut chunks = Vec::new();
        while let Some(event) = rx.recv().await {
            chunks.push(event);
        }

        sender_task.await.unwrap();

        assert_eq!(chunks.len(), 5);
        assert!(matches!(chunks[0], StreamEvent::Chunk(_)));
        assert!(matches!(chunks[4], StreamEvent::Done));
    }
}

