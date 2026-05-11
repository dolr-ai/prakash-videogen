use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};

use crate::{backend::UploadResponse, AppState};

/// Upload an image to the backend
#[utoipa::path(
    post,
    path = "/upload/image",
    request_body(content_type = "multipart/form-data", content = Vec<u8>,
        description = "Multipart form with an 'image' field containing the image file"),
    responses(
        (status = 200, description = "Upload successful", body = UploadResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Upload failed"),
    ),
    security(("bearer" = [])),
    tag = "Files"
)]
pub async fn handle_upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    let mut image_data: Option<(String, Bytes, String)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "image" {
            let filename = field.file_name().unwrap_or("upload.png").to_string();
            let content_type = field.content_type().unwrap_or("image/png").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {e}")))?;

            image_data = Some((filename, data, content_type));
        }
    }

    let (filename, data, content_type) =
        image_data.ok_or((StatusCode::BAD_REQUEST, "No 'image' field found".into()))?;

    state
        .backend
        .upload_image(&filename, data, &content_type)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!(error = %e, "Upload failed");
            sentry::capture_error(&*e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}
