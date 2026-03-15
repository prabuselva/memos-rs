use crate::import_export::tomboy::{TomboyExporter, TomboyImporter, TomboyNote};
use crate::models::auth_dto::{ErrorResponse, LoginRequest, LoginResponse, PasswordResetConfirm, PasswordResetRequest, RegisterRequest, RegisterResult, UpdateProfileRequest, ValidationError};
use crate::models::{UserProfile, AuthError};
use crate::services::auth_service::AuthService;
use crate::{models::Note, AppState, VERSION, VERSION_SHORT};
use axum::{
    extract::{FromRequestParts, Json, Multipart, Path, Query, State},
    http::{request::Parts, StatusCode},
    routing::{get, post, put},
    Router,
};
use std::env;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct CurrentUser(pub UserProfile);

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for CurrentUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
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
        
        Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
    }
}

pub fn create_router() -> Router<Arc<AppState>> {
    let notes_router = Router::new()
        .route("/", get(get_note).put(update_note).delete(delete_note))
        .route("/content", get(get_note_content));

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .route("/profile", put(update_profile))
        .route("/request-password-reset", post(request_password_reset))
        .route("/reset-password", post(reset_password))
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/search", get(search_notes))
        .nest("/notes/:id", notes_router)
        .route("/import/tomboy", post(import_tomboy))
        .route("/import/tomboy/file", post(import_tomboy_file))
        .route("/import/tomboy/xml", post(import_tomboy_xml))
        .route("/import/tomboy/directory", post(import_tomboy_directory))
        .route("/export/tomboy", get(export_tomboy))
        .route("/version", get(version_info))
        .route("/health", get(health))
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
    match db.get_notes_by_user(&current_user.id, query.limit, query.offset).await {
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
    match db.search_notes_by_user(&current_user.id, &query.q, 10).await {
        Ok(notes) => (StatusCode::OK, Json(notes)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[derive(Deserialize)]
struct CreateNoteRequest {
    title: String,
    content: String,
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
        Err(_) => (StatusCode::BAD_REQUEST, Json(Note::new("Error".to_string(), "".to_string()))),
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
                (StatusCode::NOT_FOUND, Json(Note::new("Error".to_string(), "".to_string())))
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(Note::new("Error".to_string(), "".to_string()))),
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
        Err(_) => return (StatusCode::NOT_FOUND, Json(Note::new("Error".to_string(), "".to_string()))),
    };
    
    if existing_note.user_id.as_ref() != Some(&current_user.id) {
        return (StatusCode::NOT_FOUND, Json(Note::new("Error".to_string(), "".to_string())));
    }
    
    let mut note = Note::new(payload.title, payload.content);
    note.id = id;
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
        Err(_) => (StatusCode::BAD_REQUEST, Json(Note::new("Error".to_string(), "".to_string()))),
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
    eprintln!("get_note_content called with id: {}", id);
    let db = state.db.lock().await;
    match db.get_by_id(&id).await {
        Ok(note) => (StatusCode::OK, render_markdown(&note.content)),
        Err(_) => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

async fn health() -> &'static str {
    "OK - Routes loaded"
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

async fn rollback_import_tomboy(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RollbackRequest>,
) -> (StatusCode, Json<RollbackResponse>) {
    let db = state.db.lock().await;
    match db.delete_notes_by_ids(&payload.note_ids).await {
        Ok(count) => (StatusCode::OK, Json(RollbackResponse { deleted: count })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RollbackResponse { deleted: 0 })),
    }
}

#[derive(Serialize)]
struct ImportResponse {
    imported: usize,
    note_ids: Vec<String>,
}

#[derive(Deserialize)]
struct RollbackRequest {
    note_ids: Vec<String>,
}

#[derive(Serialize)]
struct RollbackResponse {
    deleted: usize,
}

async fn import_tomboy(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TomboyImportRequest>,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    for note_xml in payload.notes {
        let cleaned_xml = note_xml.trim().replace('\n', " ");
        if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
            let note_with_user = note.with_user_id(current_user.id.clone());
            let db = state.db.lock().await;
            if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
                imported_count += 1;
                imported_ids.push(saved_note.id.clone());
            }
        }
    }

    (StatusCode::OK, Json(ImportResponse { imported: imported_count, note_ids: imported_ids }))
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
    let mut title = String::new();
    let mut content = String::new();
    let mut tags = Vec::new();

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                match e.name().as_ref() {
                    b"title" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            title = text.to_string();
                        }
                    }
                    b"content" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            content = text.to_string();
                        }
                    }
                    b"note-content" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            content = text.to_string();
                        }
                    }
                    b"tag" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            tags.push(text.to_string());
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::End(e)) => {
                if e.name().as_ref() == b"note" {
                    break;
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            _ => {}
        }
    }

    if title.is_empty() {
        return Err("Title is required".to_string());
    }

    Ok(Note::new(title, content).with_tags(tags))
}

async fn export_tomboy(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, String) {
    let db = state.db.lock().await;
    match db.list(None, None).await {
        Ok(notes) => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<notes>\n");
            for note in notes {
                let tomboy_note = TomboyExporter::note_to_tomboy(&note).unwrap_or_else(|_| {
                    TomboyNote {
                        title: note.title.clone(),
                        content: format!("<note-content>\n{}\n</note-content>", note.content),
                        tags: vec![],
                        attachments: vec![],
                        last_modified: note.updated_at,
                    }
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
                    note.tags.iter().map(|t| format!("      <tag>{}</tag>", t)).collect::<Vec<_>>().join("\n"),
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

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let bytes = field.bytes().await.ok();
        if let Some(bytes) = bytes {
            let content = String::from_utf8_lossy(&bytes);

            let cleaned_xml = content.trim().replace('\n', " ");
            if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
                let note_with_user = note.with_user_id(current_user.id.clone());
                let db = state.db.lock().await;
                if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
                    imported_count += 1;
                    imported_ids.push(saved_note.id.clone());
                }
            }
        }
    }

    (StatusCode::OK, Json(ImportResponse { imported: imported_count, note_ids: imported_ids }))
}

async fn import_tomboy_xml(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TomboyXmlImportRequest>,
) -> (StatusCode, Json<ImportResponse>) {
    let cleaned_xml = payload.xml.trim().replace('\n', " ");

    if let Ok(note) = parse_single_tomboy_note(&cleaned_xml) {
        let note_with_user = note.with_user_id(current_user.id.clone());
        let db = state.db.lock().await;
        if let Ok(saved_note) = db.create_note_with_user(note_with_user).await {
            return (StatusCode::OK, Json(ImportResponse {
                imported: 1,
                note_ids: vec![saved_note.id],
            }));
        }
    }

    (StatusCode::BAD_REQUEST, Json(ImportResponse { imported: 0, note_ids: Vec::new() }))
}

async fn import_tomboy_directory(
    CurrentUser(current_user): CurrentUser,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<ImportResponse>) {
    let mut imported_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ImportResponse { imported: 0, note_ids: Vec::new() })),
    };
    let base_dir = temp_dir.path();

    let importer = TomboyImporter::new(base_dir.to_str().unwrap_or("/tmp"));

    if let Ok(notes) = importer.import_all_recursive() {
        let db = state.db.lock().await;
        for note in notes {
            let memo_rs_note = note.to_memo_rs_note().with_user_id(current_user.id.clone());
            if let Ok(saved_note) = db.create_note_with_user(memo_rs_note).await {
                imported_count += 1;
                imported_ids.push(saved_note.id);
            }
        }
    }

    (StatusCode::OK, Json(ImportResponse { imported: imported_count, note_ids: imported_ids }))
}

// Auth routes
async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<RegisterResult>) {
    let auth_service = AuthService::new();

    match auth_service.register(
        state.db.clone(),
        &payload.username,
        &payload.email,
        &payload.password,
    ).await {
        Ok((user, session)) => {
            (StatusCode::CREATED, Json(RegisterResult {
                success: true,
                message: "Registration successful".to_string(),
                errors: None,
                user: Some(UserProfile::from(user)),
                token: Some(session.session_token),
            }))
        }
        Err(e) => {
            let error_response = match &e {
                AuthError::UsernameAlreadyExists => {
                    ErrorResponse {
                        message: "Registration failed".to_string(),
                        errors: Some(vec![ValidationError {
                            field: "username".to_string(),
                            message: "Username already exists".to_string(),
                        }]),
                    }
                }
                AuthError::EmailAlreadyExists => {
                    ErrorResponse {
                        message: "Registration failed".to_string(),
                        errors: Some(vec![ValidationError {
                            field: "email".to_string(),
                            message: "Email already exists".to_string(),
                        }]),
                    }
                }
                _ => ErrorResponse {
                    message: "Registration failed".to_string(),
                    errors: None,
                },
            };
            (StatusCode::BAD_REQUEST, Json(RegisterResult {
                success: false,
                message: error_response.message,
                errors: error_response.errors,
                user: None,
                token: None,
            }))
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    let auth_service = AuthService::new();

    match auth_service.login(
        state.db.clone(),
        &payload.username,
        &payload.password,
    ).await {
        Ok((user, session)) => {
            (StatusCode::OK, Json(LoginResponse {
                token: session.session_token,
                user: UserProfile::from(user),
                expires_at: session.expires_at,
            }))
        }
        Err(_) => (StatusCode::UNAUTHORIZED, Json(LoginResponse {
            token: String::new(),
            user: UserProfile {
                id: String::new(),
                username: String::new(),
                email: String::new(),
                created_at: chrono::Utc::now(),
            },
            expires_at: chrono::Utc::now(),
        })),
    }
}

async fn logout(
    State(_state): State<Arc<AppState>>,
) -> StatusCode {
    StatusCode::OK
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<LoginResponse>) {
    let auth_service = AuthService::new();
    
    let db_guard = state.db.lock().await;
    match db_guard.get_user_by_username("admin").await {
        Ok(user) => {
            match auth_service.create_session(&db_guard, &user.id).await {
                Ok(session) => (StatusCode::OK, Json(LoginResponse {
                    token: session.session_token,
                    user: UserProfile::from(user),
                    expires_at: session.expires_at,
                })),
                Err(_) => (StatusCode::UNAUTHORIZED, Json(LoginResponse {
                    token: String::new(),
                    user: UserProfile {
                        id: String::new(),
                        username: String::new(),
                        email: String::new(),
                        created_at: chrono::Utc::now(),
                    },
                    expires_at: chrono::Utc::now(),
                })),
            }
        }
        Err(_) => (StatusCode::UNAUTHORIZED, Json(LoginResponse {
            token: String::new(),
            user: UserProfile {
                id: String::new(),
                username: String::new(),
                email: String::new(),
                created_at: chrono::Utc::now(),
            },
            expires_at: chrono::Utc::now(),
        })),
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
    
    if let (Some(current_password), Some(new_password)) = (payload.current_password, payload.new_password) {
        match auth_service.update_password(db, &current_user.id, &current_password, &new_password).await {
            Ok(_) => {
                let db_guard = state.db.lock().await;
                match db_guard.get_user_by_id(&current_user.id).await {
                    Ok(user) => (StatusCode::OK, Json(UserProfile::from(user))),
                    Err(_) => (StatusCode::UNAUTHORIZED, Json(UserProfile {
                        id: String::new(),
                        username: String::new(),
                        email: String::new(),
                        created_at: chrono::Utc::now(),
                    })),
                }
            }
            Err(_) => (StatusCode::BAD_REQUEST, Json(UserProfile {
                id: String::new(),
                username: String::new(),
                email: String::new(),
                created_at: chrono::Utc::now(),
            })),
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(UserProfile {
            id: String::new(),
            username: String::new(),
            email: String::new(),
            created_at: chrono::Utc::now(),
        }))
    }
}

async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetRequest>,
) -> StatusCode {
    let auth_service = AuthService::new();

    if auth_service.request_password_reset(state.db.clone(), &payload.email).await.is_ok() {
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

    if auth_service.reset_password(state.db.clone(), &payload.token, &payload.password).await.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}