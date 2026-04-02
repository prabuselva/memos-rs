#!/bin/bash
set -e

echo "=== Memos RS - Full Build ==="
echo ""

# Check if models exist, if not prompt to download
if [ ! -f "models/model.onnx" ]; then
    echo "Warning: Model files not found in models/"
    read -p "Download model files now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        bash scripts/setup-embeddings.sh
    fi
fi

echo ""
echo "Building frontend..."
cd frontend
npm install
npm run build
cd ..

echo ""
echo "Building Rust backend..."
cargo build --release

echo ""
echo "=== Build Complete ==="
echo ""
echo "Binary location: target/release/memos-rs"
echo ""
