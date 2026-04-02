#!/bin/bash
# Download BERT model for embeddings

MODEL_DIR=".memos-rs/models"
mkdir -p "$MODEL_DIR/all-MiniLM-L6-v2"

echo "Downloading all-MiniLM-L6-v2 tokenizer..."
cd "$MODEL_DIR/all-MiniLM-L6-v2"

# Download tokenizer.json
if command -v curl &> /dev/null; then
    curl -L -o tokenizer.json \
        https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json
elif command -v wget &> /dev/null; then
    wget -O tokenizer.json \
        https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json
fi

# Check if download succeeded
if [ -f "tokenizer.json" ]; then
    echo "Tokenizer downloaded successfully!"
    echo "Model location: $MODEL_DIR/all-MiniLM-L6-v2/tokenizer.json"
else
    echo "Failed to download tokenizer. Please download manually:"
    echo "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json"
fi

MODEL_DIR="models"
MODEL_NAME="all-MiniLM-L6-v2"
HUGGINGFACE_REPO="Xenova/${MODEL_NAME}"

echo "Creating model directory: ${MODEL_DIR}"
mkdir -p "${MODEL_DIR}"

echo ""
echo "Downloading ONNX model files..."
echo "Repository: ${HUGGINGFACE_REPO}"
echo ""

# Download model.onnx
echo "Downloading model.onnx..."
wget -q "https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/onnx/model.onnx" -O "${MODEL_DIR}/model.onnx" || {
    echo "Warning: Could not download model.onnx from HuggingFace"
    echo "You can download it manually from: https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/onnx/model.onnx"
}

# Download config.json
echo "Downloading config.json..."
wget -q "https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/onnx/config.json" -O "${MODEL_DIR}/config.json" || {
    echo "Warning: Could not download config.json from HuggingFace"
    echo "You can download it manually from: https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/onnx/config.json"
}

# Download tokenizer.json
echo "Downloading tokenizer.json..."
wget -q "https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/tokenizer.json" -O "${MODEL_DIR}/tokenizer.json" || {
    echo "Warning: Could not download tokenizer.json from HuggingFace"
    echo "You can download it manually from: https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/tokenizer.json"
}

# Download tokenizer_config.json
echo "Downloading tokenizer_config.json..."
wget -q "https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/tokenizer_config.json" -O "${MODEL_DIR}/tokenizer_config.json" || {
    echo "Warning: Could not download tokenizer_config.json from HuggingFace"
    echo "You can download it manually from: https://huggingface.co/${HUGGINGFACE_REPO}/resolve/main/tokenizer_config.json"
}

echo ""
echo "=== Download Complete ==="
echo ""
echo "Model files saved to: ${MODEL_DIR}/"
echo ""

if [ -f "${MODEL_DIR}/model.onnx" ]; then
    MODEL_SIZE=$(du -h "${MODEL_DIR}/model.onnx" | cut -f1)
    echo "✓ model.onnx (${MODEL_SIZE})"
else
    echo "✗ model.onnx (missing)"
fi

if [ -f "${MODEL_DIR}/config.json" ]; then
    echo "✓ config.json"
else
    echo "✗ config.json (missing)"
fi

if [ -f "${MODEL_DIR}/tokenizer.json" ]; then
    echo "✓ tokenizer.json"
else
    echo "✗ tokenizer.json (missing)"
fi

if [ -f "${MODEL_DIR}/tokenizer_config.json" ]; then
    echo "✓ tokenizer_config.json"
else
    echo "✗ tokenizer_config.json (missing)"
fi

echo ""
echo "=== Next Steps ==="
echo ""
echo "1. Start Qdrant vector database:"
echo "   docker run -p 6333:6333 qdrant/qdrant"
echo ""
echo "2. Build and run the application:"
echo "   cargo build --release"
echo "   cargo run --release"
echo ""
echo "3. Or use the build script:"
echo "   bash build.sh"
echo ""
