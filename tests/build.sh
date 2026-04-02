#!/bin/bash

set -e

EMBED_FRONTEND=false
BUILD_LITE=false
BUILD_FULL=false
BUILD_BOTH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --embed-frontend)
            EMBED_FRONTEND=true
            shift
            ;;
        --lite)
            BUILD_LITE=true
            shift
            ;;
        --full)
            BUILD_FULL=true
            shift
            ;;
        --both)
            BUILD_BOTH=true
            shift
            ;;
        -h|--help)
            echo "Usage: build.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --embed-frontend  Embed frontend into binary"
            echo "  --lite            Build only Lite version (SQLite only)"
            echo "  --full            Build only Full version (with Vector DB, embeddings, LLM)"
            echo "  --both            Build both Lite and Full versions (default)"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./build.sh                    # Build both versions (default)"
            echo "  ./build.sh --lite             # Build only Lite version"
            echo "  ./build.sh --full --embed-frontend  # Build Full version with embedded frontend"
            echo "  ./build.sh --both --embed-frontend  # Build both versions with embedded frontend"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Default to building both if no version specified
if [ "$BUILD_LITE" = false ] && [ "$BUILD_FULL" = false ]; then
    BUILD_BOTH=true
fi

echo "Building memos-rs..."

# Build frontend
echo "Building frontend..."
cd /home/praburaja/projects/opencode_ws/memos-rs/frontend
npm install
npm run build
cd /home/praburaja/projects/opencode_ws/memos-rs

# Build Lite version
if [ "$BUILD_LITE" = true ] || [ "$BUILD_BOTH" = true ]; then
    echo "Building Lite version..."
    if [ "$EMBED_FRONTEND" = true ]; then
	echo "Using embedded frontend for lite version"
        cargo build --release --no-default-features --features "lite embed-frontend" --bin memos-rs-lite
    else
        cargo build --release --no-default-features --features "lite" --bin memos-rs-lite
    fi
    echo "Lite version build complete!"
fi

# Build Full version
if [ "$BUILD_FULL" = true ] || [ "$BUILD_BOTH" = true ]; then
    echo "Building Full version..."
    if [ "$EMBED_FRONTEND" = true ]; then
        echo "Embedding frontend into binary..."
        cargo build --release --features "embed-frontend"
    else
        cargo build --release
    fi
    echo "Full version build complete!"
fi

echo "Build complete!"
