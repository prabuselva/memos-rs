#!/bin/bash

BASE_URL="http://localhost:3000/api"

# Register a new user
echo "Registering user..."
RESPONSE=$(curl -s -X POST "$BASE_URL/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"vectortest","email":"vector@test.com","password":"vector123"}')

echo "Response: $RESPONSE"

# Parse token from response
TOKEN=$(echo "$RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Failed to get token, trying login instead..."
    RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
      -H "Content-Type: application/json" \
      -d '{"username":"vectortest","password":"vector123"}')
    TOKEN=$(echo "$RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
fi

if [ -z "$TOKEN" ]; then
    echo "Failed to authenticate"
    exit 1
fi

echo "Token: $TOKEN"

# Create a note
echo ""
echo "Creating notes..."
curl -s -X POST "$BASE_URL/notes" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"My First Note","content":"This is a test note about Rust programming","tags":["rust","programming"]}'

echo ""

curl -s -X POST "$BASE_URL/notes" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Python Guide","content":"Learning Python for data science and machine learning","tags":["python","data-science"]}'

echo ""

curl -s -X POST "$BASE_URL/notes" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Web Development","content":"Building web applications with JavaScript and Node.js","tags":["javascript","web"]}'

echo ""

# List notes
echo ""
echo "Listing notes..."
curl -s "$BASE_URL/notes" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Test SQL search
echo ""
echo "Testing SQL search..."
curl -s "$BASE_URL/notes/search?q=rust" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Test vector search
echo ""
echo "Testing vector search..."
curl -s "$BASE_URL/notes/vector-search?q=rust+programming&limit=5" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Test vector search for python
echo ""
echo "Testing vector search for Python..."
curl -s "$BASE_URL/notes/vector-search?q=python+machine+learning&limit=5" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Check Qdrant collection
echo ""
echo "Checking Qdrant collection..."
curl -s "http://localhost:6333/collections/notes" | jq .

echo ""
echo "Done!"
