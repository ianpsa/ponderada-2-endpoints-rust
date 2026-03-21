use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("queue unavailable: {0}")]
    Queue(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),
}

// Handling errors

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
    	let(status, message) = match self {
     		AppError::Queue(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
     		AppError::InvalidPayload(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
     	};
     	let body = Json(json!({
    		"error": message
    	}));
      
    	(status, body).into_response()
    }
}
