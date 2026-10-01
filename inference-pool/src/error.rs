//! Error types for the Inference Pool.
//!
//! All errors are designed to be OpenAI-compatible, following the format:
//! ```json
//! {
//!   "error": {
//!     "message": "Human-readable error description",
//!     "type": "error_type_string",
//!     "param": "field_name_if_applicable",
//!     "code": "specific_error_code"
//!   }
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type alias for Pool operations
pub type PoolResult<T> = Result<T, PoolError>;

/// OpenAI-compatible error types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolErrorType {
    /// Malformed request, missing required fields (400)
    InvalidRequestError,
    /// Invalid or missing API key (401)
    AuthenticationError,
    /// Client exceeded rate limits (429)
    RateLimitError,
    /// Requested model not available in pool (404)
    ModelNotFound,
    /// No Pool_Members available (503)
    ServiceUnavailable,
    /// Unexpected server error (500)
    InternalError,
    /// Registration key invalid (401)
    RegistrationError,
    /// Node not found in registry (404)
    NodeNotFound,
    /// Channel operation failed (500)
    ChannelError,
    /// Billing operation failed (500)
    BillingError,
}

impl PoolErrorType {
    /// Get the string representation for OpenAI compatibility
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequestError => "invalid_request_error",
            Self::AuthenticationError => "authentication_error",
            Self::RateLimitError => "rate_limit_error",
            Self::ModelNotFound => "model_not_found",
            Self::ServiceUnavailable => "service_unavailable",
            Self::InternalError => "internal_error",
            Self::RegistrationError => "registration_error",
            Self::NodeNotFound => "node_not_found",
            Self::ChannelError => "channel_error",
            Self::BillingError => "billing_error",
        }
    }

    /// Get the HTTP status code for this error type
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequestError => StatusCode::BAD_REQUEST,
            Self::AuthenticationError => StatusCode::UNAUTHORIZED,
            Self::RateLimitError => StatusCode::TOO_MANY_REQUESTS,
            Self::ModelNotFound => StatusCode::NOT_FOUND,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RegistrationError => StatusCode::UNAUTHORIZED,
            Self::NodeNotFound => StatusCode::NOT_FOUND,
            Self::ChannelError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BillingError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for PoolErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// OpenAI-compatible error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Human-readable error message
    pub message: String,
    /// Error type string (e.g., "invalid_request_error")
    #[serde(rename = "type")]
    pub error_type: String,
    /// Field name if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// Specific error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// OpenAI-compatible error response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Main error type for the Inference Pool
#[derive(Debug, Clone)]
pub struct PoolError {
    /// Error type classification
    pub error_type: PoolErrorType,
    /// Human-readable message
    pub message: String,
    /// Optional parameter name (for validation errors)
    pub param: Option<String>,
    /// Optional specific error code
    pub code: Option<String>,
    /// Optional source error message (for debugging, not exposed to client)
    pub source: Option<String>,
}

impl PoolError {
    /// Create a new PoolError
    pub fn new(error_type: PoolErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            message: message.into(),
            param: None,
            code: None,
            source: None,
        }
    }

    /// Add parameter name to the error
    pub fn with_param(mut self, param: impl Into<String>) -> Self {
        self.param = Some(param.into());
        self
    }

    /// Add specific error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add source error (for debugging)
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        self.error_type.status_code()
    }

    /// Convert to OpenAI-compatible error response
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                message: self.message.clone(),
                error_type: self.error_type.as_str().to_string(),
                param: self.param.clone(),
                code: self.code.clone(),
            },
        }
    }

    // Convenience constructors for common errors

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::InvalidRequestError, message)
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::AuthenticationError, message)
    }

    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::RateLimitError, message)
    }

    /// Create a model not found error
    pub fn model_not_found(model: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(
            PoolErrorType::ModelNotFound,
            format!("Model '{}' not found in pool", model),
        )
        .with_param("model")
        .with_code("model_not_found")
    }

    /// Create a service unavailable error
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::ServiceUnavailable, message)
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::InternalError, message)
    }

    /// Create a registration error
    pub fn registration(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::RegistrationError, message)
    }

    /// Create a node not found error
    pub fn node_not_found(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        Self::new(
            PoolErrorType::NodeNotFound,
            format!("Node '{}' not found in registry", node_id),
        )
        .with_param("node_id")
    }

    /// Create a no members available error
    pub fn no_members_available(model: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(
            PoolErrorType::ServiceUnavailable,
            format!("No pool members available for model '{}'", model),
        )
        .with_code("no_members_available")
    }

    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            PoolErrorType::InvalidRequestError,
            format!("Missing required field: {}", field),
        )
        .with_param(&field)
        .with_code("missing_field")
    }

    /// Create an invalid field error
    pub fn invalid_field(field: impl Into<String>, reason: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            PoolErrorType::InvalidRequestError,
            format!("Invalid field '{}': {}", field, reason.into()),
        )
        .with_param(&field)
        .with_code("invalid_field")
    }

    /// Create an API key missing error
    pub fn api_key_missing() -> Self {
        Self::new(
            PoolErrorType::AuthenticationError,
            "Missing API key. Please include 'Authorization: Bearer <api_key>' header.",
        )
        .with_code("api_key_missing")
    }

    /// Create an invalid API key error
    pub fn api_key_invalid() -> Self {
        Self::new(
            PoolErrorType::AuthenticationError,
            "Invalid API key provided.",
        )
        .with_code("api_key_invalid")
    }

    /// Create an expired API key error
    pub fn api_key_expired() -> Self {
        Self::new(PoolErrorType::AuthenticationError, "API key has expired.")
            .with_code("api_key_expired")
    }

    /// Create a generic not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::NodeNotFound, message)
            .with_code("not_found")
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::InvalidRequestError, message)
            .with_code("validation_error")
    }

    /// Create a billing error
    pub fn billing(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::BillingError, message)
            .with_code("billing_error")
    }

    /// Create a payment error
    pub fn payment(message: impl Into<String>) -> Self {
        Self::new(PoolErrorType::BillingError, message)
            .with_code("payment_error")
    }
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error_type, self.message)
    }
}

impl std::error::Error for PoolError {}

/// Implement IntoResponse for Axum integration
impl IntoResponse for PoolError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.to_response());

        // Log the error with source if available
        if let Some(source) = &self.source {
            tracing::error!(
                error_type = %self.error_type,
                message = %self.message,
                source = %source,
                "Pool error occurred"
            );
        } else {
            tracing::warn!(
                error_type = %self.error_type,
                message = %self.message,
                "Pool error occurred"
            );
        }

        (status, body).into_response()
    }
}

/// Conversion from anyhow::Error
impl From<anyhow::Error> for PoolError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal("An internal error occurred").with_source(err.to_string())
    }
}

/// Conversion from std::io::Error
impl From<std::io::Error> for PoolError {
    fn from(err: std::io::Error) -> Self {
        Self::internal("IO error occurred").with_source(err.to_string())
    }
}

/// Conversion from serde_json::Error
impl From<serde_json::Error> for PoolError {
    fn from(err: serde_json::Error) -> Self {
        Self::invalid_request(format!("JSON parsing error: {}", err))
            .with_code("json_parse_error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_status_codes() {
        assert_eq!(
            PoolErrorType::InvalidRequestError.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            PoolErrorType::AuthenticationError.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            PoolErrorType::RateLimitError.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            PoolErrorType::ModelNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            PoolErrorType::ServiceUnavailable.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            PoolErrorType::InternalError.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_type_as_str() {
        assert_eq!(
            PoolErrorType::InvalidRequestError.as_str(),
            "invalid_request_error"
        );
        assert_eq!(
            PoolErrorType::AuthenticationError.as_str(),
            "authentication_error"
        );
        assert_eq!(PoolErrorType::ModelNotFound.as_str(), "model_not_found");
    }

    #[test]
    fn test_error_to_response() {
        let error = PoolError::model_not_found("gpt-4");
        let response = error.to_response();

        assert_eq!(response.error.error_type, "model_not_found");
        assert!(response.error.message.contains("gpt-4"));
        assert_eq!(response.error.param, Some("model".to_string()));
        assert_eq!(response.error.code, Some("model_not_found".to_string()));
    }

    #[test]
    fn test_error_chaining() {
        let error = PoolError::new(PoolErrorType::InvalidRequestError, "Test error")
            .with_param("test_field")
            .with_code("test_code")
            .with_source("underlying error");

        assert_eq!(error.message, "Test error");
        assert_eq!(error.param, Some("test_field".to_string()));
        assert_eq!(error.code, Some("test_code".to_string()));
        assert_eq!(error.source, Some("underlying error".to_string()));
    }

    #[test]
    fn test_common_error_constructors() {
        let missing = PoolError::missing_field("model");
        assert_eq!(missing.error_type, PoolErrorType::InvalidRequestError);
        assert!(missing.message.contains("model"));

        let auth = PoolError::api_key_missing();
        assert_eq!(auth.error_type, PoolErrorType::AuthenticationError);

        let unavailable = PoolError::no_members_available("llama-7b");
        assert_eq!(unavailable.error_type, PoolErrorType::ServiceUnavailable);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = PoolError::invalid_request("Bad request")
            .with_param("messages")
            .with_code("invalid_messages");
        let response = error.to_response();
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"message\":\"Bad request\""));
        assert!(json.contains("\"type\":\"invalid_request_error\""));
        assert!(json.contains("\"param\":\"messages\""));
        assert!(json.contains("\"code\":\"invalid_messages\""));
    }
}


