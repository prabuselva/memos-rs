# Qdrant HTTP API Test

curl -s -X PUT "http://localhost:6333/collections/test_vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    }
  }'

echo ""
echo "Checking collection..."
curl -s "http://localhost:6333/collections/test_vectors" | jq '.result.config.params'

echo ""
echo "Upserting a point with vector..."
curl -s -X PUT "http://localhost:6333/collections/test_vectors/points" \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "1",
        "vector": [0.1, 0.2, 0.3],
        "payload": {"test": "value"}
      }
    ]
  }'

echo ""
echo "Checking points..."
curl -s "http://localhost:6333/collections/test_vectors/points?limit=1" | jq '.'

echo ""
echo "Cleaning up..."
curl -s -X DELETE "http://localhost:6333/collections/test_vectors"
