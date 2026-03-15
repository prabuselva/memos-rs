#!/bin/bash

set -e

EMBED_FRONTEND=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --embed-frontend)
            EMBED_FRONTEND=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Building memos-rs..."

# Build frontend
echo "Building frontend..."
cd /home/praburaja/projects/opencode_ws/memos-rs/frontend
npm install
npm run build
cd /home/praburaja/projects/opencode_ws/memos-rs

# Build Rust backend
echo "Building Rust backend..."
if [ "$EMBED_FRONTEND" = true ]; then
    echo "Embedding frontend into binary..."
    cargo build --release --features embed-frontend
else
    cargo build --release
fi

echo "Build complete!"
