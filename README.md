# videogen-worker

Video generation worker with pluggable backend adapters. Runs alongside GPU inference servers (e.g. ComfyUI on Vast.ai) and exposes a REST API for the [off-chain-agent](https://github.com/dolr-ai/off-chain-agent).

## Architecture

```
off-chain-agent (baremetal)
       │
       │  HTTPS (static URL, never changes)
       ▼
┌─────────────────────────────────────────────┐
│  comfyui.prakash.yral.com                   │
│  (Cloudflare Named Tunnel)                  │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│  Vast.ai GPU Instance (H100)               │
│                                             │
│  ┌─────────────────────────────┐            │
│  │  videogen-worker (:8288)    │            │
│  │  ├── POST /generate         │            │
│  │  ├── POST /upload/image     │            │
│  │  ├── GET  /view             │            │
│  │  ├── GET  /health           │            │
│  │  └── GET  /swagger-ui       │            │
│  └────────────┬────────────────┘            │
│               │ localhost                   │
│  ┌────────────▼────────────────┐            │
│  │  ComfyUI (:8188)            │            │
│  │  + LTX-2 19B Distilled      │            │
│  │  + Gemma 3 12B              │            │
│  │  + Spatial Upscaler 2x      │            │
│  └─────────────────────────────┘            │
└─────────────────────────────────────────────┘
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/generate` | Submit a video generation job |
| `GET` | `/result/{id}` | Check job status |
| `POST` | `/upload/image` | Upload an image (multipart) |
| `GET` | `/view` | Download output file |
| `GET` | `/health` | Backend health check |
| `GET` | `/swagger-ui` | Interactive API documentation |

## Backend Adapters

The worker uses an adapter pattern (`VideoGenBackend` trait). Currently supported:

- **`comfyui`** — Proxies to a local ComfyUI instance via HTTP + WebSocket

Future adapters can be added for LTX hosted API, RunPod, etc.

## Quick Start

### Local development

```bash
# Start ComfyUI on port 8188 first, then:
COMFYUI_HOST=127.0.0.1 COMFYUI_PORT=8188 cargo run
# Visit http://localhost:8288/swagger-ui
```

### Deploy to Vast.ai

#### One-time setup (new instance)

```bash
# SSH into the instance
ssh -p <PORT> root@<IP>

# Copy and run setup script
bash /workspace/deploy/setup.sh
```

#### Via GitHub Actions (recommended)

1. **Create Cloudflare tunnel** (one-time):
   - Cloudflare Zero Trust → Networks → Tunnels → Create
   - Name: `comfyui-worker`
   - Public hostname: `comfyui.prakash.yral.com` → `http://localhost:8288`
   - Copy the tunnel token

2. **Set GitHub secrets**:

   | Secret | Description |
   |--------|-------------|
   | `VASTAI_SSH_KEY` | SSH private key for Vast.ai instance |
   | `VASTAI_HOST` | Instance IP address |
   | `VASTAI_PORT` | Instance SSH port |
   | `AUTH_TOKEN` | Bearer token for API auth |
   | `CF_TUNNEL_TOKEN` | Cloudflare tunnel token |
   | `SENTRY_DSN` | Sentry DSN (optional) |

3. **Push to `main`** — the deploy workflow builds and deploys automatically.

#### Manual deploy

```bash
cargo build --release
scp target/release/videogen-worker root@<IP>:/workspace/videogen-worker
scp deploy/start.sh root@<IP>:/workspace/start.sh
ssh -p <PORT> root@<IP> "AUTH_TOKEN=xxx CF_TUNNEL_TOKEN=yyy bash /workspace/start.sh"
```

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `PORT` | `8288` | Worker listen port |
| `BACKEND_TYPE` | `comfyui` | Backend adapter to use |
| `AUTH_TOKEN` | *(none)* | Bearer token (disabled if empty) |
| `COMFYUI_HOST` | `127.0.0.1` | ComfyUI hostname |
| `COMFYUI_PORT` | `8188` | ComfyUI port |
| `SENTRY_DSN` | *(none)* | Sentry error reporting |
| `CF_TUNNEL_TOKEN` | *(none)* | Cloudflare named tunnel token |

## off-chain-agent Integration

Once deployed, set these static env vars on the off-chain-agent (they never change):

```env
COMFYUI_API_URL=https://comfyui.prakash.yral.com
COMFYUI_VIEW_URL=https://comfyui.prakash.yral.com
COMFYUI_API_TOKEN=<your-auth-token>
```
