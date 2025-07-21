#!/bin/bash

# Aircher Model Download Script
# Downloads required embedding models for semantic search

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MODELS_DIR="$PROJECT_ROOT/models"

echo "🤖 Aircher Model Downloader"
echo "========================="
echo ""

# Check if model already exists
if [ -f "$MODELS_DIR/swerank-embed-small.safetensors" ]; then
    echo "✅ Model already exists: swerank-embed-small.safetensors"
    echo "   Size: $(ls -lh "$MODELS_DIR/swerank-embed-small.safetensors" | awk '{print $5}')"
    echo ""
    echo "To re-download, delete the existing file first."
    exit 0
fi

echo "📥 Downloading SweRankEmbed-Small model (260.81 MB)..."
echo ""

# Create models directory if it doesn't exist
mkdir -p "$MODELS_DIR"

# Download from HuggingFace or another source
# NOTE: Update this URL with the actual model location
MODEL_URL="https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors"
echo "⚠️  TODO: Update MODEL_URL with actual SweRankEmbed model location"
echo ""

# Placeholder download (replace with actual download)
echo "Would download from: $MODEL_URL"
echo "To: $MODELS_DIR/swerank-embed-small.safetensors"
echo ""

# Actual download command (uncomment when URL is available):
# curl -L "$MODEL_URL" -o "$MODELS_DIR/swerank-embed-small.safetensors"
# or
# wget "$MODEL_URL" -O "$MODELS_DIR/swerank-embed-small.safetensors"

echo "❌ Download not implemented - manual download required"
echo ""
echo "Please manually download the model and place it at:"
echo "   $MODELS_DIR/swerank-embed-small.safetensors"
echo ""
echo "See models/README.md for more information."

exit 1