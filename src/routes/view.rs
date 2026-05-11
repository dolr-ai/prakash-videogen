use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, Response, StatusCode},
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ViewParams {
    /// Output filename to retrieve
    pub filename: String,
    /// Subfolder within the output directory
    pub subfolder: Option<String>,
    /// File type (e.g. "output", "input")
    #[serde(rename = "type")]
    pub file_type: Option<String>,
}

/// Download a file from the backend's output storage
#[utoipa::path(
    get,
    path = "/view",
    params(ViewParams),
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "File fetch failed"),
    ),
    security(("bearer" = [])),
    tag = "Files"
)]
pub async fn handle_view(
    State(state): State<AppState>,
    Query(params): Query<ViewParams>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let (headers, body) = state
        .backend
        .get_file(
            &params.filename,
            params.subfolder.as_deref(),
            params.file_type.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, filename = %params.filename, "File fetch failed");
            sentry::capture_error(&*e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let content_disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    if let Some(cd) = content_disposition {
        response = response.header(header::CONTENT_DISPOSITION, cd);
    }

    response
        .body(Body::from(body))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
