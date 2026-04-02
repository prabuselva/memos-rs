use crate::import_export::tomboy::{TomboyExporter, TomboyImporter, TomboyNote};
use crate::models::auth_dto::{
    ErrorResponse, GetLLMSettingsResponse, LoginRequest, LoginResponse, PasswordResetConfirm,
    PasswordResetRequest, RegisterRequest, RegisterResult, UpdateProfileRequest, ValidationError,
};
use crate::models::{AuthError, Notebook, NotebookNode, UserProfile};
use crate::services::auth_service::AuthService;
use crate::{models::Note, AppState, VERSION, VERSION_SHORT};
use serde::Deserialize;

fn serialize_notebook_tree(nodes: &[NotebookNode]) -> serde_json::Value {
    serde_json::json!({
        "folders": nodes.iter().map(serialize_notebook_node).collect::<Vec<_>>()
    })
}

fn serialize_notebook_node(node: &NotebookNode) -> serde_json::Value {
    serde_json::json!({
        "id": node.notebook.id,
        "name": node.notebook.name,
        "parent_id": node.notebook.parent_id,
        "children": node.children.iter().map(serialize_notebook_node).collect::<Vec<_>>(),
        "notes": node.notes.iter().map(|n| {
            serde_json::json!({
                "id": n.id,
                "title": n.title,
                "content": n.content,
                "content_html": n.content_html,
                "notebook_id": n.notebook_id,
                "parent_id": n.parent_id,
                "created_at": n.created_at.to_rfc3339(),
                "updated_at": n.updated_at.to_rfc3339(),
                "is_favorite": n.is_favorite,
                "is_archived": n.is_archived,
                "tags": n.tags,
                "metadata": n.metadata,
                "user_id": n.user_id
            })
        }).collect::<Vec<_>>()
    })
}

use axum::{
    extract::{FromRequestParts, Json, Multipart, Path, Query, State},
    http::{request::Parts, StatusCode},
    routing::{get, post, put},
    Router,
};
use pulldown_cmark::{html, Options, Parser};
use std::env;
use std::sync::Arc;
use tempfile::TempDir;
use tracing::{debug, error};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct CurrentUser(pub UserProfile);

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for CurrentUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth_str) = auth_header {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.strip_prefix("Bearer ").unwrap();
                let auth_service = AuthService::new();

                if let Ok(user) = auth_service.validate_session(state.db.clone(), token).await {
                    return Ok(CurrentUser(user));
                }
            }
        }

        Err((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ))
    }
}

pub fn create_router() -> Router<Arc<AppState>> {
    let notes_router = Router::new()
        .route("/", get(get_note).put(update_note).delete(delete_note))
        .route("/content", get(get_note_content));

    let notebooks_router = Router::new()
        .route("/", get(list_notebooks).post(create_notebook))
        .route("/tree", get(get_notebooks_tree))
        .route("/root-contents", get(get_root_contents))
        .route(
            "/:id",
            get(get_notebook)
                .put(update_notebook)
                .delete(delete_notebook),
        )
        .route("/:id/notes", get(get_notes_by_notebook))
        .route("/:id/contents", get(get_folder_contents))
        .route("/reorder", put(reorder_notebooks));

    Router::new()
        .nest("/notebooks", notebooks_router)
        .route("/notes/reorder", put(reorder_notes))
        .route("/bulk-delete", post(bulk_delete))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .route("/profile", put(update_profile))
        .route("/profile/search-mode", put(update_search_mode))
        .route("/profile/search-mode", get(get_search_mode))
        .route("/profile/llm-settings", put(update_llm_settings))
        .route("/profile/llm-settings", get(get_llm_settings))
        .route("/profile/llm-settings/test", post(test_llm_connection))
        .route("/request-password-reset", post(request_password_reset))
        .route("/reset-password", post(reset_password))
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/search", get(search_notes))
        .route("/notes/vector-search", get(vector_search_notes))
        .nest("/notes/:id", notes_router)
        .route("/import/tomboy", post(import_tomboy))
        .route("/import/tomboy/file", post(import_tomboy_file))
        .route("/import/tomboy/xml", post(import_tomboy_xml))
        .route("/import/tomboy/directory", post(import_tomboy_directory))
        .route("/export/tomboy", get(export_tomboy))
        .route("/version", get(version_info))
        .route("/health", get(health))
        .route("/vector-db/status", get(get_vector_db_status))
}

#[derive(Deserialize)]
struct ListNotesQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

async fn list_notes(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNotesQuery>,
) -> (StatusCode, Json<Vec<Note>>) {
    let db = state.db.lock().await;
    match db
        .get_notes_by_user(&current_user.id, query.limit, query.offset)
        .await
    {
        Ok(notes) => (StatusCode::OK, Json(notes)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_notes(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> (StatusCode, Json<Vec<Note>>) {
    let db = state.db.lock().await;
    match db
        .search_notes_by_user(&current_user.id, &query.q, 10)
        .await
    {
        Ok(notes) => (StatusCode::OK, Json(notes)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[derive(Deserialize)]
struct VectorSearchQuery {
    q: String,
    limit: Option<u32>,
}

async fn vector_search_notes(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<VectorSearchQuery>,
) -> (StatusCode, Json<Vec<Note>>) {
    let db = state.db.lock().await;

    let vector_store = match &db.vector_store {
        Some(vs) => vs,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![])),
    };

    let mut embedding = match vector_search_notes::generate_embedding(&query.q) {
        Ok(e) => e,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    };

    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();

    let limit = query.limit.unwrap_or(10);

    match vector_store
        .search_notes(embedding, &current_user.id, limit)
        .await
    {
        Ok(results) => {
            let mut notes = Vec::new();
            for result in results {
                if let serde_json::Value::Object(ref map) = result {
                    if let Some(id_value) = map.get("id") {
                        if let Some(note_id) = id_value.as_str() {
                            if let Ok(note) = db.get_by_id(note_id).await {
                                notes.push(note);
                            }
                        }
                    }
                }
            }
            (StatusCode::OK, Json(notes))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

mod vector_search_notes {
    use crate::config::Config;
    use crate::vector::EmbeddingModel;

    pub fn generate_embedding(text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let config = Config::default();
        let model = EmbeddingModel::load(&config)?;
        let mut embedding = model.embed(text)?;
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();
        Ok(embedding)
    }
}

#[derive(Deserialize)]
struct CreateNoteRequest {
    title: String,
    content: String,
    notebook_id: Option<String>,
    parent_id: Option<String>,
    tags: Option<Vec<String>>,
    is_favorite: Option<bool>,
    is_archived: Option<bool>,
}

async fn create_note(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateNoteRequest>,
) -> (StatusCode, Json<Note>) {
    let mut note = Note::new(payload.title, payload.content);
    if let Some(notebook_id) = payload.notebook_id {
        note = note.with_notebook(notebook_id);
    }
    if let Some(parent_id) = payload.parent_id {
        note = note.with_parent(parent_id);
    }
    if let Some(tags) = payload.tags {
        note = note.with_tags(tags);
    }
    if let Some(is_favorite) = payload.is_favorite {
        note.is_favorite = is_favorite;
    }
    if let Some(is_archived) = payload.is_archived {
        note.is_archived = is_archived;
    }

    note.user_id = Some(current_user.id);

    let db = state.db.lock().await;
    match db.create_note_with_user(note).await {
        Ok(saved_note) => (StatusCode::CREATED, Json(saved_note)),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Note::new("Error".to_string(), "".to_string())),
        ),
    }
}

async fn get_note(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Note>) {
    let db = state.db.lock().await;
    match db.get_by_id(&id).await {
        Ok(note) => {
            if note.user_id.as_ref() == Some(&current_user.id) {
                (StatusCode::OK, Json(note))
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(Note::new("Error".to_string(), "".to_string())),
                )
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(Note::new("Error".to_string(), "".to_string())),
        ),
    }
}

async fn update_note(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CreateNoteRequest>,
) -> (StatusCode, Json<Note>) {
    let db = state.db.lock().await;
    let existing_note = match db.get_by_id(&id).await {
        Ok(note) => note,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(Note::new("Error".to_string(), "".to_string())),
            )
        }
    };

    if existing_note.user_id.as_ref() != Some(&current_user.id) {
        return (
            StatusCode::NOT_FOUND,
            Json(Note::new("Error".to_string(), "".to_string())),
        );
    }

    let mut note = Note::new(payload.title, payload.content);
    note.id = id;
    if let Some(notebook_id) = payload.notebook_id {
        note = note.with_notebook(notebook_id);
    }
    if let Some(parent_id) = payload.parent_id {
        note = note.with_parent(parent_id);
    }
    if let Some(tags) = payload.tags {
        note = note.with_tags(tags);
    }
    if let Some(is_favorite) = payload.is_favorite {
        note.is_favorite = is_favorite;
    }
    if let Some(is_archived) = payload.is_archived {
        note.is_archived = is_archived;
    }

    note.user_id = Some(current_user.id);

    match db.update_note_with_user(note).await {
        Ok(updated) => (StatusCode::OK, Json(updated)),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Note::new("Error".to_string(), "".to_string())),
        ),
    }
}

async fn delete_note(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> StatusCode {
    let db = state.db.lock().await;
    let existing_note = match db.get_by_id(&id).await {
        Ok(note) => note,
        Err(_) => return StatusCode::NOT_FOUND,
    };

    if existing_note.user_id.as_ref() != Some(&current_user.id) {
        return StatusCode::NOT_FOUND;
    }

    match db.delete(&id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

async fn get_note_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    debug!("get_note_content called with id: {}", id);
    let db = state.db.lock().await;
    match db.get_by_id(&id).await {
        Ok(note) => (StatusCode::OK, render_markdown(&note.content)),
        Err(_) => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

async fn health() -> &'static str {
    "OK - Routes loaded"
}

async fn get_vector_db_status(
    CurrentUser(_current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<VectorDBStatusResponse>) {
    let db = state.db.lock().await;
    let enabled = state.config.vector.enabled;
    let available = db.vector_store.is_some();

    let message = if available {
        "Vector database is enabled and available".to_string()
    } else if enabled {
        "Vector database is enabled but not available (Qdrant not running or connection failed)"
            .to_string()
    } else {
        "Vector database is disabled in configuration".to_string()
    };

    drop(db);

    (
        StatusCode::OK,
        Json(VectorDBStatusResponse {
            enabled,
            available,
            message,
        }),
    )
}

#[derive(serde::Serialize)]
struct VersionInfo {
    version: &'static str,
    version_short: &'static str,
    build: &'static str,
}

async fn version_info() -> (StatusCode, Json<VersionInfo>) {
    let build_info = env!("CARGO_PKG_VERSION");
    (
        StatusCode::OK,
        Json(VersionInfo {
            version: VERSION,
            version_short: VERSION_SHORT,
            build: build_info,
        }),
    )
}

#[derive(Serialize)]
struct VectorDBStatusResponse {
    enabled: bool,
    available: bool,
    message: String,
}

#[derive(Serialize)]
struct ImportResponse {
    imported: usize,
    note_ids: Vec<String>,
}

async fn import_tomboy(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TomboyImportRequest>,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    let db = state.db.lock().await;
    let mut created_notebooks: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for note_xml in payload.notes {
        let cleaned_xml = note_xml.trim();
        if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
            let mut notebook_name: Option<String> = None;
            let mut tags: Vec<String> = Vec::new();

            for tag in note.tags.clone() {
                if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                    notebook_name = Some(nb_name.to_string());
                } else {
                    tags.push(tag);
                }
            }

            let mut note_with_user = note.with_user_id(current_user.id.clone());

            if let Some(nb_name) = notebook_name {
                let notebook_id = if let Some(id) = created_notebooks.get(&nb_name) {
                    id.clone()
                } else {
                    match db.get_notebook_by_name(&nb_name, &current_user.id).await {
                        Ok(existing) => existing.id,
                        Err(_) => {
                            let notebook = Notebook::new(nb_name.clone())
                                .with_user_id(current_user.id.clone());
                            match db.create_notebook(notebook.clone()).await {
                                Ok(created) => {
                                    created_notebooks.insert(nb_name.clone(), created.id.clone());
                                    created.id
                                }
                                Err(_) => {
                                    if let Ok(existing) =
                                        db.get_notebook_by_name(&nb_name, &current_user.id).await
                                    {
                                        existing.id
                                    } else {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                };
                note_with_user = note_with_user.with_notebook(notebook_id);
                tags.push(nb_name);
            }

            note_with_user = note_with_user.with_tags(tags);

            if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
                imported_count += 1;
                imported_ids.push(saved_note.id);
            }
        }
    }

    (
        StatusCode::OK,
        Json(ImportResponse {
            imported: imported_count,
            note_ids: imported_ids,
        }),
    )
}

#[derive(Deserialize)]
struct TomboyImportRequest {
    notes: Vec<String>,
}

#[derive(Deserialize)]
struct TomboyXmlImportRequest {
    xml: String,
}

fn parse_single_tomboy_note(xml: &str) -> Result<Note, String> {
    use crate::import_export::tomboy::TomboyNote;

    match TomboyNote::parse_xml_string(xml) {
        Ok(tomboy_note) => Ok(tomboy_note.to_memo_rs_note_with_title_removed()),
        Err(e) => Err(format!("Failed to parse Tomboy note: {}", e)),
    }
}

async fn export_tomboy(State(state): State<Arc<AppState>>) -> (StatusCode, String) {
    let db = state.db.lock().await;
    match db.list(None, None).await {
        Ok(notes) => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<notes>\n");
            for note in notes {
                let tomboy_note =
                    TomboyExporter::note_to_tomboy(&note).unwrap_or_else(|_| TomboyNote {
                        title: note.title.clone(),
                        raw_content: format!("<note-content>\n{}\n</note-content>", note.content),
                        content: format!("<note-content>\n{}\n</note-content>", note.content),
                        tags: vec![],
                        attachments: vec![],
                        create_date: Some(note.created_at),
                        last_change_date: Some(note.updated_at),
                        last_metadata_change_date: None,
                    });

                let note_xml = format!(
                    r#"  <note version="0.1">
    <title>{}</title>
    <content>{}</content>
    <tags>
      {}
    </tags>
    <last-modified>{}</last-modified>
  </note>
"#,
                    note.title,
                    tomboy_note.content,
                    note.tags
                        .iter()
                        .map(|t| format!("      <tag>{}</tag>", t))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    note.updated_at.to_rfc3339()
                );
                xml.push_str(&note_xml);
            }
            xml.push_str("</notes>");
            (StatusCode::OK, xml)
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "".to_string()),
    }
}

async fn import_tomboy_file(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    let db = state.db.lock().await;
    let mut created_notebooks: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    while let Some(field) = multipart.next_field().await.ok().flatten() {
           let bytes = field.bytes().await.ok();
        if let Some(bytes) = bytes {
            let content = String::from_utf8_lossy(&bytes);

            let cleaned_xml = content.trim();
            if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
                let mut notebook_name: Option<String> = None;
                let mut tags: Vec<String> = Vec::new();

                for tag in note.tags.clone() {
                    if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                        notebook_name = Some(nb_name.to_string());
                    } else {
                        tags.push(tag);
                    }
                }

                let mut note_with_user = note.with_user_id(current_user.id.clone());

                if let Some(nb_name) = notebook_name {
                    let notebook_id = if let Some(id) = created_notebooks.get(&nb_name) {
                        id.clone()
                    } else {
                        match db.get_notebook_by_name(&nb_name, &current_user.id).await {
                            Ok(existing) => existing.id,
                            Err(_) => {
                                let notebook = Notebook::new(nb_name.clone())
                                    .with_user_id(current_user.id.clone());
                                match db.create_notebook(notebook.clone()).await {
                                    Ok(created) => {
                                        created_notebooks.insert(nb_name.clone(), created.id.clone());
                                        created.id
                                    }
                                    Err(_) => {
                                        if let Ok(existing) =
                                            db.get_notebook_by_name(&nb_name, &current_user.id).await
                                        {
                                            existing.id
                                        } else {
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    };
                    note_with_user = note_with_user.with_notebook(notebook_id);
                    tags.push(nb_name);
                }

                note_with_user = note_with_user.with_tags(tags);

                if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
                    imported_count += 1;
                    imported_ids.push(saved_note.id);
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(ImportResponse {
            imported: imported_count,
            note_ids: imported_ids,
        }),
    )
}

async fn import_tomboy_xml(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TomboyXmlImportRequest>,
) -> (StatusCode, Json<ImportResponse>) {
    let cleaned_xml = payload.xml.trim();

    if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
        let mut notebook_name: Option<String> = None;
        let mut tags: Vec<String> = Vec::new();

        for tag in note.tags.clone() {
            if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                notebook_name = Some(nb_name.to_string());
            } else {
                tags.push(tag);
            }
        }

        let mut note_with_user = note.with_user_id(current_user.id.clone());

        if let Some(nb_name) = notebook_name {
            let db = state.db.lock().await;
            match db.get_notebook_by_name(&nb_name, &current_user.id).await {
                Ok(existing) => {
                    note_with_user = note_with_user.with_notebook(existing.id);
                    tags.push(nb_name);
                }
                Err(_) => {
                    let notebook = Notebook::new(nb_name.clone())
                        .with_user_id(current_user.id.clone());
                    match db.create_notebook(notebook).await {
                        Ok(created) => {
                            note_with_user = note_with_user.with_notebook(created.id);
                            tags.push(nb_name);
                        }
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ImportResponse {
                                    imported: 0,
                                    note_ids: Vec::new(),
                                }),
                            );
                        }
                    }
                }
            }
        }

        note_with_user = note_with_user.with_tags(tags);

        let db = state.db.lock().await;
        if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
            return (
                StatusCode::OK,
                Json(ImportResponse {
                    imported: 1,
                    note_ids: vec![saved_note.id],
                }),
            );
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ImportResponse {
            imported: 0,
            note_ids: Vec::new(),
        }),
    )
}

async fn import_tomboy_directory(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ImportResponse {
                    imported: 0,
                    note_ids: Vec::new(),
                }),
            )
        }
    };
    let base_dir = temp_dir.path();

    let importer = TomboyImporter::new(base_dir.to_str().unwrap_or("/tmp"));

    if let Ok(notes) = importer.import_all_recursive() {
        let db = state.db.lock().await;
        let mut created_notebooks: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for note in notes {
            let mut notebook_name: Option<String> = None;
            let mut tags: Vec<String> = Vec::new();

            for tag in note.tags.clone() {
                if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                    notebook_name = Some(nb_name.to_string());
                } else {
                    tags.push(tag);
                }
            }

            let mut memo_rs_note = note.to_memo_rs_note().with_user_id(current_user.id.clone());

            if let Some(nb_name) = notebook_name {
                let notebook_id = if let Some(id) = created_notebooks.get(&nb_name) {
                    id.clone()
                } else {
                    match db.get_notebook_by_name(&nb_name, &current_user.id).await {
                        Ok(existing) => existing.id,
                        Err(_) => {
                            let notebook = Notebook::new(nb_name.clone())
                                .with_user_id(current_user.id.clone());
                            match db.create_notebook(notebook.clone()).await {
                                Ok(created) => {
                                    created_notebooks.insert(nb_name.clone(), created.id.clone());
                                    created.id
                                }
                                Err(_) => {
                                    if let Ok(existing) =
                                        db.get_notebook_by_name(&nb_name, &current_user.id).await
                                    {
                                        existing.id
                                    } else {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                };
                memo_rs_note = memo_rs_note.with_notebook(notebook_id);
                tags.push(nb_name);
            }

            memo_rs_note = memo_rs_note.with_tags(tags);

            if let Ok(saved_note) = db.create_note_with_user(memo_rs_note).await {
                imported_count += 1;
                imported_ids.push(saved_note.id);
            }
        }
    }

    (
        StatusCode::OK,
        Json(ImportResponse {
            imported: imported_count,
            note_ids: imported_ids,
        }),
    )
}

// Auth routes
async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<RegisterResult>) {
    debug!("register called with username: {}", payload.username);
    let auth_service = AuthService::new();

    match auth_service
        .register(
            state.db.clone(),
            &payload.username,
            &payload.email,
            &payload.password,
        )
        .await
    {
        Ok((user, session)) => (
            StatusCode::CREATED,
            Json(RegisterResult {
                success: true,
                message: "Registration successful".to_string(),
                errors: None,
                user: Some(UserProfile::from(user)),
                token: Some(session.session_token),
            }),
        ),
        Err(e) => {
            let error_response = match &e {
                AuthError::UsernameAlreadyExists => ErrorResponse {
                    message: "Registration failed".to_string(),
                    errors: Some(vec![ValidationError {
                        field: "username".to_string(),
                        message: "Username already exists".to_string(),
                    }]),
                },
                AuthError::EmailAlreadyExists => ErrorResponse {
                    message: "Registration failed".to_string(),
                    errors: Some(vec![ValidationError {
                        field: "email".to_string(),
                        message: "Email already exists".to_string(),
                    }]),
                },
                _ => ErrorResponse {
                    message: "Registration failed".to_string(),
                    errors: None,
                },
            };
            (
                StatusCode::BAD_REQUEST,
                Json(RegisterResult {
                    success: false,
                    message: error_response.message,
                    errors: error_response.errors,
                    user: None,
                    token: None,
                }),
            )
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    let auth_service = AuthService::new();

    match auth_service
        .login(state.db.clone(), &payload.username, &payload.password)
        .await
    {
        Ok((user, session)) => (
            StatusCode::OK,
            Json(LoginResponse {
                token: session.session_token,
                user: UserProfile::from(user),
                expires_at: session.expires_at,
            }),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                token: String::new(),
                user: UserProfile {
                    id: String::new(),
                    username: String::new(),
                    email: String::new(),
                    created_at: chrono::Utc::now(),
                    search_mode: "sql".to_string(),
                    llm_settings: serde_json::json!({
                      "provider": "openai",
                      "url": "http://localhost:11434/v1",
                      "api_key": null,
                      "model": "llama3",
                      "temperature": 0.7,
                      "max_tokens": 2048
                    }),
                },
                expires_at: chrono::Utc::now(),
            }),
        ),
    }
}

async fn logout(State(_state): State<Arc<AppState>>) -> StatusCode {
    StatusCode::OK
}

async fn refresh_token(State(state): State<Arc<AppState>>) -> (StatusCode, Json<LoginResponse>) {
    let auth_service = AuthService::new();

    let db_guard = state.db.lock().await;
    match db_guard.get_user_by_username("admin").await {
        Ok(user) => match auth_service.create_session(&db_guard, &user.id).await {
            Ok(session) => (
                StatusCode::OK,
                Json(LoginResponse {
                    token: session.session_token,
                    user: UserProfile::from(user),
                    expires_at: session.expires_at,
                }),
            ),
            Err(_) => (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    token: String::new(),
                    user: UserProfile {
                        id: String::new(),
                        username: String::new(),
                        email: String::new(),
                        created_at: chrono::Utc::now(),
                        search_mode: "sql".to_string(),
                        llm_settings: serde_json::json!({
                          "provider": "openai",
                          "url": "http://localhost:11434/v1",
                          "api_key": null,
                          "model": "llama3",
                          "temperature": 0.7,
                          "max_tokens": 2048
                        }),
                    },
                    expires_at: chrono::Utc::now(),
                }),
            ),
        },
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                token: String::new(),
                user: UserProfile {
                    id: String::new(),
                    username: String::new(),
                    email: String::new(),
                    created_at: chrono::Utc::now(),
                    search_mode: "sql".to_string(),
                    llm_settings: serde_json::json!({
                      "provider": "openai",
                      "url": "http://localhost:11434/v1",
                      "api_key": null,
                      "model": "llama3",
                      "temperature": 0.7,
                      "max_tokens": 2048
                    }),
                },
                expires_at: chrono::Utc::now(),
            }),
        ),
    }
}

async fn get_current_user(
    CurrentUser(current_user): CurrentUser,
) -> (StatusCode, Json<UserProfile>) {
    (StatusCode::OK, Json(current_user))
}

async fn update_profile(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateProfileRequest>,
) -> (StatusCode, Json<UserProfile>) {
    let auth_service = AuthService::new();
    let db = state.db.clone();

    if let (Some(current_password), Some(new_password)) =
        (payload.current_password, payload.new_password)
    {
        match auth_service
            .update_password(db, &current_user.id, &current_password, &new_password)
            .await
        {
            Ok(_) => {
                let db_guard = state.db.lock().await;
                match db_guard.get_user_by_id(&current_user.id).await {
                    Ok(user) => (StatusCode::OK, Json(UserProfile::from(user))),
                    Err(_) => (
                        StatusCode::UNAUTHORIZED,
                        Json(UserProfile {
                            id: String::new(),
                            username: String::new(),
                            email: String::new(),
                            created_at: chrono::Utc::now(),
                            search_mode: "sql".to_string(),
                            llm_settings: serde_json::json!({
                              "provider": "ollama",
                              "url": "http://localhost:11434",
                              "api_key": null,
                              "model": "llama2",
                              "temperature": 0.7,
                              "max_tokens": 2048
                            }),
                        }),
                    ),
                }
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(UserProfile {
                    id: String::new(),
                    username: String::new(),
                    email: String::new(),
                    created_at: chrono::Utc::now(),
                    search_mode: "sql".to_string(),
                    llm_settings: serde_json::json!({
                      "provider": "ollama",
                      "url": "http://localhost:11434",
                      "api_key": null,
                      "model": "llama2",
                      "temperature": 0.7,
                      "max_tokens": 2048
                    }),
                }),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(UserProfile {
                id: String::new(),
                username: String::new(),
                email: String::new(),
                created_at: chrono::Utc::now(),
                search_mode: "sql".to_string(),
                llm_settings: serde_json::json!({
                  "provider": "ollama",
                  "url": "http://localhost:11434",
                  "api_key": null,
                  "model": "llama2",
                  "temperature": 0.7,
                  "max_tokens": 2048
                }),
            }),
        )
    }
}

async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetRequest>,
) -> StatusCode {
    let auth_service = AuthService::new();

    if auth_service
        .request_password_reset(state.db.clone(), &payload.email)
        .await
        .is_ok()
    {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetConfirm>,
) -> StatusCode {
    let auth_service = AuthService::new();

    if auth_service
        .reset_password(state.db.clone(), &payload.token, &payload.password)
        .await
        .is_ok()
    {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Deserialize)]
struct UpdateSearchModeRequest {
    search_mode: String,
}

#[derive(Serialize)]
struct GetSearchModeResponse {
    search_mode: String,
}

async fn update_search_mode(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateSearchModeRequest>,
) -> (StatusCode, Json<GetSearchModeResponse>) {
    let db = state.db.lock().await;

    match db
        .update_user_search_mode(&current_user.id, &payload.search_mode)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(GetSearchModeResponse {
                search_mode: payload.search_mode,
            }),
        ),
        Err(e) => {
            error!("Failed to update search mode: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetSearchModeResponse {
                    search_mode: "sql".to_string(),
                }),
            )
        }
    }
}

async fn get_search_mode(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<GetSearchModeResponse>) {
    let db = state.db.lock().await;

    match db.get_user_metadata(&current_user.id).await {
        Ok(metadata) => {
            let search_mode = metadata
                .get("search_mode")
                .and_then(|v| v.as_str())
                .unwrap_or("sql")
                .to_string();

            (StatusCode::OK, Json(GetSearchModeResponse { search_mode }))
        }
        Err(e) => {
            error!("Failed to get user metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetSearchModeResponse {
                    search_mode: "sql".to_string(),
                }),
            )
        }
    }
}

#[derive(Deserialize)]
struct UpdateLLMSettingsPayload {
    llm_settings: serde_json::Value,
}

async fn update_llm_settings(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateLLMSettingsPayload>,
) -> (StatusCode, Json<GetLLMSettingsResponse>) {
    let db = state.db.lock().await;

    let current_settings = match db.get_user_llm_settings(&current_user.id).await {
        Ok(settings) => settings,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetLLMSettingsResponse {
                    provider: "openai".to_string(),
                    url: "http://localhost:11434/v1".to_string(),
                    api_key: None,
                    model: "llama3".to_string(),
                    temperature: 0.7,
                    max_tokens: 2048,
                }),
            )
        }
    };

    let mut updated_settings = current_settings.clone();

    if let Some(provider) = payload.llm_settings.get("provider") {
        updated_settings["provider"] = provider.clone();
    }
    if let Some(url) = payload.llm_settings.get("url") {
        updated_settings["url"] = url.clone();
    }
    if let Some(api_key) = payload.llm_settings.get("api_key") {
        updated_settings["api_key"] = api_key.clone();
    }
    if let Some(model) = payload.llm_settings.get("model") {
        updated_settings["model"] = model.clone();
    }
    if let Some(temperature) = payload.llm_settings.get("temperature") {
        updated_settings["temperature"] = temperature.clone();
    }
    if let Some(max_tokens) = payload.llm_settings.get("max_tokens") {
        updated_settings["max_tokens"] = max_tokens.clone();
    }

    debug!("update_llm_settings called with payload");
    debug!("Updated settings: {:?}", updated_settings);
    match db
        .update_user_llm_settings(&current_user.id, &updated_settings)
        .await
    {
        Ok(_) => {
            debug!("Settings updated successfully in database");
            let provider = updated_settings
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("ollama")
                .to_string();
            let url = updated_settings
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:11434")
                .to_string();
            let api_key = updated_settings
                .get("api_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let model = updated_settings
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("llama2")
                .to_string();
            let temperature = updated_settings
                .get("temperature")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7);
            let max_tokens = updated_settings
                .get("max_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(2048) as i32;

            (
                StatusCode::OK,
                Json(GetLLMSettingsResponse {
                    provider,
                    url,
                    api_key,
                    model,
                    temperature,
                    max_tokens,
                }),
            )
        }
        Err(e) => {
            error!("Failed to update LLM settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetLLMSettingsResponse {
                    provider: "openai".to_string(),
                    url: "http://localhost:11434/v1".to_string(),
                    api_key: None,
                    model: "llama3".to_string(),
                    temperature: 0.7,
                    max_tokens: 2048,
                }),
            )
        }
    }
}

async fn get_llm_settings(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<GetLLMSettingsResponse>) {
    let db = state.db.lock().await;

    match db.get_user_llm_settings(&current_user.id).await {
        Ok(settings) => {
            let provider = settings
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("ollama")
                .to_string();
            let url = settings
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:11434")
                .to_string();
            let api_key = settings
                .get("api_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let model = settings
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("llama2")
                .to_string();
            let temperature = settings
                .get("temperature")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7);
            let max_tokens = settings
                .get("max_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(2048) as i32;

            (
                StatusCode::OK,
                Json(GetLLMSettingsResponse {
                    provider,
                    url,
                    api_key,
                    model,
                    temperature,
                    max_tokens,
                }),
            )
        }
        Err(e) => {
            error!("Failed to get LLM settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetLLMSettingsResponse {
                    provider: "openai".to_string(),
                    url: "http://localhost:11434/v1".to_string(),
                    api_key: None,
                    model: "llama3".to_string(),
                    temperature: 0.7,
                    max_tokens: 2048,
                }),
            )
        }
    }
}

async fn test_llm_connection(
    CurrentUser(_current_user): CurrentUser,
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    debug!("Testing LLM connection with settings: {:?}", payload);
    StatusCode::OK
}

#[derive(Deserialize)]
struct CreateNotebookRequest {
    name: String,
    parent_id: Option<String>,
}

async fn list_notebooks(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<Notebook>>) {
    let db = state.db.lock().await;
    match db.list_notebooks_by_user(&current_user.id).await {
        Ok(notebooks) => (StatusCode::OK, Json(notebooks)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

async fn get_notebook(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Notebook>) {
    let db = state.db.lock().await;
    match db.get_notebook_by_id(&id, &current_user.id).await {
        Ok(notebook) => (StatusCode::OK, Json(notebook)),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(Notebook::new("Error".to_string())),
        ),
    }
}

async fn create_notebook(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateNotebookRequest>,
) -> (StatusCode, Json<Notebook>) {
    let mut notebook = Notebook::new(payload.name);
    if let Some(parent_id) = payload.parent_id {
        notebook = notebook.with_parent(parent_id);
    }
    notebook = notebook.with_user_id(current_user.id.clone());

    let db = state.db.lock().await;
    match db.create_notebook(notebook).await {
        Ok(saved_notebook) => (StatusCode::CREATED, Json(saved_notebook)),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Notebook::new("Error".to_string())),
        ),
    }
}

async fn update_notebook(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CreateNotebookRequest>,
) -> (StatusCode, Json<Notebook>) {
    let db = state.db.lock().await;
    let existing_notebook = match db.get_notebook_by_id(&id, &current_user.id).await {
        Ok(notebook) => notebook,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(Notebook::new("Error".to_string())),
            )
        }
    };

    let mut notebook = Notebook::new(payload.name);
    notebook.id = id;
    if let Some(parent_id) = payload.parent_id {
        notebook = notebook.with_parent(parent_id);
    }
    notebook.user_id = Some(current_user.id);

    match db.update_notebook(notebook).await {
        Ok(updated) => (StatusCode::OK, Json(updated)),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Notebook::new("Error".to_string())),
        ),
    }
}

async fn delete_notebook(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> StatusCode {
    let db = state.db.lock().await;
    let existing_notebook = match db.get_notebook_by_id(&id, &current_user.id).await {
        Ok(notebook) => notebook,
        Err(_) => return StatusCode::NOT_FOUND,
    };

    if existing_notebook.user_id.as_ref() != Some(&current_user.id) {
        return StatusCode::NOT_FOUND;
    }

    match db.delete_notebook(&id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[derive(Deserialize)]
struct ListNotesByNotebookQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

async fn get_notes_by_notebook(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<ListNotesByNotebookQuery>,
) -> (StatusCode, Json<Vec<Note>>) {
    let db = state.db.lock().await;
    match db
        .get_notes_by_notebook_id(&id, query.limit, query.offset)
        .await
    {
        Ok(notes) => (StatusCode::OK, Json(notes)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

async fn get_notebooks_tree(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    match db.get_notebooks_tree(&current_user.id).await {
        Ok(tree) => {
            let tree_json = serialize_notebook_tree(&tree);
            (StatusCode::OK, Json(tree_json))
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!([])),
        ),
    }
}

async fn get_folder_contents(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    match db.get_folder_contents(Some(&id), &current_user.id).await {
        Ok(node) => {
            let json = serialize_notebook_node(&node);
            (StatusCode::OK, Json(json))
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        ),
    }
}

async fn get_root_contents(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    match db.get_folder_contents(None, &current_user.id).await {
        Ok(node) => {
            let json = serialize_notebook_node(&node);
            (StatusCode::OK, Json(json))
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        ),
    }
}

#[derive(Deserialize)]
struct ReorderItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    parent_id: Option<String>,
    order: Option<i64>,
}

#[derive(Deserialize)]
struct ReorderRequest {
    items: Vec<ReorderItem>,
}

#[derive(Deserialize)]
struct BulkDeleteRequest {
    ids: Vec<String>,
    #[serde(rename = "type")]
    item_type: String,
}

async fn reorder_notebooks(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    let mut results = Vec::new();

    for item in &payload.items {
        if item.item_type == "notebook" {
            let notebook = match db.get_notebook_by_id(&item.id, &current_user.id).await {
                Ok(n) => n,
                Err(_) => continue,
            };

            let mut updated_notebook = Notebook::new(notebook.name.clone());
            updated_notebook.id = notebook.id.clone();
            updated_notebook.parent_id = item.parent_id.clone();
            updated_notebook.user_id = Some(current_user.id.clone());

            if let Err(e) = db.update_notebook(updated_notebook).await {
                results.push(serde_json::json!({
                    "id": item.id,
                    "success": false,
                    "error": e.to_string()
                }));
                continue;
            }
            results.push(serde_json::json!({
                "id": item.id,
                "success": true
            }));
        } else if item.item_type == "note" {
            let note = match db.get_by_id(&item.id).await {
                Ok(n) => n,
                Err(_) => continue,
            };

            if note.user_id.as_ref() != Some(&current_user.id) {
                continue;
            }

            let mut updated_note = Note::new(note.title.clone(), note.content.clone());
            updated_note.id = note.id.clone();
            updated_note.notebook_id = item.parent_id.clone();
            updated_note.parent_id = note.parent_id.clone();
            updated_note.tags = note.tags.clone();
            updated_note.user_id = Some(current_user.id.clone());

            if let Err(e) = db.update_note_with_user(updated_note).await {
                results.push(serde_json::json!({
                    "id": item.id,
                    "success": false,
                    "error": e.to_string()
                }));
                continue;
            }
            results.push(serde_json::json!({
                "id": item.id,
                "success": true
            }));
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "results": results,
        "total": results.len()
    })))
}

async fn reorder_notes(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    let mut results = Vec::new();

    for item in &payload.items {
        if item.item_type == "note" {
            let note = match db.get_by_id(&item.id).await {
                Ok(n) => n,
                Err(_) => continue,
            };

            if note.user_id.as_ref() != Some(&current_user.id) {
                continue;
            }

            let mut updated_note = Note::new(note.title.clone(), note.content.clone());
            updated_note.id = note.id.clone();
            updated_note.notebook_id = item.parent_id.clone();
            updated_note.parent_id = note.parent_id.clone();
            updated_note.tags = note.tags.clone();
            updated_note.user_id = Some(current_user.id.clone());

            if let Err(e) = db.update_note_with_user(updated_note).await {
                results.push(serde_json::json!({
                    "id": item.id,
                    "success": false,
                    "error": e.to_string()
                }));
                continue;
            }
            results.push(serde_json::json!({
                "id": item.id,
                "success": true
            }));
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "results": results,
        "total": results.len()
    })))
}

async fn bulk_delete(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BulkDeleteRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().await;
    let mut deleted_ids = Vec::new();
    let mut errors = Vec::new();

    for id in &payload.ids {
        if payload.item_type == "notebook" {
            match db.delete_notebook(id).await {
                Ok(()) => deleted_ids.push(id.clone()),
                Err(e) => errors.push(serde_json::json!({
                    "id": id,
                    "type": "notebook",
                    "error": e.to_string()
                })),
            }
        } else if payload.item_type == "note" {
            match db.delete(id).await {
                Ok(()) => deleted_ids.push(id.clone()),
                Err(e) => errors.push(serde_json::json!({
                    "id": id,
                    "type": "note",
                    "error": e.to_string()
                })),
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "deleted": deleted_ids,
        "errors": errors,
        "total_deleted": deleted_ids.len()
    })))
}
