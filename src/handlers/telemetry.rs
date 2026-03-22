use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::TelemetryPayload;
use crate::queue::publisher::Publisher;

pub async fn post_telemetry(
    State(publisher): State<Arc<Publisher>>,
    Json(payload): Json<TelemetryPayload>,
) -> Result<impl IntoResponse, AppError> {
    publisher
        .publish(&payload)
        .await
        .map_err(|e| AppError::Queue(e.to_string()))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"status": "enqueued"})),
    ))
}
