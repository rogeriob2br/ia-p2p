//! Property tests for OpenAI-compatible data models.
//!
//! Feature: api-integration
//! Tests Properties 3, 4, 7, 8 from the design document.

use inference_pool::models::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, EmbeddingInput, EmbeddingRequest,
    EmbeddingResponse, EmbeddingUsage, MessageRole, Usage,
};
use proptest::prelude::*;

/// Strategy for generating valid message roles
fn message_role_strategy() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

/// Strategy for generating valid chat messages
fn chat_message_strategy() -> impl Strategy<Value = ChatMessage> {
    (
        message_role_strategy(),
        prop::string::string_regex("[a-zA-Z0-9 .,!?'-]{1,500}").unwrap(),
        prop::option::of(prop::string::string_regex("[a-z_]{1,20}").unwrap()),
    )
        .prop_map(|(role, content, name)| ChatMessage { role, content, name })
}

/// Strategy for generating valid model names
fn model_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9-]{3,50}")
        .unwrap()
        .prop_filter("model not empty", |s| !s.is_empty())
}

/// Strategy for generating valid chat completion requests
fn chat_request_strategy() -> impl Strategy<Value = ChatCompletionRequest> {
    (
        model_name_strategy(),
        prop::collection::vec(chat_message_strategy(), 1..10),
        prop::option::of(0.0f32..2.0f32),
        prop::option::of(1u32..4096u32),
        prop::bool::ANY,
        prop::option::of(0.0f32..1.0f32),
    )
        .prop_map(
            |(model, messages, temperature, max_tokens, stream, top_p)| ChatCompletionRequest {
                model,
                messages,
                temperature,
                max_tokens,
                stream,
                top_p,
                frequency_penalty: None,
                presence_penalty: None,
                stop: None,
                user: None,
                n: None,
                logprobs: None,
                seed: None,
            },
        )
}

/// Strategy for generating embedding inputs
fn embedding_input_strategy() -> impl Strategy<Value = EmbeddingInput> {
    prop_oneof![
        prop::string::string_regex("[a-zA-Z0-9 ]{1,100}")
            .unwrap()
            .prop_map(EmbeddingInput::Single),
        prop::collection::vec(
            prop::string::string_regex("[a-zA-Z0-9 ]{1,100}").unwrap(),
            1..5
        )
        .prop_map(EmbeddingInput::Multiple),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: api-integration
    /// Property 3: OpenAI Request Format Acceptance
    ///
    /// For any valid OpenAI-format chat completion request, the Inference_Pool
    /// SHALL accept the payload without format errors.
    ///
    /// Validates: Requirements 2.1
    #[test]
    fn property_3_openai_request_format_acceptance(
        request in chat_request_strategy(),
    ) {
        // Serialize the request to JSON
        let json_result = serde_json::to_string(&request);
        prop_assert!(json_result.is_ok(), "Request should serialize to JSON");

        let json = json_result.unwrap();

        // Verify JSON contains required fields
        prop_assert!(json.contains("\"model\""), "JSON should contain model field");
        prop_assert!(json.contains("\"messages\""), "JSON should contain messages field");

        // Deserialize back
        let parsed_result: Result<ChatCompletionRequest, _> = serde_json::from_str(&json);
        prop_assert!(parsed_result.is_ok(), "Should deserialize back from JSON");

        let parsed = parsed_result.unwrap();

        // Verify round-trip preserves data
        prop_assert_eq!(&parsed.model, &request.model);
        prop_assert_eq!(parsed.messages.len(), request.messages.len());
        prop_assert_eq!(parsed.temperature, request.temperature);
        prop_assert_eq!(parsed.max_tokens, request.max_tokens);
        prop_assert_eq!(parsed.stream, request.stream);

        // Validate the request
        let validation = parsed.validate();
        prop_assert!(validation.is_ok(), "Valid request should pass validation");
    }

    /// Feature: api-integration
    /// Property 4: Message Roles Parsing
    ///
    /// For any messages array containing roles (system, user, assistant),
    /// the Inference_Pool SHALL correctly parse and preserve all message
    /// roles and content.
    ///
    /// Validates: Requirements 2.2
    #[test]
    fn property_4_message_roles_parsing(
        messages in prop::collection::vec(chat_message_strategy(), 1..10),
    ) {
        // Create a request with these messages
        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: messages.clone(),
            temperature: None,
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

        // Serialize
        let json = serde_json::to_string(&request).unwrap();

        // Deserialize
        let parsed: ChatCompletionRequest = serde_json::from_str(&json).unwrap();

        // Verify all messages preserved
        prop_assert_eq!(parsed.messages.len(), messages.len());

        for (original, parsed_msg) in messages.iter().zip(parsed.messages.iter()) {
            // Role preserved
            prop_assert_eq!(parsed_msg.role, original.role);

            // Content preserved
            prop_assert_eq!(&parsed_msg.content, &original.content);

            // Name preserved (if present)
            prop_assert_eq!(&parsed_msg.name, &original.name);
        }

        // Verify role serialization format
        for msg in &parsed.messages {
            let msg_json = serde_json::to_string(msg).unwrap();

            match msg.role {
                MessageRole::System => {
                    prop_assert!(msg_json.contains("\"role\":\"system\""));
                }
                MessageRole::User => {
                    prop_assert!(msg_json.contains("\"role\":\"user\""));
                }
                MessageRole::Assistant => {
                    prop_assert!(msg_json.contains("\"role\":\"assistant\""));
                }
                _ => {}
            }
        }
    }

    /// Feature: api-integration
    /// Property 7: Embedding Input Flexibility
    ///
    /// For any embedding request, the input SHALL be accepted as either
    /// a single string or an array of strings.
    ///
    /// Validates: Requirements 3.1, 3.2
    #[test]
    fn property_7_embedding_input_flexibility(
        input in embedding_input_strategy(),
        model in model_name_strategy(),
    ) {
        let request = EmbeddingRequest {
            model: model.clone(),
            input: input.clone(),
            encoding_format: None,
            user: None,
            dimensions: None,
        };

        // Serialize
        let json = serde_json::to_string(&request).unwrap();

        // Verify model is present
        let expected_model = format!("\"model\":\"{}\"", model);
        prop_assert!(json.contains(&expected_model), "JSON should contain model");

        // Deserialize
        let parsed: EmbeddingRequest = serde_json::from_str(&json).unwrap();

        // Verify input preserved
        match (&input, &parsed.input) {
            (EmbeddingInput::Single(s1), EmbeddingInput::Single(s2)) => {
                prop_assert_eq!(s1, s2);
            }
            (EmbeddingInput::Multiple(v1), EmbeddingInput::Multiple(v2)) => {
                prop_assert_eq!(v1.len(), v2.len());
                for (a, b) in v1.iter().zip(v2.iter()) {
                    prop_assert_eq!(a, b);
                }
            }
            _ => {
                prop_assert!(false, "Input type changed after serialization");
            }
        }

        // Validate
        let validation = parsed.validate();
        prop_assert!(validation.is_ok());
    }

    /// Feature: api-integration
    /// Property 8: Embedding Order Preservation
    ///
    /// For any embedding request with N inputs, the response SHALL contain
    /// exactly N embeddings in the same order as the inputs.
    ///
    /// Validates: Requirements 3.3
    #[test]
    fn property_8_embedding_order_preservation(
        inputs in prop::collection::vec(
            prop::string::string_regex("[a-zA-Z0-9 ]{1,50}").unwrap(),
            1..10
        ),
    ) {
        let n = inputs.len();

        // Simulate embeddings (in real scenario, these would come from the model)
        let embeddings: Vec<Vec<f32>> = (0..n)
            .map(|i| vec![i as f32; 128]) // 128-dimensional embeddings
            .collect();

        // Create response
        let response = EmbeddingResponse::new(
            "text-embedding-ada-002".to_string(),
            embeddings.clone(),
            EmbeddingUsage {
                prompt_tokens: inputs.iter().map(|s| s.len() as u64).sum(),
                total_tokens: inputs.iter().map(|s| s.len() as u64).sum(),
            },
        );

        // Verify count matches
        prop_assert_eq!(response.data.len(), n, "Should have N embeddings");

        // Verify order preserved (index field)
        for (i, embedding_data) in response.data.iter().enumerate() {
            prop_assert_eq!(
                embedding_data.index as usize, i,
                "Embedding index should match position"
            );

            // Verify embedding content matches
            prop_assert_eq!(
                embedding_data.embedding.len(), 128,
                "Embedding dimension should be preserved"
            );

            // First element should match the input index
            prop_assert!((embedding_data.embedding[0] - i as f32).abs() < 0.001);
        }

        // Verify object types
        prop_assert_eq!(&response.object, "list");
        for data in &response.data {
            prop_assert_eq!(&data.object, "embedding");
        }
    }

    /// Additional property: Chat completion response has required fields
    #[test]
    fn chat_response_has_required_fields(
        model in model_name_strategy(),
        content in prop::string::string_regex("[a-zA-Z0-9 .,!?'-]{1,200}").unwrap(),
        prompt_tokens in 1u64..1000u64,
        completion_tokens in 1u64..1000u64,
    ) {
        let response = ChatCompletionResponse::new(
            model.clone(),
            ChatMessage {
                role: MessageRole::Assistant,
                content: content.clone(),
                name: None,
            },
            Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        );

        // Verify ID format
        prop_assert!(response.id.starts_with("chatcmpl-"), "ID should have correct prefix");

        // Verify object type
        prop_assert_eq!(&response.object, "chat.completion");

        // Verify model
        prop_assert_eq!(&response.model, &model);

        // Verify choices
        prop_assert_eq!(response.choices.len(), 1);
        prop_assert_eq!(response.choices[0].index, 0);
        prop_assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        prop_assert_eq!(&response.choices[0].message.content, &content);
        prop_assert_eq!(&response.choices[0].finish_reason, &Some("stop".to_string()));

        // Verify usage
        prop_assert_eq!(response.usage.prompt_tokens, prompt_tokens);
        prop_assert_eq!(response.usage.completion_tokens, completion_tokens);
        prop_assert_eq!(response.usage.total_tokens, prompt_tokens + completion_tokens);

        // Verify serialization
        let json = serde_json::to_string(&response).unwrap();
        prop_assert!(json.contains("\"id\""));
        prop_assert!(json.contains("\"object\""));
        prop_assert!(json.contains("\"choices\""));
        prop_assert!(json.contains("\"usage\""));
    }

    /// Additional property: Invalid requests fail validation
    #[test]
    fn invalid_requests_fail_validation(
        temperature in 2.1f32..10.0f32,  // Invalid: > 2.0
    ) {
        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(temperature),
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

        let validation = request.validate();
        prop_assert!(validation.is_err(), "Invalid temperature should fail validation");
    }

    /// Additional property: Empty model fails validation
    #[test]
    fn empty_model_fails_validation(
        messages in prop::collection::vec(chat_message_strategy(), 1..5),
    ) {
        let request = ChatCompletionRequest {
            model: "".to_string(),  // Invalid: empty
            messages,
            temperature: None,
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

        let validation = request.validate();
        prop_assert!(validation.is_err(), "Empty model should fail validation");
    }

    /// Additional property: Empty messages fails validation
    #[test]
    fn empty_messages_fails_validation(
        model in model_name_strategy(),
    ) {
        let request = ChatCompletionRequest {
            model,
            messages: vec![],  // Invalid: empty
            temperature: None,
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

        let validation = request.validate();
        prop_assert!(validation.is_err(), "Empty messages should fail validation");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_embedding_input_single_deserialization() {
        let json = r#"{"model":"ada","input":"Hello world"}"#;
        let request: EmbeddingRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(request.input, EmbeddingInput::Single(_)));
        if let EmbeddingInput::Single(s) = request.input {
            assert_eq!(s, "Hello world");
        }
    }

    #[test]
    fn test_embedding_input_multiple_deserialization() {
        let json = r#"{"model":"ada","input":["Hello","World"]}"#;
        let request: EmbeddingRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(request.input, EmbeddingInput::Multiple(_)));
        if let EmbeddingInput::Multiple(v) = request.input {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], "Hello");
            assert_eq!(v[1], "World");
        }
    }

    #[test]
    fn test_message_role_serialization() {
        let roles = [
            (MessageRole::System, "system"),
            (MessageRole::User, "user"),
            (MessageRole::Assistant, "assistant"),
        ];

        for (role, expected) in roles {
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }

    #[test]
    fn test_chat_request_minimal() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        }"#;

        let request: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
        assert!(!request.stream); // Default
    }

    #[test]
    fn test_chat_request_with_stream() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": true
        }"#;

        let request: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert!(request.stream);
    }
}

