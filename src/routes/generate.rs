use axum::{extract::State, http::StatusCode, Json};

use crate::{
    backend::{GenerateRequest, GenerateResponse, JobStatus},
    AppState,
};

/// Submit a video generation job
#[utoipa::path(
    post,
    path = "/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Job accepted", body = GenerateResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error"),
    ),
    security(("bearer" = [])),
    tag = "Video Generation"
)]
pub async fn handle_generate(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, String)> {
    state
        .backend
        .generate(request, &state.http_client)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!(error = %e, "Generate failed");
            sentry::capture_error(&*e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

/// Check the status of a generation job
#[utoipa::path(
    get,
    path = "/result/{job_id}",
    params(("job_id" = String, Path, description = "Job identifier")),
    responses(
        (status = 200, description = "Job status", body = JobStatus),
        (status = 404, description = "Job not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer" = [])),
    tag = "Video Generation"
)]
pub async fn handle_get_result(
    State(state): State<AppState>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> Result<Json<JobStatus>, (StatusCode, String)> {
    match state
        .backend
        .get_job_status(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        Some(status) => Ok(Json(status)),
        None => Err((StatusCode::NOT_FOUND, "Job not found".into())),
    }
}
