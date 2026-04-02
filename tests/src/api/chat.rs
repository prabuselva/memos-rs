use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as AxumJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

use crate::config::RagConfig;
use crate::embeddings::EmbeddingModel;
use crate::llm::{generate_llm_response, generate_llm_response_with_history, test_llm_connection};
use crate::models::UserProfile;
use crate::AppState;
use crate::{NoteReference, Reference, SearchMetadata};

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub context_notes: Vec<NoteReference>,
    pub references: Vec<Reference>,
    pub search_metadata: SearchMetadata,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageHistory {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub query: String,
    pub mode: Option<String>,
    pub context_note_ids: Option<Vec<u64>>,
    pub history: Option<Vec<ChatMessageHistory>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RagSearchRequest {
    pub query: String,
    pub limit: Option<u32>,
    pub mode: Option<String>,
    pub context_note_ids: Option<Vec<u64>>,
    pub use_hybrid_search: Option<bool>,
    pub stream: Option<bool>,
    pub max_context_tokens: Option<usize>,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<AxumJson<ChatHealthResponse>, (StatusCode, String)> {
    let mut status = ChatHealthStatus::Healthy;
    let mut messages = Vec::new();

    // Check database - just check if we can acquire a connection
    let db = state.db.lock().await;
    match db.get_pool().await.try_acquire() {
        Some(_) => messages.push("Database: OK".to_string()),
        None => {
            status = ChatHealthStatus::Degraded;
            messages.push("Database: CONNECTION FAILED".to_string());
        }
    }

    // Check vector store
    if let Some(vector_store) = &db.vector_store {
        match vector_store
            .search_notes(vec![0.0; 384], "health-check", 1)
            .await
        {
            Ok(_) => messages.push("Vector Store: OK".to_string()),
            Err(e) => {
                status = ChatHealthStatus::Degraded;
                messages.push(format!("Vector Store: ERROR - {}", e));
            }
        }
    } else {
        messages.push("Vector Store: Not enabled".to_string());
    }
    drop(db);

    // Check LLM
    let user_profile = UserProfile {
        id: "health-check".to_string(),
        username: "health".to_string(),
        email: "health@example.com".to_string(),
        created_at: chrono::Utc::now(),
        search_mode: "sqlite".to_string(),
        llm_settings: serde_json::json!({
            "provider": "openai",
            "url": "http://192.168.0.87:8083/v1",
            "api_key": "123456",
            "model": "llama3",
            "temperature": 0.7,
            "max_tokens": 2048
        }),
    };

    let settings = build_llm_settings_from_profile(&user_profile);
    match test_llm_connection(&settings).await {
        Ok(_) => messages.push("LLM: OK".to_string()),
        Err(e) => {
            status = ChatHealthStatus::Degraded;
            messages.push(format!("LLM: ERROR - {}", e));
        }
    }

    let response = ChatHealthResponse { status, messages };

    match status {
        ChatHealthStatus::Healthy => Ok(AxumJson(response.clone())),
        ChatHealthStatus::Degraded => Ok(AxumJson(response)),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatHealthResponse {
    status: ChatHealthStatus,
    messages: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatHealthStatus {
    Healthy,
    Degraded,
}

async fn get_user_profile(
    headers: &axum::http::HeaderMap,
    state: &Arc<AppState>,
) -> Result<UserProfile, (StatusCode, String)> {
    let auth_str = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ))?;

    if !auth_str.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid authorization format".to_string(),
        ));
    }

    let token = auth_str.strip_prefix("Bearer ").unwrap();
    let auth_service = crate::services::auth_service::AuthService::new();

    auth_service
        .validate_session(state.db.clone(), token)
        .await
        .map(UserProfile::from)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid session: {}", e)))
}

pub fn router<M: EmbeddingModel + Clone + 'static>(_model: Arc<M>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/query", post(chat_query))
        .route("/search", post(search_notes))
        .route("/search-rag", post(rag_search_notes))
        .route("/health", get(health_check))
}

async fn chat_query(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    info!("chat_query called with query: {}", req.query);
    debug!("[chat_query::DEBUG] Starting chat_query endpoint");
    debug!("[chat_query::DEBUG] Query: {}", req.query);
    debug!("[chat_query::DEBUG] Mode: {:?}", req.mode);
    debug!("[chat_query::DEBUG] Context note IDs: {:?}", req.context_note_ids);
    debug!("[chat_query::DEBUG] History length: {:?}", req.history.as_ref().map(|h| h.len()));

    if req.query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query cannot be empty".to_string()));
    }

    debug!("[chat_query::DEBUG] Getting user profile");
    let model: &Arc<dyn EmbeddingModel + Send + Sync> = &state.embedding_model;
    let user_profile = get_user_profile(&headers, &state).await?;
    debug!("[chat_query::DEBUG] User profile obtained: {}", user_profile.id);
    let mode = req.mode.as_deref().unwrap_or("rag");
    debug!("[chat_query::DEBUG] Final mode: {}", mode);

    match mode {
        "search" => {
            debug!("[chat_query::DEBUG] Handling search mode");
            handle_search_mode(&req.query, model, &state, &user_profile, &state.rag_config).await
        }
        "rag" => {
            debug!("[chat_query::DEBUG] Handling RAG mode");
            handle_rag_mode(&req.query, model, &state, &user_profile, &state.rag_config).await
        }
        "chat" => {
            debug!("[chat_query::DEBUG] Handling chat mode");
            handle_chat_mode(
                &req.query,
                req.context_note_ids,
                req.history,
                model,
                &state,
                &user_profile,
            )
            .await
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid mode: '{}'. Use 'search', 'rag', or 'chat'", mode),
        )),
    }
}

async fn handle_rag_mode(
    query: &str,
    model: &Arc<dyn EmbeddingModel + Send + Sync>,
    state: &Arc<AppState>,
    user_profile: &UserProfile,
    rag_config: &RagConfig,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    debug!("handle_rag_mode called with RAG enabled");
    debug!("[handle_rag_mode::DEBUG] Starting RAG mode handling");
    debug!("[handle_rag_mode::DEBUG] Embedding model: {}", model.name());
    debug!("[handle_rag_mode::DEBUG] Embedding query: {}", query);

    debug!("[handle_rag_mode::DEBUG] Computing embedding for query");
    let mut embedding = model.embed(query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to embed query: {}", e),
        )
    })?;
    debug!("[handle_rag_mode::DEBUG] Embedding computed, dimension: {}", embedding.len());
    debug!(
        "[handle_rag_mode::DEBUG] First 5 embedding values: {:?}",
        &embedding[0..5.min(embedding.len())]
    );

    debug!("[handle_rag_mode::DEBUG] Normalizing embedding vector");
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();
    debug!("[handle_rag_mode::DEBUG] Embedding normalized, norm: {}", norm);

    debug!("[handle_rag_mode::DEBUG] Performing RAG search");
    let (context_notes, search_time) =
        perform_rag_search(query, &embedding, state, user_profile, rag_config).await?;
    debug!(
        "[handle_rag_mode::DEBUG] RAG search complete, {} notes retrieved in {}ms",
        context_notes.len(),
        search_time
    );

    debug!("[handle_rag_mode::DEBUG] Building dynamic context");
    let context: Vec<String> = build_dynamic_context(&context_notes, rag_config.max_context_tokens);
    debug!(
        "[handle_rag_mode::DEBUG] Context built, {} chunks, max_tokens: {}",
        context.len(),
        rag_config.max_context_tokens
    );

    debug!("[handle_rag_mode::DEBUG] Calling LLM with context");
    debug!("[handle_rag_mode::DEBUG] Context notes returned from vector search:");
    for (i, note) in context_notes.iter().enumerate() {
        debug!(
            "[handle_rag_mode::DEBUG]   Note {}: id={}, title={}, score={:.4}, distance={:.4}",
            i, note.note_id, note.title, note.score, note.distance
        );
        debug!(
            "[handle_rag_mode::DEBUG]   Note {} content preview: {} chars",
            i,
            note.content.len()
        );
    }
    let start_time = Instant::now();
    let response = if context.is_empty() {
        format!(
            "I couldn't find any relevant notes for your query.\n\nQuery: {}",
            query
        )
    } else {
        debug!("[handle_rag_mode::DEBUG] Context not empty, calling LLM");
        debug!("[handle_rag_mode::DEBUG] Context chunks to be sent to LLM:");
        for (i, chunk) in context.iter().enumerate() {
            debug!("[handle_rag_mode::DEBUG]   Chunk {}: {} chars", i, chunk.len());
            debug!("[handle_rag_mode::DEBUG]   Chunk {}: {}...", i, &chunk[..50.min(chunk.len())]);
        }
        debug!("[handle_rag_mode::DEBUG] LLM query JSON:");
        debug!("[handle_rag_mode::DEBUG]   Query: {}", query);
        debug!("[handle_rag_mode::DEBUG]   Context size: {} chunks", context.len());
        match call_llm_api_with_context(query, &context, user_profile).await {
            Ok(resp) => resp,
            Err(_) => generate_search_response(query, &context_notes),
        }
    };
    debug!("[handle_rag_mode::DEBUG] LLM response generated");
    debug!("[handle_rag_mode::DEBUG] LLM response: {} chars", response.len());

    let generation_time = start_time.elapsed().as_millis() as u64;
    let total_tokens = response.split_whitespace().count();

    debug!("[handle_rag_mode::DEBUG] Building references");
    let references = build_references_from_notes(&context_notes, &[]);

    let search_metadata = SearchMetadata {
        query: query.to_string(),
        vector_search_time_ms: search_time,
        llm_generation_time_ms: generation_time,
        total_tokens,
        retrieved_count: context_notes.len(),
        filtered_count: 0,
        hybrid_search: rag_config.hybrid_search,
        model: model.name().to_string(),
    };

    debug!("[handle_rag_mode::DEBUG] Returning RAG response");
    Ok(AxumJson(ChatResponse {
        response,
        context_notes,
        references,
        search_metadata,
        model: model.name().to_string(),
    }))
}

async fn handle_search_mode(
    query: &str,
    model: &Arc<dyn EmbeddingModel + Send + Sync>,
    state: &Arc<AppState>,
    user_profile: &UserProfile,
    rag_config: &RagConfig,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    debug!("handle_search_mode called");
    debug!("[handle_search_mode::DEBUG] Starting search mode handling");
    debug!("[handle_search_mode::DEBUG] Embedding model: {}", model.name());
    debug!("[handle_search_mode::DEBUG] Embedding query: {}", query);

    debug!("[handle_search_mode::DEBUG] Computing embedding for query");
    let mut embedding = model.embed(query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to embed query: {}", e),
        )
    })?;
    debug!("[handle_search_mode::DEBUG] Embedding computed, dimension: {}", embedding.len());
    debug!(
        "[handle_search_mode::DEBUG] First 5 embedding values: {:?}",
        &embedding[0..5.min(embedding.len())]
    );

    debug!("[handle_search_mode::DEBUG] Normalizing embedding vector");
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();
    debug!("[handle_search_mode::DEBUG] Embedding normalized, norm: {}", norm);

    debug!("[handle_search_mode::DEBUG] Performing RAG search");
    let (context_notes, search_time) =
        perform_rag_search(query, &embedding, state, user_profile, rag_config).await?;
    debug!(
        "[handle_search_mode::DEBUG] RAG search complete, {} notes retrieved in {}ms",
        context_notes.len(),
        search_time
    );

    debug!("[handle_search_mode::DEBUG] Building dynamic context");
    let context: Vec<String> = build_dynamic_context(&context_notes, rag_config.max_context_tokens);
    debug!(
        "[handle_search_mode::DEBUG] Context built, {} chunks, max_tokens: {}",
        context.len(),
        rag_config.max_context_tokens
    );

    debug!("[handle_search_mode::DEBUG] Calling LLM with context");
    debug!("[handle_search_mode::DEBUG] Context notes returned from vector search:");
    for (i, note) in context_notes.iter().enumerate() {
        debug!(
            "[handle_search_mode::DEBUG]   Note {}: id={}, title={}, score={:.4}, distance={:.4}",
            i, note.note_id, note.title, note.score, note.distance
        );
        debug!(
            "[handle_search_mode::DEBUG]   Note {} content preview: {} chars",
            i,
            note.content.len()
        );
    }
    let start_time = Instant::now();
    let response = if context.is_empty() {
        format!(
            "I couldn't find any relevant notes for your query.\n\nQuery: {}",
            query
        )
    } else {
        debug!("[handle_search_mode::DEBUG] Context not empty, calling LLM");
        debug!("[handle_search_mode::DEBUG] Context chunks to be sent to LLM:");
        for (i, chunk) in context.iter().enumerate() {
            debug!("[handle_search_mode::DEBUG]   Chunk {}: {} chars", i, chunk.len());
            debug!("[handle_search_mode::DEBUG]   Chunk {}: {}...", i, &chunk[..50.min(chunk.len())]);
        }
        debug!("[handle_search_mode::DEBUG] LLM query JSON:");
        debug!("[handle_search_mode::DEBUG]   Query: {}", query);
        debug!("[handle_search_mode::DEBUG]   Context size: {} chunks", context.len());
        match call_llm_api_with_context(query, &context, user_profile).await {
            Ok(resp) => resp,
            Err(_) => generate_search_response(query, &context_notes),
        }
    };
    debug!("[handle_search_mode::DEBUG] LLM response generated");
    debug!("[handle_search_mode::DEBUG] LLM response: {} chars", response.len());

    let generation_time = start_time.elapsed().as_millis() as u64;
    let total_tokens = response.split_whitespace().count();

    debug!("[handle_search_mode::DEBUG] Building references");
    let references = build_references_from_notes(&context_notes, &[]);

    let search_metadata = SearchMetadata {
        query: query.to_string(),
        vector_search_time_ms: search_time,
        llm_generation_time_ms: generation_time,
        total_tokens,
        retrieved_count: context_notes.len(),
        filtered_count: 0,
        hybrid_search: rag_config.hybrid_search,
        model: model.name().to_string(),
    };

    debug!("[handle_search_mode::DEBUG] Returning search response");
    Ok(AxumJson(ChatResponse {
        response,
        context_notes,
        references,
        search_metadata,
        model: model.name().to_string(),
    }))
}

async fn call_llm_api_with_context(
    query: &str,
    context: &[String],
    user_profile: &UserProfile,
) -> Result<String, (StatusCode, String)> {
    debug!("[call_llm_api_with_context::DEBUG] Starting LLM API call with context");
    debug!("[call_llm_api_with_context::DEBUG] Query: {}", query);
    debug!(
        "[call_llm_api_with_context::DEBUG] Context chunks: {}",
        context.len()
    );
    for (i, chunk) in context.iter().enumerate() {
        debug!(
            "[call_llm_api_with_context::DEBUG] Context chunk {}: {} chars",
            i,
            chunk.len()
        );
        debug!(
            "[call_llm_api_with_context::DEBUG] Context chunk {}: {}...",
            i,
            &chunk[..50.min(chunk.len())]
        );
    }
    debug!("[call_llm_api_with_context::DEBUG] JSON query to be sent to LLM:");
    debug!("[call_llm_api_with_context::DEBUG]   {{");
    debug!("[call_llm_api_with_context::DEBUG]     \"query\": \"{}\",", query);
    debug!("[call_llm_api_with_context::DEBUG]     \"context\": [",);
    for (i, chunk) in context.iter().enumerate() {
        let preview = if chunk.len() > 100 {
            format!("{}...", &chunk[..100])
        } else {
            chunk.clone()
        };
        debug!("[call_llm_api_with_context::DEBUG]       \"{}\",", preview);
    }
    debug!("[call_llm_api_with_context::DEBUG]     ]");
    debug!("[call_llm_api_with_context::DEBUG]   }}");

    let settings = build_llm_settings_from_profile(user_profile);
    debug!("[call_llm_api_with_context::DEBUG] LLM settings built");

    debug!("[call_llm_api_with_context::DEBUG] Testing LLM connection");
    if let Err(e) = test_llm_connection(&settings).await {
        error!("LLM connection test failed: {}", e);
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("LLM service unavailable: {}", e),
        ));
    }
    debug!("[call_llm_api_with_context::DEBUG] LLM connection test passed");

    debug!("[call_llm_api_with_context::DEBUG] Calling generate_llm_response");
    generate_llm_response(&settings, query, context)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM API error: {}", e),
            )
        })
}

async fn handle_chat_mode(
    query: &str,
    context_note_ids: Option<Vec<u64>>,
    history: Option<Vec<ChatMessageHistory>>,
    model: &Arc<dyn EmbeddingModel + Send + Sync>,
    state: &Arc<AppState>,
    user_profile: &UserProfile,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    debug!("[handle_chat_mode::DEBUG] Starting chat mode handling");
    debug!("[handle_chat_mode::DEBUG] Query: {}", query);
    debug!("[handle_chat_mode::DEBUG] Context note IDs: {:?}", context_note_ids);
    debug!("[handle_chat_mode::DEBUG] History length: {:?}", history.as_ref().map(|h| h.len()));

    debug!("[handle_chat_mode::DEBUG] Getting notes by IDs");
    let context_notes = if let Some(note_ids) = context_note_ids {
        debug!("[handle_chat_mode::DEBUG] Fetching {} notes by IDs", note_ids.len());
        get_notes_by_ids(note_ids, state).await?
    } else {
        debug!("[handle_chat_mode::DEBUG] No context note IDs provided");
        Vec::new()
    };
    debug!(
        "[handle_chat_mode::DEBUG] Context notes retrieved: {}",
        context_notes.len()
    );

    debug!("[handle_chat_mode::DEBUG] Calling LLM with history");
    debug!("[handle_chat_mode::DEBUG] Context notes for chat mode:");
    for (i, note) in context_notes.iter().enumerate() {
        debug!(
            "[handle_chat_mode::DEBUG]   Note {}: id={}, title={}, score={:.4}",
            i, note.note_id, note.title, note.score
        );
        debug!(
            "[handle_chat_mode::DEBUG]   Note {} content preview: {} chars",
            i,
            note.content.len()
        );
    }
    let response = call_llm_api_with_history(query, &context_notes, user_profile, history).await?;
    debug!("[handle_chat_mode::DEBUG] LLM response generated");
    debug!("[handle_chat_mode::DEBUG] LLM response: {} chars", response.len());

    let response_clone = response.clone();
    let context_notes_clone = context_notes.clone();

    debug!("[handle_chat_mode::DEBUG] Building references");
    let references = build_references_from_notes(&context_notes, &[]);

    debug!("[handle_chat_mode::DEBUG] Returning chat response");
    Ok(AxumJson(ChatResponse {
        response,
        context_notes,
        references,
        search_metadata: SearchMetadata {
            query: query.to_string(),
            vector_search_time_ms: 0,
            llm_generation_time_ms: 0,
            total_tokens: response_clone.split_whitespace().count(),
            retrieved_count: context_notes_clone.len(),
            filtered_count: 0,
            hybrid_search: false,
            model: model.name().to_string(),
        },
        model: model.name().to_string(),
    }))
}

async fn get_notes_by_ids(
    note_ids: Vec<u64>,
    state: &Arc<AppState>,
) -> Result<Vec<NoteReference>, (StatusCode, String)> {
    debug!("[get_notes_by_ids::DEBUG] Fetching {} notes by IDs", note_ids.len());
    debug!("[get_notes_by_ids::DEBUG] Note IDs: {:?}", note_ids);

    let db = state.db.lock().await;
    debug!("[get_notes_by_ids::DEBUG] Database lock acquired");

    let mut notes = Vec::new();
    for note_id in note_ids {
        debug!("[get_notes_by_ids::DEBUG] Fetching note: {}", note_id);
        let note_id_str = note_id.to_string();
        if let Ok(note) = db.get_by_id(&note_id_str).await {
            debug!("[get_notes_by_ids::DEBUG] Note fetched: {}", note_id);
            notes.push(NoteReference {
                id: note_id,
                note_id: note.id,
                title: note.title,
                content: note.content,
                score: 1.0,
                distance: 0.0,
                user_id: note.user_id,
                created_at: Some(note.created_at.to_rfc3339()),
                updated_at: Some(note.updated_at.to_rfc3339()),
                tags: note.tags,
            });
        } else {
            debug!("[get_notes_by_ids::DEBUG] Note not found: {}", note_id);
        }
    }

    debug!("[get_notes_by_ids::DEBUG] Returning {} notes", notes.len());
    Ok(notes)
}

async fn perform_rag_search(
    query: &str,
    embedding: &[f32],
    state: &Arc<AppState>,
    user_profile: &UserProfile,
    rag_config: &RagConfig,
) -> Result<(Vec<NoteReference>, u64), (StatusCode, String)> {
    debug!(
        "perform_rag_search called with user_id: {}",
        user_profile.id
    );
    debug!("[perform_rag_search::DEBUG] Starting RAG search");
    debug!("[perform_rag_search::DEBUG] Embedding dimension: {}", embedding.len());
    debug!(
        "[perform_rag_search::DEBUG] First 5 embedding values: {:?}",
        &embedding[0..5.min(embedding.len())]
    );
    debug!("[perform_rag_search::DEBUG] Top K: {}", rag_config.top_k);
    debug!("[perform_rag_search::DEBUG] Relevance threshold: {}", rag_config.relevance_threshold);
    debug!("[perform_rag_search::DEBUG] Qdrant vector search threshold documentation:");
    debug!("[perform_rag_search::DEBUG]   - Threshold is cosine similarity score (0.0 to 1.0)");
    debug!("[perform_rag_search::DEBUG]   - 1.0 = perfect match, 0.0 = no similarity");
    debug!("[perform_rag_search::DEBUG]   - Only notes with similarity >= threshold are returned");
    debug!("[perform_rag_search::DEBUG]   - Distance is calculated as: distance = 1.0 - similarity");
    debug!("[perform_rag_search::DEBUG]   - Common thresholds: 0.0 (all), 0.5 (moderate), 0.7 (high), 0.9 (very high)");

    let config = &state.config;

    if !config.vector.enabled {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            "Vector database not enabled. Configure Qdrant or Pinecone.".to_string(),
        ));
    }

    debug!("[perform_rag_search::DEBUG] Acquiring database lock");
    let db = state.db.lock().await;
    debug!("[perform_rag_search::DEBUG] Database lock acquired");

    let vector_store = match &db.vector_store {
        Some(vs) => vs,
        None => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Vector store not initialized".to_string(),
            ))
        }
    };
    debug!("[perform_rag_search::DEBUG] Vector store obtained");

    let start_time = Instant::now();
    debug!("[perform_rag_search::DEBUG] Calling vector store search");
    debug!("[perform_rag_search::DEBUG] Hybrid search enabled: {}", rag_config.hybrid_search);
    debug!("[perform_rag_search::DEBUG] Vector weight: {}", rag_config.hybrid_weight);
    debug!("[perform_rag_search::DEBUG] Filter fields: {:?}", rag_config.filter_fields);

    let results = if rag_config.hybrid_search {
        debug!("[perform_rag_search::DEBUG] Using BM25 hybrid search");
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            vector_store.hybrid_search_with_bm25(
                embedding.to_vec(),
                query,
                &user_profile.id,
                rag_config.top_k as u32,
                rag_config.hybrid_weight,
            ),
        )
        .await
        {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                error!(
                    "[perform_rag_search::ERROR] BM25 hybrid search failed: {}",
                    e
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to BM25 hybrid search: {}", e),
                ));
            }
            Err(_) => {
                error!("[perform_rag_search::ERROR] BM25 hybrid search timed out");
                return Ok((Vec::new(), 0));
            }
        }
    } else {
        debug!("[perform_rag_search::DEBUG] Using standard vector search");
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            vector_store.search_notes_with_scores(
                embedding.to_vec(),
                &user_profile.id,
                rag_config.top_k as u32,
            ),
        )
        .await
        {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                error!(
                    "[perform_rag_search::ERROR] Vector store search failed: {}",
                    e
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to search: {}", e),
                ));
            }
            Err(_) => {
                error!("[perform_rag_search::ERROR] Vector store search timed out");
                return Ok((Vec::new(), 0));
            }
        }
    };
    debug!("[perform_rag_search::DEBUG] Vector store search complete");
    debug!("[perform_rag_search::DEBUG] Hybrid search configuration: {} notes to retrieve with top_k={}", results.len(), rag_config.top_k);

    let search_time = start_time.elapsed().as_millis() as u64;
    debug!(
        "[perform_rag_search::DEBUG] Got {} results from vector search in {}ms",
        results.len(),
        search_time
    );

    debug!("[perform_rag_search::DEBUG] Converting results to NoteReferences");
    let notes: Vec<NoteReference> = results
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let default_payload = serde_json::Value::Object(serde_json::Map::new());
            let payload = r.payload.as_ref().unwrap_or(&default_payload);
            let distance = 1.0 - r.score; // Convert similarity to distance
            let note_id = payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&r.id)
                .to_string();

            NoteReference {
                id: (i + 1) as u64,
                note_id,
                title: payload
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                content: payload
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: r.score,
                distance,
                user_id: payload
                    .get("user_id")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string()),
                created_at: payload
                    .get("created_at")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string()),
                updated_at: payload
                    .get("updated_at")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string()),
                tags: payload
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        })
        .filter(|n| n.score >= rag_config.relevance_threshold)
        .collect();
    debug!("[perform_rag_search::DEBUG] Results converted to NoteReferences");

    debug!(
        "[perform_rag_search::DEBUG] {} notes passed relevance threshold",
        notes.len()
    );

    debug!("[perform_rag_search::DEBUG] Hybrid search vector context notes returned:");
    for (i, note) in notes.iter().enumerate() {
        debug!(
            "[perform_rag_search::DEBUG]   Context Note {}: id={}, title={}, score={:.4}, distance={:.4}",
            i, note.note_id, note.title, note.score, note.distance
        );
        debug!(
            "[perform_rag_search::DEBUG]   Context Note {}: content preview: {} chars",
            i,
            note.content.len()
        );
    }

    Ok((notes, search_time))
}

fn generate_search_response(query: &str, context_notes: &[NoteReference]) -> String {
    if context_notes.is_empty() {
        format!(
            "I couldn't find any relevant notes for your query.\n\nQuery: {}",
            query
        )
    } else {
        let context_text: Vec<String> = context_notes
            .iter()
            .map(|n| format!("Note: {} (Score: {:.3})\n{}", n.title, n.score, n.content))
            .collect();

        let context_str = context_text.join("\n\n");

        format!(
            "Based on the following notes:\n\n{}\n\nQuery: {}\n\nResponse: I can help with that based on the context provided.",
            context_str,
            query
        )
    }
}

fn build_references_from_notes(notes: &[NoteReference], used_indices: &[usize]) -> Vec<Reference> {
    debug!("[build_references_from_notes::DEBUG] Building references for {} notes", notes.len());
    let mut references = Vec::new();

    for (idx, note) in notes.iter().enumerate() {
        debug!(
            "[build_references_from_notes::DEBUG] Building reference {}: id={}, score={}",
            idx, note.note_id, note.score
        );
        let content_snippet = if note.content.len() > 200 {
            format!("{}...", &note.content[..200])
        } else {
            note.content.clone()
        };

        references.push(Reference {
            id: note.id,
            note_id: note.note_id.clone(),
            title: note.title.clone(),
            content_snippet,
            score: note.score,
            distance: note.distance,
            used_in_response: used_indices.contains(&idx),
            created_at: note.created_at.clone(),
            updated_at: note.updated_at.clone(),
            tags: note.tags.clone(),
        });
    }

    debug!("[build_references_from_notes::DEBUG] Created {} references", references.len());
    references
}

fn build_dynamic_context(notes: &[NoteReference], max_tokens: usize) -> Vec<String> {
    debug!(
        "[build_dynamic_context::DEBUG] Building context with max_tokens={}",
        max_tokens
    );
    debug!(
        "[build_dynamic_context::DEBUG] Input notes: {}",
        notes.len()
    );
    let mut context = Vec::new();
    let mut total_tokens = 0;

    for note in notes {
        debug!(
            "[build_dynamic_context::DEBUG] Processing note: {}",
            note.note_id
        );
        let note_text = format!("Title: {}\nContent: {}", note.title, note.content);
        let token_count = note_text.split_whitespace().count();
        debug!(
            "[build_dynamic_context::DEBUG] Note token count: {}",
            token_count
        );

        if total_tokens + token_count <= max_tokens {
            debug!("[build_dynamic_context::DEBUG] Adding note to context");
            context.push(note_text);
            total_tokens += token_count;
            debug!(
                "[build_dynamic_context::DEBUG] Total tokens so far: {}",
                total_tokens
            );
        } else {
            debug!("[build_dynamic_context::DEBUG] Skipping note (would exceed max_tokens)");
            break;
        }
    }

    debug!(
        "[build_dynamic_context::DEBUG] Final context size: {} chunks, {} total tokens",
        context.len(),
        total_tokens
    );
    context
}

pub async fn call_llm_api(
    query: &str,
    context_notes: &[NoteReference],
    user_profile: &UserProfile,
) -> Result<String, (StatusCode, String)> {
    let context: Vec<String> = context_notes.iter().map(|n| n.content.clone()).collect();

    let settings = build_llm_settings_from_profile(user_profile);

    if let Err(e) = test_llm_connection(&settings).await {
        error!("LLM connection test failed: {}", e);
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("LLM service unavailable: {}", e),
        ));
    }

    generate_llm_response(&settings, query, &context)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM API error: {}", e),
            )
        })
}

async fn call_llm_api_with_history(
    query: &str,
    context_notes: &[NoteReference],
    user_profile: &UserProfile,
    history: Option<Vec<ChatMessageHistory>>,
) -> Result<String, (StatusCode, String)> {
    debug!("[call_llm_api_with_history::DEBUG] Starting LLM API call with history");
    debug!("[call_llm_api_with_history::DEBUG] Query: {}", query);
    debug!(
        "[call_llm_api_with_history::DEBUG] Context notes: {}",
        context_notes.len()
    );
    for (i, note) in context_notes.iter().enumerate() {
        debug!(
            "[call_llm_api_with_history::DEBUG] Context note {}: id={}, title={}",
            i, note.note_id, note.title
        );
    }
    debug!("[call_llm_api_with_history::DEBUG] History length: {:?}", history.as_ref().map(|h| h.len()));

    let context: Vec<String> = context_notes.iter().map(|n| n.content.clone()).collect();

    let settings = build_llm_settings_from_profile(user_profile);
    debug!("[call_llm_api_with_history::DEBUG] LLM settings built");

    debug!("[call_llm_api_with_history::DEBUG] Testing LLM connection");
    if let Err(e) = test_llm_connection(&settings).await {
        error!("LLM connection test failed: {}", e);
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("LLM service unavailable: {}", e),
        ));
    }
    debug!("[call_llm_api_with_history::DEBUG] LLM connection test passed");

    debug!("[call_llm_api_with_history::DEBUG] Calling generate_llm_response_with_history");
    generate_llm_response_with_history(&settings, query, &context, history)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM API error: {}", e),
            )
        })
}

fn build_llm_settings_from_profile(
    user_profile: &UserProfile,
) -> crate::models::auth_dto::LLMSettings {
    let settings = &user_profile.llm_settings;

    crate::models::auth_dto::LLMSettings {
        provider: settings
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openai")
            .to_string(),
        url: settings
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:11434/v1")
            .to_string(),
        api_key: settings
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        model: settings
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("llama3")
            .to_string(),
        temperature: settings
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7),
        max_tokens: settings
            .get("max_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(2048) as i32,
    }
}

async fn search_notes(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SearchRequest>,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    debug!("[search_notes::DEBUG] Starting search_notes endpoint");
    debug!("[search_notes::DEBUG] Query: {}", req.query);
    debug!("[search_notes::DEBUG] Limit: {:?}", req.limit);

    if req.query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query cannot be empty".to_string()));
    }

    debug!("[search_notes::DEBUG] Getting user profile");
    let model: &Arc<dyn EmbeddingModel + Send + Sync> = &state.embedding_model;
    let user_profile = get_user_profile(&headers, &state).await?;
    debug!("[search_notes::DEBUG] User profile obtained: {}", user_profile.id);

    debug!("[search_notes::DEBUG] Computing embedding for query");
    let mut embedding = model.embed(&req.query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to embed query: {}", e),
        )
    })?;
    debug!("[search_notes::DEBUG] Embedding computed, dimension: {}", embedding.len());
    debug!(
        "[search_notes::DEBUG] First 5 embedding values: {:?}",
        &embedding[0..5.min(embedding.len())]
    );

    debug!("[search_notes::DEBUG] Normalizing embedding vector");
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();
    debug!("[search_notes::DEBUG] Embedding normalized, norm: {}", norm);

    debug!("[search_notes::DEBUG] Performing RAG search");
    let context_notes = perform_rag_search(&req.query, &embedding, &state, &user_profile, &state.rag_config)
        .await?
        .0;
    debug!(
        "[search_notes::DEBUG] RAG search complete, {} notes retrieved",
        context_notes.len()
    );

    let context_notes_clone = context_notes.clone();

    debug!("[search_notes::DEBUG] Generating search response");
    let response = generate_search_response(&req.query, &context_notes);

    debug!("[search_notes::DEBUG] Returning search response");
    Ok(AxumJson(ChatResponse {
        response,
        context_notes,
        references: Vec::new(),
        search_metadata: SearchMetadata {
            query: req.query,
            vector_search_time_ms: 0,
            llm_generation_time_ms: 0,
            total_tokens: 0,
            retrieved_count: context_notes_clone.len(),
            filtered_count: 0,
            hybrid_search: false,
            model: model.name().to_string(),
        },
        model: model.name().to_string(),
    }))
}

async fn rag_search_notes(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RagSearchRequest>,
) -> Result<AxumJson<ChatResponse>, (StatusCode, String)> {
    debug!("[rag_search_notes::DEBUG] Starting rag_search_notes endpoint");
    debug!("[rag_search_notes::DEBUG] Query: {}", req.query);
    debug!("[rag_search_notes::DEBUG] Limit: {:?}", req.limit);
    debug!("[rag_search_notes::DEBUG] Mode: {:?}", req.mode);
    debug!("[rag_search_notes::DEBUG] Context note IDs: {:?}", req.context_note_ids);
    debug!("[rag_search_notes::DEBUG] Max context tokens: {:?}", req.max_context_tokens);
    debug!("[rag_search_notes::DEBUG] Use hybrid search: {:?}", req.use_hybrid_search);
    debug!("[rag_search_notes::DEBUG] Stream: {:?}", req.stream);

    if req.query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query cannot be empty".to_string()));
    }

    debug!("[rag_search_notes::DEBUG] Getting user profile");
    let model: &Arc<dyn EmbeddingModel + Send + Sync> = &state.embedding_model;
    let user_profile = get_user_profile(&headers, &state).await?;
    debug!("[rag_search_notes::DEBUG] User profile obtained: {}", user_profile.id);

    debug!("[rag_search_notes::DEBUG] Computing embedding for query");
    let mut embedding = model.embed(&req.query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to embed query: {}", e),
        )
    })?;
    debug!("[rag_search_notes::DEBUG] Embedding computed, dimension: {}", embedding.len());
    debug!(
        "[rag_search_notes::DEBUG] First 5 embedding values: {:?}",
        &embedding[0..5.min(embedding.len())]
    );

    debug!("[rag_search_notes::DEBUG] Normalizing embedding vector");
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();
    debug!("[rag_search_notes::DEBUG] Embedding normalized, norm: {}", norm);

    debug!("[rag_search_notes::DEBUG] Performing RAG search");
    let context_notes = perform_rag_search(&req.query, &embedding, &state, &user_profile, &state.rag_config)
        .await?
        .0;
    debug!(
        "[rag_search_notes::DEBUG] RAG search complete, {} notes retrieved",
        context_notes.len()
    );

    debug!("[rag_search_notes::DEBUG] Building dynamic context");
    let context: Vec<String> = build_dynamic_context(
        &context_notes,
        req.max_context_tokens
            .unwrap_or(state.rag_config.max_context_tokens),
    );
    debug!("[rag_search_notes::DEBUG] Context built with {} chunks", context.len());

    debug!("[rag_search_notes::DEBUG] Calling LLM with context");
    debug!("[rag_search_notes::DEBUG] Context notes returned from vector search:");
    for (i, note) in context_notes.iter().enumerate() {
        debug!(
            "[rag_search_notes::DEBUG]   Note {}: id={}, title={}, score={:.4}, distance={:.4}",
            i, note.note_id, note.title, note.score, note.distance
        );
        debug!(
            "[rag_search_notes::DEBUG]   Note {} content preview: {} chars",
            i,
            note.content.len()
        );
    }
    let start_time = Instant::now();
    let response = if context.is_empty() {
        format!(
            "I couldn't find any relevant notes for your query.\n\nQuery: {}",
            req.query
        )
    } else {
        debug!("[rag_search_notes::DEBUG] Context not empty, calling LLM");
        debug!("[rag_search_notes::DEBUG] Context chunks to be sent to LLM:");
        for (i, chunk) in context.iter().enumerate() {
            debug!("[rag_search_notes::DEBUG]   Chunk {}: {} chars", i, chunk.len());
            debug!("[rag_search_notes::DEBUG]   Chunk {}: {}...", i, &chunk[..50.min(chunk.len())]);
        }
        debug!("[rag_search_notes::DEBUG] LLM query JSON:");
        debug!("[rag_search_notes::DEBUG]   Query: {}", req.query);
        debug!("[rag_search_notes::DEBUG]   Context size: {} chunks", context.len());
        match call_llm_api_with_context(&req.query, &context, &user_profile).await {
            Ok(resp) => resp,
            Err(_) => generate_search_response(&req.query, &context_notes),
        }
    };
    debug!("[rag_search_notes::DEBUG] LLM response generated");
    debug!("[rag_search_notes::DEBUG] LLM response: {} chars", response.len());

    let generation_time = start_time.elapsed().as_millis() as u64;
    let total_tokens = response.split_whitespace().count();

    debug!("[rag_search_notes::DEBUG] Building references");
    let references = build_references_from_notes(&context_notes, &[]);

    let search_metadata = SearchMetadata {
        query: req.query,
        vector_search_time_ms: 0,
        llm_generation_time_ms: generation_time,
        total_tokens,
        retrieved_count: context_notes.len(),
        filtered_count: 0,
        hybrid_search: req
            .use_hybrid_search
            .unwrap_or(state.rag_config.hybrid_search),
        model: model.name().to_string(),
    };

    debug!("[rag_search_notes::DEBUG] Returning RAG search response");
    Ok(AxumJson(ChatResponse {
        response,
        context_notes,
        references,
        search_metadata,
        model: model.name().to_string(),
    }))
}
