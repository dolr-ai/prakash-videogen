use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};
use utoipa::ToSchema;

/// Webhook configuration from the generate request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookConfig {
    /// URL to POST the result to on completion
    pub url: String,
    /// Extra parameters to include in the webhook payload
    #[serde(default)]
    pub extra_params: Value,
}

/// Webhook payload sent to the off-chain-agent
#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookPayload {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<OutputFile>>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Output file metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OutputFile {
    /// Output filename
    pub filename: String,
    /// Local filesystem path (on the GPU server)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    /// Public URL if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Output subfolder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subfolder: Option<String>,
    /// ComfyUI node that produced this output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Type of output (videos, images)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_type: Option<String>,
}

/// Send a webhook notification to the off-chain-agent.
/// Retries once on transient failure.
pub async fn send_webhook(
    client: &reqwest::Client,
    webhook: &WebhookConfig,
    job_id: &str,
    status: &str,
    output: Option<Vec<OutputFile>>,
    message: Option<String>,
) -> Result<()> {
    let payload = WebhookPayload {
        id: job_id.to_string(),
        status: status.to_string(),
        message,
        output,
        extra: webhook.extra_params.clone(),
    };

    info!(job_id, status, url = %webhook.url, "Sending webhook");

    const MAX_RETRIES: u32 = 2;

    for attempt in 0..=MAX_RETRIES {
        let result = client
            .post(&webhook.url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                info!(job_id, attempt, "Webhook sent successfully");
                return Ok(());
            }
            Ok(resp) => {
                let status_code = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error!(job_id, attempt, %status_code, body, "Webhook request failed with status");

                if attempt == MAX_RETRIES {
                    anyhow::bail!("Webhook failed after retries: {status_code} {body}");
                }
            }
            Err(e) => {
                error!(job_id, attempt, error = %e, "Webhook request network error");

                if attempt == MAX_RETRIES {
                    anyhow::bail!("Webhook failed after retries: {e}");
                }
            }
        }

        info!(job_id, "Retrying webhook in 2 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    anyhow::bail!("Webhook failed unexpectedly")
}
