use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("policy not found: {0}")]
    PolicyNotFound(u64),
    #[error("claim not found: {0}")]
    ClaimNotFound(u64),
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("policy not active")]
    PolicyNotActive,
    #[error("policy expired")]
    PolicyExpired,
    #[error("claim already processed")]
    ClaimAlreadyProcessed,
    #[error("insufficient pool")]
    InsufficientPool,
    #[error("not authorized")]
    NotAuthorized,
    #[allow(dead_code)]
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse { error: &'static str, code: u16, message: String }

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::PolicyNotFound(id) => (StatusCode::NOT_FOUND, 404, format!("Policy {id} not found")),
            ApiError::ClaimNotFound(id) => (StatusCode::NOT_FOUND, 404, format!("Claim {id} not found")),
            ApiError::InvalidAmount(m) => (StatusCode::BAD_REQUEST, 400, m.clone()),
            ApiError::InvalidInput(m) => (StatusCode::BAD_REQUEST, 400, m.clone()),
            ApiError::PolicyNotActive => (StatusCode::BAD_REQUEST, 400, "Policy is not active".into()),
            ApiError::PolicyExpired => (StatusCode::BAD_REQUEST, 400, "Policy has expired".into()),
            ApiError::ClaimAlreadyProcessed => (StatusCode::BAD_REQUEST, 400, "Claim already processed".into()),
            ApiError::InsufficientPool => (StatusCode::BAD_REQUEST, 400, "Insufficient pool balance".into()),
            ApiError::NotAuthorized => (StatusCode::FORBIDDEN, 403, "Not authorized".into()),
            ApiError::Internal(m) => { tracing::error!(error=%m, "Internal error"); (StatusCode::INTERNAL_SERVER_ERROR, 500, "Internal server error".into()) }
        };
        let label = match status {
            StatusCode::NOT_FOUND => "not_found",
            StatusCode::BAD_REQUEST => "bad_request",
            StatusCode::FORBIDDEN => "forbidden",
            _ => "internal_error",
        };
        (status, Json(ErrorResponse { error: label, code, message })).into_response()
    }
}
