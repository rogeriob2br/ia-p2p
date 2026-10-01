//! Property tests for error handling.
//!
//! Feature: api-integration
//! Tests Properties 15 and 16 from the design document.

use inference_pool::error::{ErrorResponse, PoolError, PoolErrorType};
use proptest::prelude::*;

/// Strategy for generating random error types
fn error_type_strategy() -> impl Strategy<Value = PoolErrorType> {
    prop_oneof![
        Just(PoolErrorType::InvalidRequestError),
        Just(PoolErrorType::AuthenticationError),
        Just(PoolErrorType::RateLimitError),
        Just(PoolErrorType::ModelNotFound),
        Just(PoolErrorType::ServiceUnavailable),
        Just(PoolErrorType::InternalError),
        Just(PoolErrorType::RegistrationError),
        Just(PoolErrorType::NodeNotFound),
        Just(PoolErrorType::ChannelError),
        Just(PoolErrorType::BillingError),
    ]
}

/// Strategy for generating random error messages
fn message_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 .,!?'-]{1,200}")
        .unwrap()
        .prop_filter("message not empty", |s| !s.is_empty())
}

/// Strategy for generating optional param names
fn param_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(
        prop::string::string_regex("[a-z_]{1,30}")
            .unwrap()
            .prop_filter("param not empty", |s| !s.is_empty()),
    )
}

/// Strategy for generating optional error codes
fn code_strategy() -> impl Strategy<Value = Option<String>> {
    prop::option::of(
        prop::string::string_regex("[a-z_]{1,30}")
            .unwrap()
            .prop_filter("code not empty", |s| !s.is_empty()),
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: api-integration
    /// Property 15: Error Responses Follow OpenAI Format
    ///
    /// For any error during processing, the response SHALL contain
    /// `error.message`, `error.type`, and `error.code` in OpenAI format.
    ///
    /// Validates: Requirements 9.1
    #[test]
    fn property_15_error_responses_follow_openai_format(
        error_type in error_type_strategy(),
        message in message_strategy(),
        param in param_strategy(),
        code in code_strategy(),
    ) {
        // Create error with random properties
        let mut error = PoolError::new(error_type, message.clone());
        if let Some(p) = param.clone() {
            error = error.with_param(p);
        }
        if let Some(c) = code.clone() {
            error = error.with_code(c);
        }

        // Convert to response
        let response = error.to_response();

        // Verify OpenAI format
        // 1. Response has error object
        // 2. error.message exists and matches
        prop_assert_eq!(&response.error.message, &message);

        // 3. error.type exists and is valid string
        prop_assert!(!response.error.error_type.is_empty());
        prop_assert_eq!(&response.error.error_type, error_type.as_str());

        // 4. error.param matches if provided
        prop_assert_eq!(&response.error.param, &param);

        // 5. error.code matches if provided
        prop_assert_eq!(&response.error.code, &code);

        // 6. Response can be serialized to JSON
        let json = serde_json::to_string(&response);
        prop_assert!(json.is_ok());

        let json_str = json.unwrap();

        // 7. JSON contains required fields
        prop_assert!(json_str.contains("\"error\""));
        prop_assert!(json_str.contains("\"message\""));
        prop_assert!(json_str.contains("\"type\""));

        // 8. Response can be deserialized back
        let parsed: Result<ErrorResponse, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok());

        let parsed = parsed.unwrap();
        prop_assert_eq!(parsed.error.message, response.error.message);
        prop_assert_eq!(parsed.error.error_type, response.error.error_type);
    }

    /// Feature: api-integration
    /// Property 16: Invalid Payload Returns 400
    ///
    /// For any request with invalid payload structure, the Inference_Pool
    /// SHALL return HTTP 400 with type `invalid_request_error`.
    ///
    /// Validates: Requirements 9.3
    #[test]
    fn property_16_invalid_payload_returns_400(
        field_name in prop::string::string_regex("[a-z_]{1,20}").unwrap(),
        reason in message_strategy(),
    ) {
        // Create invalid request error for missing field
        let missing_error = PoolError::missing_field(&field_name);

        // Verify status code is 400
        prop_assert_eq!(
            missing_error.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );

        // Verify error type is invalid_request_error
        prop_assert_eq!(
            missing_error.error_type,
            PoolErrorType::InvalidRequestError
        );

        // Verify param is set to the field name
        let response = missing_error.to_response();
        prop_assert!(response.error.param.is_some());
        prop_assert_eq!(
            response.error.param.as_ref().unwrap(),
            &field_name
        );

        // Verify error type string in response
        prop_assert_eq!(
            response.error.error_type,
            "invalid_request_error"
        );

        // Create invalid request error for invalid field value
        let invalid_error = PoolError::invalid_field(&field_name, &reason);

        // Verify same properties for invalid field error
        prop_assert_eq!(
            invalid_error.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        prop_assert_eq!(
            invalid_error.error_type,
            PoolErrorType::InvalidRequestError
        );

        let response = invalid_error.to_response();
        prop_assert_eq!(
            response.error.error_type,
            "invalid_request_error"
        );
    }

    /// Additional property: Error type status codes are correct
    #[test]
    fn error_type_status_codes_are_correct(
        error_type in error_type_strategy(),
    ) {
        let status = error_type.status_code();

        match error_type {
            PoolErrorType::InvalidRequestError => {
                prop_assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
            }
            PoolErrorType::AuthenticationError |
            PoolErrorType::RegistrationError => {
                prop_assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
            }
            PoolErrorType::RateLimitError => {
                prop_assert_eq!(status, axum::http::StatusCode::TOO_MANY_REQUESTS);
            }
            PoolErrorType::ModelNotFound |
            PoolErrorType::NodeNotFound => {
                prop_assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
            }
            PoolErrorType::ServiceUnavailable => {
                prop_assert_eq!(status, axum::http::StatusCode::SERVICE_UNAVAILABLE);
            }
            PoolErrorType::InternalError |
            PoolErrorType::ChannelError |
            PoolErrorType::BillingError => {
                prop_assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    /// Additional property: Error type string representation is valid
    #[test]
    fn error_type_string_representation_is_valid(
        error_type in error_type_strategy(),
    ) {
        let type_str = error_type.as_str();

        // String is not empty
        prop_assert!(!type_str.is_empty());

        // String is snake_case (lowercase with underscores)
        prop_assert!(type_str.chars().all(|c| c.is_lowercase() || c == '_'));

        // String matches expected pattern
        prop_assert!(type_str.contains('_') || type_str.len() < 20);
    }

    /// Additional property: Common errors have correct properties
    #[test]
    fn common_errors_have_correct_properties(
        model_name in prop::string::string_regex("[a-z0-9-]{1,50}").unwrap(),
        node_id in prop::string::string_regex("[a-z0-9-]{1,50}").unwrap(),
    ) {
        // Model not found error
        let model_error = PoolError::model_not_found(&model_name);
        prop_assert_eq!(model_error.error_type, PoolErrorType::ModelNotFound);
        prop_assert!(model_error.message.contains(&model_name));
        prop_assert_eq!(model_error.param.clone(), Some("model".to_string()));
        prop_assert_eq!(model_error.status_code(), axum::http::StatusCode::NOT_FOUND);

        // Node not found error
        let node_error = PoolError::node_not_found(&node_id);
        prop_assert_eq!(node_error.error_type, PoolErrorType::NodeNotFound);
        prop_assert!(node_error.message.contains(&node_id));
        prop_assert_eq!(node_error.param.clone(), Some("node_id".to_string()));

        // API key errors
        let missing_key = PoolError::api_key_missing();
        prop_assert_eq!(missing_key.error_type, PoolErrorType::AuthenticationError);
        prop_assert_eq!(missing_key.status_code(), axum::http::StatusCode::UNAUTHORIZED);

        let invalid_key = PoolError::api_key_invalid();
        prop_assert_eq!(invalid_key.error_type, PoolErrorType::AuthenticationError);

        // Service unavailable
        let no_members = PoolError::no_members_available(&model_name);
        prop_assert_eq!(no_members.error_type, PoolErrorType::ServiceUnavailable);
        prop_assert_eq!(no_members.status_code(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_error_response_json_structure() {
        let error = PoolError::invalid_request("Test error")
            .with_param("test_field")
            .with_code("test_code");

        let response = error.to_response();
        let json = serde_json::to_value(&response).unwrap();

        // Verify JSON structure
        assert!(json.is_object());
        assert!(json.get("error").is_some());

        let error_obj = json.get("error").unwrap();
        assert!(error_obj.get("message").is_some());
        assert!(error_obj.get("type").is_some());
        assert!(error_obj.get("param").is_some());
        assert!(error_obj.get("code").is_some());

        // Verify values
        assert_eq!(error_obj.get("message").unwrap(), "Test error");
        assert_eq!(error_obj.get("type").unwrap(), "invalid_request_error");
        assert_eq!(error_obj.get("param").unwrap(), "test_field");
        assert_eq!(error_obj.get("code").unwrap(), "test_code");
    }

    #[test]
    fn test_error_without_optional_fields() {
        let error = PoolError::internal("Internal error");
        let response = error.to_response();
        let json = serde_json::to_string(&response).unwrap();

        // Optional fields should not be serialized
        assert!(!json.contains("\"param\""));
        assert!(!json.contains("\"code\""));
    }

    #[test]
    fn test_all_error_types_have_valid_status() {
        let error_types = [
            PoolErrorType::InvalidRequestError,
            PoolErrorType::AuthenticationError,
            PoolErrorType::RateLimitError,
            PoolErrorType::ModelNotFound,
            PoolErrorType::ServiceUnavailable,
            PoolErrorType::InternalError,
            PoolErrorType::RegistrationError,
            PoolErrorType::NodeNotFound,
            PoolErrorType::ChannelError,
            PoolErrorType::BillingError,
        ];

        for error_type in error_types {
            let status = error_type.status_code();
            // Status should be a valid HTTP status
            assert!(status.as_u16() >= 400);
            assert!(status.as_u16() < 600);
        }
    }
}

