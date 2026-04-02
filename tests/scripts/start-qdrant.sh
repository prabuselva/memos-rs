#!/bin/bash

echo "=== Memos RS - Start Qdrant Vector Database ==="
echo ""

# Check if Docker is running
if ! command -v docker &> /dev/null; then
    echo "Docker not found. Please install Docker first."
    echo "https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Qdrant is already running
if docker ps | grep -q qdrant; then
    echo "Qdrant is already running!"
    docker ps | grep qdrant
    exit 0
fi

echo "Starting Qdrant vector database..."
echo ""

# Create network if it doesn't exist
docker network create memos-network 2>/dev/null || true

# Start Qdrant
docker run -d \
    --name qdrant \
    --network memos-network \
    -p 6333:6333 \
    -p 6334:6334 \
    -v $(pwd)/qdrant_storage:/qdrant/storage \
    qdrant/qdrant

echo ""
echo "=== Qdrant Started ==="
echo ""
echo "Vector database URL: http://localhost:6333"
echo "Status: docker ps | grep qdrant"
echo "Logs: docker logs qdrant"
echo "Stop: docker stop qdrant"
echo ""
