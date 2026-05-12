#!/bin/bash
# =============================================================================
# One-time Vast.ai setup: ComfyUI + models + system dependencies
# =============================================================================
# Usage: bash setup.sh
# Run this ONCE when provisioning a new Vast.ai instance.
# =============================================================================

set -euo pipefail

WORKSPACE="/workspace"
COMFYUI_DIR="${WORKSPACE}/ComfyUI"
LOG_DIR="/var/log/comfyui"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[SETUP]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

mkdir -p "$LOG_DIR"

# =============================================================================
# System dependencies
# =============================================================================
log "Installing system dependencies..."
apt-get update -qq
apt-get install -y -qq git wget curl tmux ffmpeg jq > /dev/null 2>&1

# =============================================================================
# cloudflared
# =============================================================================
if ! command -v cloudflared &> /dev/null; then
    log "Installing cloudflared..."
    curl -sL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
        -o /usr/local/bin/cloudflared
    chmod +x /usr/local/bin/cloudflared
fi

# =============================================================================
# Beszel Agent
# =============================================================================
if [ ! -f "/usr/local/bin/beszel-agent" ]; then
    log "Installing Beszel Agent..."
    wget -q --show-progress -O beszel-agent.tar.gz https://github.com/henrygd/beszel/releases/latest/download/beszel-agent_linux_amd64.tar.gz
    tar -xzf beszel-agent.tar.gz
    mv beszel-agent /usr/local/bin/
    rm beszel-agent.tar.gz
fi

# =============================================================================
# ComfyUI
# =============================================================================
if [ -d "$COMFYUI_DIR" ]; then
    log "ComfyUI exists, updating..."
    cd "$COMFYUI_DIR"
    git checkout master || true
    git pull
else
    log "Cloning ComfyUI..."
    cd "$WORKSPACE"
    git clone https://github.com/comfyanonymous/ComfyUI.git
fi

cd "$COMFYUI_DIR"
log "Installing ComfyUI dependencies..."
pip install -r requirements.txt -q

# =============================================================================
# Custom nodes
# =============================================================================
log "Installing custom nodes..."
cd "${COMFYUI_DIR}/custom_nodes"

for repo in \
    "https://github.com/Lightricks/ComfyUI-LTXVideo.git" \
    "https://github.com/kijai/ComfyUI-KJNodes.git" \
    "https://github.com/Kosinkadink/ComfyUI-VideoHelperSuite.git"; do

    dir=$(basename "$repo" .git)
    if [ ! -d "$dir" ]; then
        log "  -> $dir"
        git clone "$repo"
        cd "$dir"
        [ -f requirements.txt ] && pip install -r requirements.txt -q 2>/dev/null || true
        cd ..
    else
        log "  -> $dir (exists)"
    fi
done

pip install sageattention -q 2>/dev/null || warn "SageAttention failed"

# =============================================================================
# Model weights
# =============================================================================
log "Downloading model weights..."

CKPT="${COMFYUI_DIR}/models/checkpoints"
mkdir -p "$CKPT"

# LTX-2 19B Distilled
if [ ! -s "${CKPT}/ltx-2-19b-distilled.safetensors" ]; then
    rm -f "${CKPT}/ltx-2-19b-distilled.safetensors"
    log "  -> LTX-2 19B Distilled (~38GB)..."
    wget -q --show-progress --header="Authorization: Bearer ${HF_TOKEN:-}" -O "${CKPT}/ltx-2-19b-distilled.safetensors" \
        "https://huggingface.co/Lightricks/LTX-2/resolve/main/ltx-2-19b-distilled.safetensors"
fi

# Gemma 3 12B
CLIP_DIR="${COMFYUI_DIR}/models/clip"
mkdir -p "$CLIP_DIR"
GEMMA="${CLIP_DIR}/gemma-3-12b-it-qat-q4_0-unquantized"
if [ ! -d "$GEMMA" ] || [ -z "$(ls -A "$GEMMA")" ]; then
    log "  -> Gemma 3 12B text encoder..."
    mkdir -p "$GEMMA"
    for i in $(seq -w 1 5); do
        file="${GEMMA}/model-0000${i}-of-00005.safetensors"
        if [ ! -s "$file" ]; then
            rm -f "$file"
            wget -q --show-progress --header="Authorization: Bearer ${HF_TOKEN:-}" -O "$file" \
                "https://huggingface.co/google/gemma-3-12b-it-qat-q4_0-unquantized/resolve/main/model-0000${i}-of-00005.safetensors" || true
        fi
    done
fi

log "  -> Gemma 3 config files..."
for f in tokenizer.model tokenizer_config.json config.json model.safetensors.index.json special_tokens_map.json tokenizer.json preprocessor_config.json generation_config.json; do
    if [ ! -s "${GEMMA}/${f}" ]; then
        wget -q --show-progress --header="Authorization: Bearer ${HF_TOKEN:-}" -O "${GEMMA}/${f}" \
            "https://huggingface.co/google/gemma-3-12b-it-qat-q4_0-unquantized/resolve/main/${f}" || true
    fi
done

# Spatial upscaler
UP="${COMFYUI_DIR}/models/latent_upscale_models"
mkdir -p "$UP"
if [ ! -s "${UP}/ltx-2-spatial-upscaler-x2-1.0.safetensors" ]; then
    rm -f "${UP}/ltx-2-spatial-upscaler-x2-1.0.safetensors"
    log "  -> Spatial Upscaler 2x..."
    wget -q --show-progress --header="Authorization: Bearer ${HF_TOKEN:-}" -O "${UP}/ltx-2-spatial-upscaler-x2-1.0.safetensors" \
        "https://huggingface.co/Lightricks/LTX-2/resolve/main/ltx-2-spatial-upscaler-x2-1.0.safetensors" || true
fi

# =============================================================================
# Restart ComfyUI to load new custom nodes
# =============================================================================
log "Restarting ComfyUI to apply custom nodes..."
supervisorctl restart comfyui || warn "Failed to restart ComfyUI, you may need to restart it manually."

# =============================================================================
# Done
# =============================================================================
echo ""
log "Setup complete! Run 'bash /workspace/start.sh' to start services."
