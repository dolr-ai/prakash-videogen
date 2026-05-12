# ============================================================================
# Extend the existing dolr-ai ComfyUI image with the Rust worker
# ============================================================================
# This replaces the Python API wrapper with the Rust videogen-worker.
#
# Vast.ai port mapping (configured in template):
#   External 8288 -> Internal 18288 (videogen-worker)
#   External 8188 -> Internal 18188 (ComfyUI)
# ============================================================================

# --- Build stage ---
FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY src/ src/
RUN cargo build --release

# --- Runtime stage: extend the existing ComfyUI image ---
FROM ghcr.io/dolr-ai/comfyui-ltx2:latest

# Copy the Rust binary
COPY --from=builder /app/target/release/videogen-worker /usr/local/bin/videogen-worker
RUN chmod +x /usr/local/bin/videogen-worker

# Install cloudflared
RUN curl -sL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
    -o /usr/local/bin/cloudflared && chmod +x /usr/local/bin/cloudflared

# Install Beszel Agent
RUN curl -sL https://github.com/henrygd/beszel/releases/latest/download/beszel-agent_linux_amd64.tar.gz -o beszel-agent.tar.gz \
    && tar -xzf beszel-agent.tar.gz \
    && mv beszel-agent /usr/local/bin/ \
    && rm beszel-agent.tar.gz

# Default env vars (matching Vast.ai template port scheme)
ENV PORT=18288 \
    BACKEND_TYPE=comfyui \
    COMFYUI_API_BASE=http://localhost:18188 \
    RUST_LOG=info,videogen_worker=debug

EXPOSE 18288
