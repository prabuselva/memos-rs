# memos-rs Plan

## Recent Updates (March 2026)

### Import/Export UI Changes

**Date**: March 3, 2026

**Changes**:
- Removed Export button from the frontend
- Removed "Import Paste" (Import from Text) button from the frontend
- Updated Import button to support both single files and directory imports
- Changed import functionality to upload files recursively from browser

**Frontend Implementation**:
- Single import button: "Import File(s) (including Directory)"
- Uses `<input type="file" webkitdirectory>` for directory selection
- Reads all files recursively from selected directory
- Uploads each `.note` and `.xml` file as XML content to backend
- Backend endpoint `/api/import/tomboy` accepts JSON array of XML notes

**Backend Implementation**:
- `/api/import/tomboy` - Accepts JSON with `notes: string[]` (XML content)
- `/api/import/tomboy/file` - Multipart file upload endpoint
- `/api/import/tomboy/directory` - Server-side directory import (for local filesystem)
- Improved Tomboy XML parser for Title and Body extraction

**Files Modified**:
- `frontend/src/components/NoteList.tsx` - Updated import UI and logic
- `frontend/src/lib/api.ts` - Simplified importExportApi
- `src/api/routes.rs` - Added new import endpoints
- `src/import_export/tomboy/importer.rs` - Added recursive directory support

---

## Overview

A Joplin-like note-taking application built in Rust with the following features:
- Markdown-based note storage
- Web interface accessible via browser (HTTP API)
- SQLite/PostgreSQL database support
- Import/export of notes as Markdown
- Support for importing Tomboy Notes and Gnome GNotes formats
- Vector database integration (Weaviate) for AI LLM search
- Local embedding models (BERT) for semantic search
- Cross-platform desktop app using Tauri

## Technical Stack

- **Language**: Rust
- **Web Framework**: Axum (HTTP API)
- **Database**: SQLite/PostgreSQL (SQLX)
- **Vector Database**: Weaviate
- **Embeddings**: ONNX Runtime (ORT) with local BERT models
- **Markdown Processing**: `comrak` or `pulldown-cmark`
- **Desktop Framework**: Tauri
- **CLI**: `clap` (for desktop mode)

## Project Structure

```
memos-rs/
├── Cargo.toml
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Configuration system
│   ├── db/
│   │   ├── mod.rs           # Database layer
│   │   └── schema.rs        # Database schema
│   ├── models.rs            # Data models
│   ├── repository.rs        # Repository pattern implementations
│   ├── markdown.rs          # Markdown processing utilities
│   ├── import_export.rs     # Import/Export functionality
│   ├── api.rs               # HTTP API routes
│   ├── server.rs            # Server setup
│   ├── state.rs             # Application state
│   ├── vector/
│   │   ├── mod.rs           # Vector database integration
│   │   └── weaviate.rs      # Weaviate client wrapper
│   ├── embeddings/
│   │   ├── mod.rs           # Embedding models
│   │   └── ort.rs           # ONNX Runtime integration
│   └── desktop/
│       ├── mod.rs           # Tauri integration
│       └── tray.rs          # System tray
├── migrations/              # Database migrations
├── assets/                  # Static assets
├── examples/                # Example data and scripts
│   ├── tomboy/              # Tomboy XML examples
│   └── gnote/               # Gnote XML examples
├── tests/                   # Integration tests
├── docs/                    # Documentation
├── docker/                  # Docker configuration
└── frontend/                # React frontend (optional)
```

## Data Models

### Note
- `id`: UUID
- `title`: String
- `content`: String (Markdown)
- `content_html`: String (Rendered HTML)
- `created_at`: DateTime<Utc>
- `updated_at`: DateTime<Utc>
- `notebook_id`: Option<UUID>
- `is_favorite`: bool
- `is_archived`: bool
- `tags`: Vec<String>
- `metadata`: serde_json::Value (for extensibility)

### Notebook
- `id`: UUID
- `name`: String
- `parent_id`: Option<UUID>
- `created_at`: DateTime<Utc>
- `updated_at`: DateTime<Utc>

### Tag
- `id`: UUID
- `name`: String
- `created_at`: DateTime<Utc>

## Database Schema

### SQLite/PostgreSQL Tables

```sql
-- Notes table
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    content_html TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    notebook_id TEXT,
    is_favorite BOOLEAN DEFAULT FALSE,
    is_archived BOOLEAN DEFAULT FALSE,
    metadata TEXT
);

-- Notebooks table
CREATE TABLE notebooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES notebooks(id)
);

-- Tags table
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL
);

-- Note tags junction table
CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_notes_title ON notes(title);
CREATE INDEX idx_notes_created_at ON notes(created_at);
CREATE INDEX idx_notes_updated_at ON notes(updated_at);
CREATE INDEX idx_notes_notebook_id ON notes(notebook_id);
CREATE INDEX idx_tags_name ON tags(name);
```

## API Endpoints

### Notes
- `GET /api/notes` - List all notes
- `GET /api/notes/{id}` - Get note by ID
- `POST /api/notes` - Create new note
- `PUT /api/notes/{id}` - Update note
- `DELETE /api/notes/{id}` - Delete note
- `GET /api/notes/search` - Search notes (text + vector)
- `GET /api/notes/{id}/content` - Get note content (rendered)
- `POST /api/notes/{id}/favorite` - Toggle favorite
- `POST /api/notes/{id}/archive` - Toggle archive

### Notebooks
- `GET /api/notebooks` - List all notebooks
- `GET /api/notebooks/{id}` - Get notebook by ID
- `POST /api/notebooks` - Create notebook
- `PUT /api/notebooks/{id}` - Update notebook
- `DELETE /api/notebooks/{id}` - Delete notebook

### Tags
- `GET /api/tags` - List all tags
- `GET /api/tags/{id}` - Get tag by ID
- `POST /api/tags` - Create tag
- `PUT /api/tags/{id}` - Update tag
- `DELETE /api/tags/{id}` - Delete tag

### Import/Export
- `POST /api/import/tomboy` - Import Tomboy notes (JSON array of XML)
- `POST /api/import/tomboy/file` - Import Tomboy notes (multipart file upload)
- `POST /api/import/tomboy/directory` - Import Tomboy notes from server directory
- `POST /api/import/gnote` - Import Gnote notes
- `GET /api/export/markdown/{id}` - Export note as Markdown
- `GET /api/export/zip` - Export all notes as ZIP

### Vector Search
- `GET /api/vector/search` - Search using vector embeddings
- `POST /api/vector/index` - Rebuild vector index

### System
- `GET /api/health` - Health check
- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration

## Import/Export Formats

### Markdown (Default)
- Single file per note
- Frontmatter for metadata (YAML format)
- Example:
```markdown
---
title: My Note
created_at: 2024-01-01T00:00:00Z
tags: [tag1, tag2]
notebook: My Notebook
---

# Content
```

### Tomboy XML
- Parse Tomboy XML format
- Extract title, content, tags, creation/update times
- Support nested note structures
- **Recent Update**: Now supports importing multiple files recursively from directory
- **Recent Update**: Frontend uploads files one-by-one for remote directory support

### Gnote XML
- Parse Gnote XML format (similar to Tomboy)
- Handle Gnote-specific metadata

## Vector Database Integration

### Weaviate Schema
```json
{
  "class": "Note",
  "vectorIndexConfig": {
    "distance": "cosine"
  },
  "properties": [
    {
      "name": "title",
      "dataType": ["text"]
    },
    {
      "name": "content",
      "dataType": ["text"]
    },
    {
      "name": "notebook",
      "dataType": ["text"]
    },
    {
      "name": "tags",
      "dataType": ["text[]"]
    }
  ]
}
```

### Embedding Process
1. Extract text from note (title + content)
2. Split text into chunks if needed
3. Generate embeddings using local BERT model
4. Store in Weaviate with note reference

## Local Embedding Models

### Model Selection
- **BERT** (base or mini) for local embeddings
- ONNX Runtime for model execution
- Model size: ~100-500MB

### Integration
1. Load ONNX model on startup
2. Preprocess text (tokenization)
3. Generate embeddings
4. Return vector representation

### Tokenization
- Use `tokenizers` crate
- Load pre-trained tokenizer (e.g., `bert-base-uncased`)
- Handle padding, truncation, attention masks

## CLI Commands

```bash
# Start server
memos-rs server --port 8080 --host 0.0.0.0

# Import notes
memos-rs import --file notes.xml --format tomboy

# Export notes
memos-rs export --format markdown --output ./export/

# Initialize database
memos-rs init --db sqlite --path ./data/memos.db

# Vector index operations
memos-rs vector --rebuild

# Desktop mode
memos-rs desktop
```

## Desktop Application (Tauri)

### Features
- System tray with quick actions
- Native notifications
- Auto-update
- Local file storage
- Keyboard shortcuts
- Dark/light theme

### Tray Menu
- Open app
- New note
- Search
- Preferences
- Quit

## Configuration

```rust
struct Config {
    database: DatabaseConfig,
    server: ServerConfig,
    vector: VectorConfig,
    embeddings: EmbeddingConfig,
}

struct DatabaseConfig {
    kind: DatabaseKind, // SQLite or PostgreSQL
    path: Option<String>,
    connection_string: Option<String>,
}

struct ServerConfig {
    host: String,
    port: u16,
    enable_cors: bool,
}

struct VectorConfig {
    enabled: bool,
    endpoint: String,
    api_key: Option<String>,
}

struct EmbeddingConfig {
    model_path: String,
    device: Device, // CPU or GPU
    batch_size: usize,
}
```

## Implementation Phases

### Phase 1: Core Foundation
- [ ] Project structure setup
- [ ] Database layer (SQLite)
- [ ] Data models
- [ ] Configuration system
- [ ] Basic repository pattern

### Phase 2: API Development
- [ ] REST API endpoints
- [ ] Markdown processing
- [ ] Error handling
- [ ] Validation

### Phase 3: Import/Export
- [x] Markdown import/export
- [x] Tomboy XML parsing (Title and Body extraction)
- [x] Gnote XML parsing
- [x] ZIP packaging
- [x] Recursive directory import support
- [x] Frontend file upload for remote directory support

### Phase 4: Vector Search
- [ ] Weaviate integration
- [ ] Embedding generation
- [ ] ONNX Runtime setup
- [ ] BERT model loading

### Phase 5: Desktop App
- [ ] Tauri setup
- [ ] React frontend
- [ ] System tray
- [ ] Native features

### Phase 6: Testing & Documentation
- [ ] Unit tests
- [ ] Integration tests
- [ ] Documentation
- [ ] Docker configuration

## Dependencies (Cargo.toml)

```toml
[dependencies]
# Web framework
actix-web = "4"
actix-rt = "2"

# Database
sqlx = { version = "0.7", features = ["sqlite", "postgres", "chrono", "json"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Markdown
comrak = "0.18"

# CLI
clap = { version = "4", features = ["derive"] }

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Configuration
config = "0.14"

# Error handling
thiserror = "1"

# Async
async-trait = "0.1"

# Vector database
weaviate-client = "0.5"

# Embeddings
ort = "2"
tokenizers = "0.15"
```

## Build & Deployment

### Build Process (Recommended for All Builds)

**Always build the project using the following process:**

1. Start the build in the background with output to a log file
2. Check periodically every 60 seconds for completion
3. Use `wait` to monitor background process and check exit status
4. Only if the Build is completed and then proceed to run the Application

**Build Command Template:**
```bash
# Start build in background
cargo build --release 2>&1 | tee /home/praburaja/projects/opencode_ws/memos-rs/build_output.log &
BUILD_PID=$!

# Periodically check build status
while kill -0 $BUILD_PID 2>/dev/null; do
    echo "Build still running... PID: $BUILD_PID"
    sleep 180
done

# Wait for final completion and check exit status
wait $BUILD_PID
BUILD_STATUS=$?
if [ $BUILD_STATUS -eq 0 ]; then
    echo "Build completed successfully"
else
    echo "Build failed with exit status: $BUILD_STATUS"
fi
```

**Alternative: Simple Background Build**
```bash
cargo build --release > /home/praburaja/projects/opencode_ws/memos-rs/build_output.log 2>&1 &
BUILD_PID=$!
echo "Build started with PID: $BUILD_PID"

# Wait for completion (with periodic checks)
while kill -0 $BUILD_PID 2>/dev/null; do
    echo "Build in progress... (check every 180s)"
    sleep 180
done

# Check result
if [ -f /home/praburaja/projects/opencode_ws/memos-rs/build_output.log ]; then
    echo "Build log available at: build_output.log"
fi
```

### Using build.sh (Recommended)
```bash
bash build.sh
```

This script:
1. Installs frontend dependencies
2. Builds the React frontend with Vite
3. Compiles Rust backend in release mode
4. Outputs to `dist/` directory

### Manual Build
```bash
# Frontend
cd frontend
npm install
npm run build

# Rust Backend
cargo build --release
```

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/memos-rs /usr/local/bin/
EXPOSE 8080
CMD ["memos-rs", "server"]
```

### System Requirements
- Rust 1.75+
- SQLite 3.35+ or PostgreSQL 12+
- 512MB RAM minimum
- 100MB disk space for BERT model

## Future Enhancements
- Sync with cloud backend
- Collaborative editing
- Mobile app (iOS/Android)
- Browser extension
- Plugin system
- Custom themes
- Keyboard shortcuts customization
- Advanced search filters
- Backup/restore functionality

# Run the Application Guidelines

Follow these Rules when running the application after building

1. Always execute the binary in the background and capture its PID status
2. Sleep for 3 secs and then check for PID Status
3. Finally perform curl health check first
4. Later do all API tests for Notes CRUD operation
5. If there's any issue in PID status check due to application crash, Delete the sqlite file and start from running freshly again from (1).


