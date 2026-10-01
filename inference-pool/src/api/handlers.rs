//! HTTP API Handlers.
//!
//! OpenAI-compatible endpoints and pool management.

use crate::error::PoolError;
use crate::models::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, EmbeddingInput, EmbeddingRequest,
    EmbeddingResponse, EmbeddingUsage, InferenceJob, JobType, MemberStatus, MessageRole,
    ModelObject, ModelsResponse, PoolRegistrationRequest, PoolStatus, RegistrationResponse,
    RegistrationStatus, SlaTier, Usage,
};
use crate::registry::{MemberRegistration, PoolRegistry};
use crate::router::JobRouter;

use super::streaming::{create_configured_stream, create_sse_response, StreamSender};
use super::ApiState;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;

/// Health check response
#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Readiness check response
#[derive(serde::Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub active_members: usize,
    pub models_available: usize,
}

/// API Error response wrapper
pub struct ApiError(PoolError);

impl From<PoolError> for ApiError {
    fn from(error: PoolError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.0.status_code();
        let body = Json(self.0.to_response());
        (status, body).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

// =============================================================================
// Health Endpoints
// =============================================================================

/// Health check endpoint
///
/// GET /health
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// Readiness check endpoint
///
/// GET /ready
pub async fn readiness_check<R, J>(
    State(state): State<ApiState<R, J>>,
) -> Json<ReadinessResponse>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    let active_members = state.registry.count_by_status(MemberStatus::Active);
    let models_available = state.registry.get_available_models().len();

    Json(ReadinessResponse {
        ready: active_members > 0,
        active_members,
        models_available,
    })
}

// =============================================================================
// OpenAI-Compatible Endpoints
// =============================================================================

/// Chat completions endpoint
///
/// POST /v1/chat/completions
///
/// Supports both streaming and non-streaming responses based on the `stream` field.
pub async fn chat_completions<R, J>(
    State(state): State<ApiState<R, J>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ApiError>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    // Validate request
    if let Err(errors) = request.validate() {
        return Err(PoolError::invalid_request(errors.join(", ")).into());
    }

    // Create inference job
    let job = InferenceJob {
        job_id: format!("job-{}", uuid::Uuid::new_v4()),
        client_id: "test-client".to_string(), // TODO: Get from auth
        model: request.model.clone(),
        job_type: JobType::ChatCompletion,
        payload: serde_json::to_value(&request).unwrap_or_default(),
        created_at: Utc::now(),
        timeout_ms: 30_000,
        sla_tier: SlaTier::Basic, // TODO: Get from auth
    };

    // Route the job
    let routing_result = state.router.route_job(&job).await?;

    tracing::info!(
        job_id = %job.job_id,
        model = %request.model,
        member_id = %routing_result.member_id,
        stream = request.stream,
        "Processing chat completion request"
    );

    // Check if streaming is requested
    if request.stream {
        // Return streaming response
        let response_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
        let model = request.model.clone();
        let member_id = routing_result.member_id.clone();

        let (sender, rx) = create_configured_stream(&response_id, &model, 32);

        // Spawn task to send mock streaming response
        tokio::spawn(async move {
            if let Err(e) = send_mock_stream(&sender, &member_id).await {
                tracing::error!(error = %e, "Error sending stream");
            }
        });

        Ok(create_sse_response(rx))
    } else {
        // Return non-streaming response
        let response = ChatCompletionResponse::new(
            request.model.clone(),
            ChatMessage {
                role: MessageRole::Assistant,
                content: format!(
                    "This is a mock response from the Inference Pool. Your request was routed to member: {}",
                    routing_result.member_id
                ),
                name: None,
            },
            Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        );

        Ok(Json(response).into_response())
    }
}

/// Send mock streaming response
async fn send_mock_stream(sender: &StreamSender, member_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::time::{sleep, Duration};

    // Simulate streaming response with delays
    let response_text = format!(
        "This is a mock streaming response from the Inference Pool. Your request was routed to member: {}. \
        Streaming allows you to receive tokens as they are generated, providing a better user experience \
        for long responses.",
        member_id
    );

    // Send first chunk with role
    sender.send_first().await?;
    sleep(Duration::from_millis(50)).await;

    // Split response into words and stream them
    for word in response_text.split_whitespace() {
        sender.send_content(&format!("{} ", word)).await?;
        sleep(Duration::from_millis(30)).await;
    }

    // Send finish chunk with usage
    let usage = Usage {
        prompt_tokens: 10,
        completion_tokens: response_text.split_whitespace().count() as u64,
        total_tokens: 10 + response_text.split_whitespace().count() as u64,
    };
    sender.send_finish(Some(usage)).await?;

    // Send done marker
    sender.send_done().await?;

    Ok(())
}

/// Embeddings endpoint
///
/// POST /v1/embeddings
pub async fn embeddings<R, J>(
    State(state): State<ApiState<R, J>>,
    Json(request): Json<EmbeddingRequest>,
) -> ApiResult<Json<EmbeddingResponse>>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    // Validate request
    if let Err(errors) = request.validate() {
        return Err(PoolError::invalid_request(errors.join(", ")).into());
    }

    // Create inference job
    let job = InferenceJob {
        job_id: format!("job-{}", uuid::Uuid::new_v4()),
        client_id: "test-client".to_string(),
        model: request.model.clone(),
        job_type: JobType::Embedding,
        payload: serde_json::to_value(&request).unwrap_or_default(),
        created_at: Utc::now(),
        timeout_ms: 30_000,
        sla_tier: SlaTier::Basic,
    };

    // Route the job
    let routing_result = state.router.route_job(&job).await?;

    tracing::info!(
        job_id = %job.job_id,
        model = %request.model,
        member_id = %routing_result.member_id,
        "Processing embedding request"
    );

    // Count inputs
    let input_count = match &request.input {
        EmbeddingInput::Single(_) => 1,
        EmbeddingInput::Multiple(v) => v.len(),
    };

    // Mock response - return random embeddings
    let embeddings: Vec<Vec<f32>> = (0..input_count)
        .map(|_| {
            // Return 1536-dimensional embedding (OpenAI ada-002 size)
            (0..1536).map(|_| rand::random::<f32>() * 2.0 - 1.0).collect()
        })
        .collect();

    let response = EmbeddingResponse::new(
        request.model.clone(),
        embeddings,
        EmbeddingUsage {
            prompt_tokens: (input_count * 10) as u64,
            total_tokens: (input_count * 10) as u64,
        },
    );

    Ok(Json(response))
}

/// List models endpoint
///
/// GET /v1/models
pub async fn list_models<R, J>(
    State(state): State<ApiState<R, J>>,
) -> Json<ModelsResponse>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    let available_models = state.registry.get_available_models();

    let models: Vec<ModelObject> = available_models
        .iter()
        .map(|model| ModelObject {
            id: model.model_id.clone(),
            object: "model".to_string(),
            created: Utc::now().timestamp(),
            owned_by: "inference-pool".to_string(),
            pool_info: None,
        })
        .collect();

    Json(ModelsResponse {
        object: "list".to_string(),
        data: models,
    })
}

// =============================================================================
// Pool Management Endpoints
// =============================================================================

/// Pool status endpoint
///
/// GET /v1/pool/status
pub async fn pool_status<R, J>(
    State(state): State<ApiState<R, J>>,
) -> Json<PoolStatus>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    let active_count = state.registry.count_by_status(MemberStatus::Active);
    let total_count = state.registry.total_count();
    let models = state.registry.get_available_models();

    Json(PoolStatus {
        pool_id: state.config.pool_id().to_string(),
        active_members: active_count as u32,
        total_capacity: total_count as u32,
        models_available: models.iter().map(|m| m.model_id.clone()).collect(),
        avg_latency_ms: 0.0, // TODO: Calculate from metrics
        requests_per_minute: 0.0, // TODO: Track
        uptime_percentage: 100.0, // TODO: Track
        last_updated: Utc::now(),
    })
}

/// Node registration endpoint
///
/// POST /v1/pool/register
pub async fn register_node<R, J>(
    State(state): State<ApiState<R, J>>,
    Json(request): Json<PoolRegistrationRequest>,
) -> ApiResult<Json<RegistrationResponse>>
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    // Convert to MemberRegistration
    let registration = MemberRegistration {
        node_id: request.node_id.clone(),
        peer_id: request.peer_id,
        capabilities: request.capabilities,
        models: request.models,
        lightning_pubkey: request.lightning_pubkey,
        multiaddrs: request.multiaddrs,
        registration_key: request.registration_key,
    };

    // Register the member
    match state.registry.register_member(registration).await {
        Ok(member) => {
            tracing::info!(
                member_id = %member.member_id,
                node_id = %member.node_id,
                "Registered new member"
            );

            Ok(Json(RegistrationResponse {
                status: RegistrationStatus::Accepted,
                member_id: Some(member.member_id),
                channel_id: None, // TODO: Open Lightning channel
                pool_info: state.config.to_pool_info(),
                message: Some("Registration successful. Send heartbeat to activate.".to_string()),
            }))
        }
        Err(e) => {
            tracing::warn!(
                node_id = %request.node_id,
                error = %e,
                "Registration failed"
            );

            Ok(Json(RegistrationResponse {
                status: RegistrationStatus::Rejected,
                member_id: None,
                channel_id: None,
                pool_info: state.config.to_pool_info(),
                message: Some(e.to_string()),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
    }

    #[test]
    fn test_readiness_response_serialization() {
        let response = ReadinessResponse {
            ready: true,
            active_members: 5,
            models_available: 3,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ready\":true"));
        assert!(json.contains("\"active_members\":5"));
    }
}

