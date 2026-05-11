use serde::{Deserialize, Serialize};

/// ComfyUI /prompt response
#[derive(Debug, Deserialize)]
pub struct PromptResponse {
    pub prompt_id: String,
}

/// ComfyUI WebSocket message types
#[derive(Debug, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// ComfyUI /history response for a prompt
#[derive(Debug, Deserialize)]
pub struct HistoryOutput {
    #[serde(default)]
    pub outputs: std::collections::HashMap<String, NodeOutput>,
}

#[derive(Debug, Deserialize)]
pub struct NodeOutput {
    #[serde(default)]
    pub gifs: Vec<FileOutput>,
    #[serde(default)]
    pub images: Vec<FileOutput>,
}

#[derive(Debug, Deserialize)]
pub struct FileOutput {
    pub filename: String,
    #[serde(default)]
    pub subfolder: String,
    #[serde(rename = "type", default)]
    pub file_type: String,
}

/// ComfyUI /upload/image response
#[derive(Debug, Deserialize, Serialize)]
pub struct ComfyUploadResponse {
    pub name: String,
}
