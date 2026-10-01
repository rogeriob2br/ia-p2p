//! Property tests for the HTTP API.
//!
//! Feature: api-integration
//! Tests Properties 5, 6 from the design document.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inference_pool::{
    api::{create_router, ApiState},
    config::PoolConfig,
    models::{JobType, MemberStatus, ModelInfo, NodeCapabilities, SlaTier},
    registry::{InMemoryPoolRegistry, MemberRegistration},
    router::{JobRouterImpl, RoundRobinStrategy},
    PoolRegistry,
};
use proptest::prelude::*;
use std::sync::Arc;
use tower::ServiceExt;

/// Create test API state
async fn create_test_state() -> ApiState<
    InMemoryPoolRegistry,
    JobRouterImpl<InMemoryPoolRegistry, RoundRobinStrategy>,
> {
    let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
        "test-key".to_string(),
    ]));

    // Register a test member
    let registration = MemberRegistration {
        node_id: "test-node".to_string(),
        peer_id: "12D3KooWtest".to_string(),
        capabilities: NodeCapabilities::default(),
        models: vec![ModelInfo {
            model_id: "gpt-4".to_string(),
            family: Some("gpt".to_string()),
            parameters: None,
            context_length: Some(8192),
            quantization: None,
        }],
        lightning_pubkey: "02abc".to_string(),
        multiaddrs: vec!["/ip4/127.0.0.1/tcp/9000".to_string()],
        registration_key: "test-key".to_string(),
    };

    registry.register_member(registration).await.unwrap();
    registry
        .update_member_status("test-node", MemberStatus::Active)
        .await
        .unwrap();

    let router = Arc::new(JobRouterImpl::new(
        Arc::clone(&registry),
        RoundRobinStrategy::new(),
    ));

    ApiState::new(registry, router, PoolConfig::default())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Feature: api-integration
    /// Property 5: Health Endpoint Always Returns 200
    ///
    /// The /health endpoint SHALL always return HTTP 200 with valid JSON.
    #[test]
    fn property_5_health_endpoint_always_200(_dummy in 0..10u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let request = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::OK);

            // Parse body
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            prop_assert!(json.get("status").is_some());
            prop_assert!(json.get("version").is_some());
            prop_assert!(json.get("timestamp").is_some());

            Ok(())
        })?;
    }

    /// Feature: api-integration
    /// Property 6: Models Endpoint Lists Available Models
    ///
    /// GET /v1/models SHALL return an OpenAI-compatible model list.
    #[test]
    fn property_6_models_endpoint_lists_available(_dummy in 0..10u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let request = Request::builder()
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::OK);

            // Parse body
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Check OpenAI format
            prop_assert_eq!(json.get("object").and_then(|v| v.as_str()), Some("list"));
            prop_assert!(json.get("data").is_some());

            let data = json.get("data").unwrap().as_array().unwrap();
            // Should have at least one model (gpt-4 from our test setup)
            prop_assert!(!data.is_empty());

            for model in data {
                prop_assert!(model.get("id").is_some());
                prop_assert_eq!(model.get("object").and_then(|v| v.as_str()), Some("model"));
                prop_assert!(model.get("created").is_some());
                prop_assert!(model.get("owned_by").is_some());
            }

            Ok(())
        })?;
    }

    /// Feature: api-integration
    /// Property: Pool Status Returns Valid Data
    #[test]
    fn pool_status_returns_valid_data(_dummy in 0..10u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let request = Request::builder()
                .uri("/v1/pool/status")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            prop_assert!(json.get("pool_id").is_some());
            prop_assert!(json.get("active_members").is_some());
            prop_assert!(json.get("models_available").is_some());

            Ok(())
        })?;
    }

    /// Feature: api-integration
    /// Property: Chat Completions Returns Valid Response
    #[test]
    fn chat_completions_returns_valid_response(_dummy in 0..5u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let body = serde_json::json!({
                "model": "gpt-4",
                "messages": [
                    {"role": "user", "content": "Hello, world!"}
                ]
            });

            let request = Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Verify OpenAI response format
            prop_assert!(json.get("id").is_some());
            prop_assert_eq!(json.get("object").and_then(|v| v.as_str()), Some("chat.completion"));
            prop_assert!(json.get("choices").is_some());
            prop_assert!(json.get("usage").is_some());

            Ok(())
        })?;
    }

    /// Feature: api-integration
    /// Property: Invalid Requests Return 400
    #[test]
    fn invalid_chat_request_returns_400(_dummy in 0..5u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            // Missing messages
            let body = serde_json::json!({
                "model": "gpt-4"
            });

            let request = Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            // Should return 400 or 422 for validation errors
            prop_assert!(
                response.status() == StatusCode::BAD_REQUEST ||
                response.status() == StatusCode::UNPROCESSABLE_ENTITY,
                "Expected 400 or 422, got {}",
                response.status()
            );

            Ok(())
        })?;
    }

    /// Feature: api-integration  
    /// Property: Streaming Returns SSE Format
    #[test]
    fn streaming_returns_sse_format(_dummy in 0..3u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let body = serde_json::json!({
                "model": "gpt-4",
                "messages": [
                    {"role": "user", "content": "Hello"}
                ],
                "stream": true
            });

            let request = Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::OK);

            // Check SSE headers
            let content_type = response.headers().get("content-type");
            prop_assert!(
                content_type.is_some(),
                "Should have content-type header"
            );
            prop_assert!(
                content_type.unwrap().to_str().unwrap().contains("text/event-stream"),
                "Content-type should be text/event-stream"
            );

            Ok(())
        })?;
    }

    /// Feature: api-integration
    /// Property: Model Not Found Returns 404
    #[test]
    fn model_not_found_returns_404(_dummy in 0..5u32) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let state = create_test_state().await;
            let app = create_router(state);

            let body = serde_json::json!({
                "model": "nonexistent-model",
                "messages": [
                    {"role": "user", "content": "Hello"}
                ]
            });

            let request = Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            prop_assert_eq!(response.status(), StatusCode::NOT_FOUND);

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let state = create_test_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .uri("/ready")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("ready").unwrap().as_bool().unwrap());
        assert_eq!(json.get("active_members").unwrap().as_i64().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_embeddings_endpoint() {
        let state = create_test_state().await;
        let app = create_router(state);

        let body = serde_json::json!({
            "model": "gpt-4",
            "input": "Hello, world!"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/v1/embeddings")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.get("object").unwrap(), "list");
        assert!(json.get("data").is_some());
        assert!(json.get("usage").is_some());
    }
}

