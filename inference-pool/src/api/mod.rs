//! API module.
//!
//! HTTP server and OpenAI-compatible API endpoints.
//!
//! ## Endpoints
//!
//! - `POST /v1/chat/completions` - Chat completions (supports streaming)
//! - `POST /v1/embeddings` - Text embeddings
//! - `GET /v1/models` - List available models
//! - `GET /health` - Health check
//! - `GET /v1/pool/status` - Pool status
//! - `POST /v1/pool/register` - Node registration

mod handlers;
mod server;
mod streaming;
pub mod admin;

pub use handlers::*;
pub use server::{create_router, ApiServer, ApiState};
pub use streaming::{
    ChatCompletionChunk, ChunkChoice, ChunkDelta, StreamEvent, StreamSender,
    create_configured_stream, create_sse_response,
};
pub use admin::{AdminState, admin_routes};
