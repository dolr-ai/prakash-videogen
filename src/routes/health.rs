use axum::{extract::State, Json};
use serde_json::json;

use crate::{backend::HealthResponse, AppState};

/// Check backend health
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health status", body = HealthResponse),
    ),
    security(("bearer" = [])),
    tag = "System"
)]
pub async fn handle_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let health = state.backend.health_check().await;
    match health {
        Ok(h) => Json(serde_json::to_value(h).unwrap_or_default()),
        Err(e) => Json(json!({
            "status": "error",
            "backend": state.backend.name(),
            "error": e.to_string(),
        })),
    }
}

/// Service info
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service info"),
    ),
    tag = "System"
)]
pub async fn handle_root(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "service": "videogen-worker",
        "version": env!("CARGO_PKG_VERSION"),
        "backend": state.backend.name(),
        "endpoints": ["/generate", "/result/{id}", "/upload/image", "/view", "/health", "/swagger-ui"],
    }))
}
