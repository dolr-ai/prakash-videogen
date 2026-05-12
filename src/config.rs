use anyhow::Result;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub backend_type: String,
    pub auth_token: Option<String>,
    pub sentry_dsn: Option<String>,

    // ComfyUI backend config
    pub comfyui_host: String,
    pub comfyui_port: u16,
    pub comfyui_output_dir: String,

    // Video cleanup config
    pub video_ttl_minutes: u64,
    pub cleanup_check_interval: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // Parse COMFYUI_API_BASE (e.g. "http://localhost:18188") if set,
        // falling back to COMFYUI_HOST + COMFYUI_PORT env vars.
        let (comfyui_host, comfyui_port) = if let Ok(base) = std::env::var("COMFYUI_API_BASE") {
            // Parse "http://host:port" format
            let url = url::Url::parse(&base)?;
            let host = url.host_str().unwrap_or("127.0.0.1").to_string();
            let port = url.port().unwrap_or(18188);
            (host, port)
        } else {
            let host = std::env::var("COMFYUI_HOST").unwrap_or_else(|_| "127.0.0.1".into());
            let port: u16 = std::env::var("COMFYUI_PORT")
                .unwrap_or_else(|_| "18188".into())
                .parse()?;
            (host, port)
        };

        Ok(Self {
            // Default 18288: Vast.ai maps external 8288 -> internal 18288
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "18288".into())
                .parse()?,
            backend_type: std::env::var("BACKEND_TYPE").unwrap_or_else(|_| "comfyui".into()),
            auth_token: std::env::var("AUTH_TOKEN").ok().filter(|s| !s.is_empty()),
            sentry_dsn: std::env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty()),

            comfyui_host,
            comfyui_port,
            comfyui_output_dir: std::env::var("COMFYUI_OUTPUT_DIR")
                .unwrap_or_else(|_| "/workspace/ComfyUI/output".into()),
            video_ttl_minutes: std::env::var("VIDEO_TTL_MINUTES")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            cleanup_check_interval: std::env::var("CLEANUP_CHECK_INTERVAL")
                .unwrap_or_else(|_| "300".into())
                .parse()
                .unwrap_or(300),
        })
    }
}
