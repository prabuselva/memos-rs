use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::vector::bm25::BM25;

#[derive(Debug)]
pub struct VectorStore {
    client: Client,
    url: String,
}

#[derive(Debug, Serialize)]
struct CreateCollectionRequest {
    vectors: VectorsConfig,
}

#[derive(Debug, Serialize)]
struct VectorsConfig {
    size: usize,
    distance: String,
}

#[derive(Debug, Serialize)]
struct UpsertPoint {
    id: String,
    vector: Vec<f32>,
    payload: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct UpsertRequest {
    points: Vec<UpsertPoint>,
}

#[derive(Debug, Serialize)]
struct QueryRequest {
    vector: Vec<f32>,
    limit: u64,
    filter: Option<Filter>,
    with_payload: bool,
    with_vector: bool,
}

#[derive(Debug, Serialize)]
struct Filter {
    must: Vec<Condition>,
}

#[derive(Debug, Serialize)]
struct Condition {
    key: String,
    r#match: Match,
}

#[derive(Debug, Serialize)]
struct Match {
    value: String,
}

#[derive(Debug, Deserialize)]
struct Point {
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct QueryResponse {
    result: QueryResult,
}

#[derive(Debug, Deserialize)]
struct QueryResult {
    points: Vec<Point>,
}

#[derive(Debug, Deserialize)]
struct FetchPoint {
    id: String,
    payload: Option<serde_json::Value>,
    vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoredPoint {
    pub id: String,
    pub version: u64,
    pub score: f32,
    pub payload: Option<serde_json::Value>,
    pub vector: Option<Vec<f32>>,
}

impl VectorStore {
    pub async fn new(url: &str) -> Result<Self, anyhow::Error> {
        info!("Connecting to Qdrant at {}", url);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let store = Self {
            client,
            url: url.to_string(),
        };

        Ok(store)
    }

    fn collection_name_for_user(user_id: &str) -> String {
        let sanitized: String = user_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("notes_{}", sanitized)
    }

    async fn get_collection_for_user(&self, user_id: &str) -> String {
        Self::collection_name_for_user(user_id)
    }

    async fn create_collection_if_not_exists(
        &self,
        collection_name: &str,
        size: usize,
        distance: &str,
    ) -> Result<(), anyhow::Error> {
        let collections_url = format!("{}/collections", self.url);

        let response = self.client.get(&collections_url).send().await?;
        if response.status().is_success() {
            let collections: serde_json::Value = response.json().await?;
            if let Some(collections_array) =
                collections.get("result").and_then(|r| r.get("collections"))
            {
                if let Some(collections_array) = collections_array.as_array() {
                    for collection in collections_array {
                        if let Some(name) = collection.get("name").and_then(|n| n.as_str()) {
                            if name == collection_name {
                                debug!("Using existing vector collection: {}", collection_name);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        let create_url = format!("{}/collections/{}", self.url, collection_name);
        let request = CreateCollectionRequest {
            vectors: VectorsConfig {
                size,
                distance: distance.to_string(),
            },
        };

        let response = self.client.put(&create_url).json(&request).send().await?;

        if response.status().is_success() {
            info!("Created vector collection: {}", collection_name);
        } else {
            error!("Failed to create collection: {:?}", response.text().await?);
        }

        Ok(())
    }

    pub async fn upsert_note(
        &self,
        user_id: &str,
        note_id: &str,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        debug!("[Qdrant::upsert_note::DEBUG] Starting upsert for note_id={}", note_id);
        let collection = self.get_collection_for_user(user_id).await;
        debug!("[Qdrant::upsert_note::DEBUG] Collection: {}", collection);
        self.create_collection_if_not_exists(&collection, 384, "Cosine")
            .await?;

        let payload_map: HashMap<String, serde_json::Value> = match payload {
            serde_json::Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        };

        debug!("[Qdrant::upsert_note::DEBUG] Vector dimension: {}", vector.len());
        debug!(
            "[Qdrant::upsert_note::DEBUG] First 5 vector values: {:?}",
            &vector[0..5.min(vector.len())]
        );

        let upsert_url = format!("{}/collections/{}/points", self.url, collection);
        let request = UpsertRequest {
            points: vec![UpsertPoint {
                id: note_id.to_string(),
                vector,
                payload: payload_map,
            }],
        };

        debug!("[Qdrant::upsert_note::DEBUG] Sending upsert request to Qdrant");
        let response = self
            .client
            .put(&format!("{}?wait=true", upsert_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Failed to upsert: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to upsert note"));
        }

        debug!("[Qdrant::upsert_note::DEBUG] Upsert successful");
        Ok(())
    }

    pub async fn search_notes(
        &self,
        vector: Vec<f32>,
        user_id: &str,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        debug!("[Qdrant::search_notes::DEBUG] Starting search_notes query");
        debug!("[Qdrant::search_notes::DEBUG] Vector dimension: {}", vector.len());
        debug!(
            "[Qdrant::search_notes::DEBUG] First 5 vector values: {:?}",
            &vector[0..5.min(vector.len())]
        );
        debug!("[Qdrant::search_notes::DEBUG] User ID: {}", user_id);
        debug!("[Qdrant::search_notes::DEBUG] Limit: {}", limit);

        let collection = self.get_collection_for_user(user_id).await;
        debug!("[Qdrant::search_notes::DEBUG] Collection: {}", collection);
        let query_url = format!("{}/collections/{}/points/query", self.url, collection);
        let request = QueryRequest {
            vector,
            limit: limit as u64,
            filter: None,
            with_payload: true,
            with_vector: false,
        };

        debug!("[Qdrant::search_notes::DEBUG] Sending search request to Qdrant");
        let response = self.client.post(&query_url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to query: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to query notes"));
        }

        debug!("[Qdrant::search_notes::DEBUG] Received response from Qdrant");
        let result: QueryResponse = response.json().await?;
        let results: Vec<serde_json::Value> = result
            .result
            .points
            .into_iter()
            .filter_map(|p| p.payload)
            .collect();

        debug!("[Qdrant::search_notes::DEBUG] Search complete, {} results", results.len());
        Ok(results)
    }

    pub async fn search_notes_with_scores(
        &self,
        vector: Vec<f32>,
        user_id: &str,
        limit: u32,
    ) -> Result<Vec<ScoredPoint>, anyhow::Error> {
        debug!("[Qdrant::search_notes_with_scores::DEBUG] Starting search_notes_with_scores");
        debug!("[Qdrant::search_notes_with_scores::DEBUG] Vector dimension: {}", vector.len());
        debug!(
            "[Qdrant::search_notes_with_scores::DEBUG] First 5 vector values: {:?}",
            &vector[0..5.min(vector.len())]
        );
        debug!("[Qdrant::search_notes_with_scores::DEBUG] User ID: {}", user_id);
        debug!("[Qdrant::search_notes_with_scores::DEBUG] Limit: {}", limit);

        let collection = self.get_collection_for_user(user_id).await;
        debug!("[Qdrant::search_notes_with_scores::DEBUG] Collection: {}", collection);
        let query_url = format!("{}/collections/{}/points/search", self.url, collection);
        
        let request = serde_json::json!({
            "vector": vector,
            "limit": limit as u64,
            "with_payload": true,
            "with_vector": false,
            "filter": {
                "must": [
                    {
                        "key": "user_id",
                        "match": {
                            "value": user_id
                        }
                    }
                ]
            }
        });

        debug!("[Qdrant::search_notes_with_scores::DEBUG] Sending search request to Qdrant");
        debug!("[Qdrant::search_notes_with_scores::DEBUG] Search request JSON: {}", serde_json::to_string(&request)?);
        let response = self.client.post(&query_url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to search: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to search notes"));
        }

        debug!("[Qdrant::search_notes_with_scores::DEBUG] Received response from Qdrant");
        let search_result: serde_json::Value = response.json().await?;

        if let Some(results) = search_result.get("result").and_then(|r| r.as_array()) {
            debug!("[Qdrant::search_notes_with_scores::DEBUG] Parsing {} results", results.len());
            let points: Vec<ScoredPoint> = results
                .iter()
                .map(|p| {
                    serde_json::from_value(p.clone()).unwrap_or_else(|_| ScoredPoint {
                        id: String::new(),
                        version: 0,
                        score: 0.0,
                        payload: None,
                        vector: None,
                    })
                })
                .collect();
            debug!("[Qdrant::search_notes_with_scores::DEBUG] Parsed {} points", points.len());
            for (i, point) in points.iter().enumerate() {
                debug!(
                    "[Qdrant::search_notes_with_scores::DEBUG] Result {}: id={}, score={:.4}",
                    i, point.id, point.score
                );
            }
            return Ok(points);
        }

        debug!("[Qdrant::search_notes_with_scores::DEBUG] No results found");
        Ok(Vec::new())
    }

    pub async fn hybrid_search_with_scores(
        &self,
        vector: Vec<f32>,
        user_id: &str,
        limit: u32,
        filter_fields: &HashMap<String, String>,
        vector_weight: f32,
    ) -> Result<Vec<ScoredPoint>, anyhow::Error> {
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Starting hybrid search");
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Vector dimension: {}", vector.len());
        debug!(
            "[Qdrant::hybrid_search_with_scores::DEBUG] First 5 vector values: {:?}",
            &vector[0..5.min(vector.len())]
        );
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] User ID: {}", user_id);
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Limit: {}", limit);
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Vector weight: {}", vector_weight);
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Filter fields: {:?}", filter_fields);

        let collection = self.get_collection_for_user(user_id).await;
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Collection: {}", collection);
        
        // Qdrant hybrid search uses the /points/query endpoint
        // For true hybrid search with weighted fusion, we need at least 2 prefetches
        // Since we only have one vector, we use a single prefetch with the vector query
        // and apply RRF with a single weight (which works in v1.17.0+)
        let query_url = format!("{}/collections/{}/points/query", self.url, collection);

        // Build filter conditions
        let mut must_conditions: Vec<serde_json::Value> = Vec::new();
        must_conditions.push(serde_json::json!({
            "key": "user_id",
            "match": {
                "value": user_id
            }
        }));

        for (field, value) in filter_fields {
            if field != "user_id" {
                must_conditions.push(serde_json::json!({
                    "key": field,
                    "match": {
                        "value": value
                    }
                }));
            }
        }

        let filter = if must_conditions.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::json!({
                "must": must_conditions
            })
        };

        // For single prefetch, use nearest query with filter
        // The vector_weight is used as score_threshold to filter results
        // With single prefetch, we can use score_threshold on the nearest query
        let request = serde_json::json!({
            "query": {
                "nearest": vector
            },
            "limit": limit as u64,
            "filter": filter,
            "score_threshold": vector_weight
        });

        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Hybrid search request JSON: {}", serde_json::to_string(&request)?);
        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Using RRF with weights: [{}]", vector_weight);
        let response = self.client.post(&query_url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to hybrid search: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to hybrid search"));
        }

        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Received response from Qdrant");
        let search_result: serde_json::Value = response.json().await?;

        if let Some(results) = search_result.get("result").and_then(|r| r.as_array()) {
            debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Parsing {} hybrid search results", results.len());
            let points: Vec<ScoredPoint> = results
                .iter()
                .map(|p| {
                    serde_json::from_value(p.clone()).unwrap_or_else(|_| ScoredPoint {
                        id: String::new(),
                        version: 0,
                        score: 0.0,
                        payload: None,
                        vector: None,
                    })
                })
                .collect();
            debug!("[Qdrant::hybrid_search_with_scores::DEBUG] Parsed {} hybrid search points", points.len());
            for (i, point) in points.iter().enumerate() {
                debug!(
                    "[Qdrant::hybrid_search_with_scores::DEBUG] Result {}: id={}, score={:.4}",
                    i, point.id, point.score
                );
            }
            return Ok(points);
        }

        debug!("[Qdrant::hybrid_search_with_scores::DEBUG] No hybrid search results found");
        Ok(Vec::new())
    }

    pub async fn delete_note(&self, user_id: &str, note_id: &str) -> Result<(), anyhow::Error> {
        let collection = self.get_collection_for_user(user_id).await;
        let delete_url = format!("{}/collections/{}/points/delete", self.url, collection);

        let request = serde_json::json!({
            "points": [note_id]
        });

        let response = self.client.post(&delete_url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to delete: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to delete note"));
        }

        Ok(())
    }

    pub async fn delete_user_data(&self, user_id: &str) -> Result<(), anyhow::Error> {
        let collection = self.get_collection_for_user(user_id).await;
        let delete_url = format!("{}/collections/{}", self.url, collection);

        let response = self.client.delete(&delete_url).send().await?;

        if response.status().is_success() {
            info!("Deleted collection for user: {}", user_id);
            Ok(())
        } else {
            error!("Failed to delete collection for user {}: {:?}", user_id, response.text().await?);
            Err(anyhow::anyhow!("Failed to delete user data"))
        }
    }

    const BM25_META_ID: &str = "00000000-0000-0000-0000-000000000001";

    pub async fn get_bm25_state(&self, user_id: &str) -> Result<Option<BM25>, anyhow::Error> {
        let collection = self.get_collection_for_user(user_id).await;
        let point_url = format!(
            "{}/collections/{}/points/{}?with_payload=true",
            self.url, collection, Self::BM25_META_ID
        );

        let response = self.client.get(&point_url).send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            if let Some(payload) = result.get("result").and_then(|r| r.get("payload")) {
                if let Some(bm25_json) = payload.get("bm25_state").and_then(|v| v.as_str()) {
                    let bm25 = BM25::from_dict(&serde_json::from_str(bm25_json)?);
                    return Ok(Some(bm25));
                }
            }
        }

        Ok(None)
    }

    pub async fn save_bm25_state(&self, user_id: &str, bm25: &BM25) -> Result<(), anyhow::Error> {
        let collection = self.get_collection_for_user(user_id).await;
        let state_json = bm25.to_dict();
        let state_str = serde_json::to_string(&state_json)?;

        let point = serde_json::json!({
            "id": Self::BM25_META_ID,
            "vector": vec![0.0; 384],
            "payload": {
                "bm25_state": state_str
            }
        });

        let upsert_url = format!("{}/collections/{}/points?wait=true", self.url, collection);
        let request = serde_json::json!({
            "points": [point]
        });

        let response = self.client.put(&upsert_url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            error!("Failed to save BM25 state: {:?}", response.text().await?);
            Err(anyhow::anyhow!("Failed to save BM25 state"))
        }
    }

    pub async fn hybrid_search_with_bm25(
        &self,
        vector: Vec<f32>,
        query: &str,
        user_id: &str,
        limit: u32,
        vector_weight: f32,
    ) -> Result<Vec<ScoredPoint>, anyhow::Error> {
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Starting BM25 hybrid search");
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Query: {}", query);
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Vector dimension: {}", vector.len());
        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] First 5 vector values: {:?}",
            &vector[0..5.min(vector.len())]
        );
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] User ID: {}", user_id);
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Limit: {}", limit);
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Vector weight: {}", vector_weight);

        let collection = self.get_collection_for_user(user_id).await;
        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] Collection: {}", collection);

        let bm25 = match self.get_bm25_state(user_id).await? {
            Some(bm25) => bm25,
            None => {
                debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] No BM25 state found, creating new instance");
                BM25::new(1.5, 0.75)
            }
        };

        debug!("[Qdrant::hybrid_search_with_bm25::DEBUG] BM25 has {} documents", bm25.len());

        let bm25_scores = bm25.get_scores(query);
        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] BM25 scores computed: {:?}",
            &bm25_scores[0..5.min(bm25_scores.len())]
        );

        let max_bm25 = bm25_scores.iter().fold(0.0f32, |max, &s| if s > max { s } else { max });
        let bm25_scores_normalized: Vec<f32> = if max_bm25 > 0.0 {
            bm25_scores.iter().map(|&s| s / max_bm25).collect()
        } else {
            vec![0.0; bm25_scores.len()]
        };

        let top_bm25_indexes = bm25
            .get_top_results(query, limit as usize)
            .iter()
            .map(|(idx, _)| *idx)
            .collect::<Vec<_>>();

        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] Top BM25 indexes: {:?}",
            top_bm25_indexes
        );

        let top_point_ids: Vec<String> = top_bm25_indexes
            .iter()
            .filter_map(|&idx| bm25.get_point_id(idx).cloned())
            .collect();

        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] Top point IDs: {:?}",
            top_point_ids
        );

        let qdrant_scores = self
            .search_notes_with_scores_for_ids(&vector, user_id, &top_point_ids)
            .await?;

        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] Qdrant scores: {:?}",
            qdrant_scores
        );

        let mut final_scores: HashMap<String, f32> = HashMap::new();

        for point_id in &top_point_ids {
            if let Some(bm25_idx) = bm25.get_index(point_id) {
                let d_score = *qdrant_scores.get(point_id).unwrap_or(&0.0);
                let bm25_score = bm25_scores_normalized[bm25_idx];
                let combined_score = vector_weight * d_score + (1.0 - vector_weight) * bm25_score;
                final_scores.insert(point_id.clone(), combined_score);
            }
        }

        let mut ranked_ids: Vec<(String, f32)> = final_scores
            .iter()
            .map(|(id, score)| (id.clone(), *score))
            .collect();

        ranked_ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let final_point_ids: Vec<String> = ranked_ids
            .iter()
            .map(|(id, _)| id.clone())
            .collect();

        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] Final ranked IDs: {:?}",
            final_point_ids
        );

        let mut results = self
            .fetch_points_by_ids(&collection, &final_point_ids)
            .await?;

        for result in &mut results {
            if let Some(score) = final_scores.get(&result.id) {
                result.score = *score;
            }
        }

        debug!(
            "[Qdrant::hybrid_search_with_bm25::DEBUG] Fetched {} results",
            results.len()
        );

        Ok(results)
    }

    async fn search_notes_with_scores_for_ids(
        &self,
        vector: &[f32],
        user_id: &str,
        point_ids: &[String],
    ) -> Result<HashMap<String, f32>, anyhow::Error> {
        if point_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let collection = self.get_collection_for_user(user_id).await;
        let search_url = format!(
            "{}/collections/{}/points/search",
            self.url, collection
        );

        let mut all_scores: HashMap<String, f32> = HashMap::new();

        for chunk in point_ids.chunks(100) {
            let request = serde_json::json!({
                "vector": vector,
                "limit": chunk.len() as u64,
                "with_payload": false,
                "with_vector": false,
                "filter": {
                    "must": [
                        {
                            "key": "user_id",
                            "match": {
                                "value": user_id
                            }
                        }
                    ]
                }
            });

            let response = self.client.post(&search_url).json(&request).send().await?;

            if response.status().is_success() {
                let search_result: serde_json::Value = response.json().await?;

                if let Some(results) = search_result.get("result").and_then(|r| r.as_array()) {
                    for item in results {
                        if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                            if let Some(score) = item.get("score").and_then(|s| s.as_f64()) {
                                all_scores.insert(id.to_string(), score as f32);
                            }
                        }
                    }
                }
            }
        }

        Ok(all_scores)
    }

    async fn fetch_points_by_ids(
        &self,
        collection: &str,
        point_ids: &[String],
    ) -> Result<Vec<ScoredPoint>, anyhow::Error> {
        if point_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_points: Vec<ScoredPoint> = Vec::new();

        for point_id in point_ids {
            let fetch_url = format!(
                "{}/collections/{}/points/{}?with_payload=true",
                self.url, collection, point_id
            );

            let response = self.client.get(&fetch_url).send().await?;

            if response.status().is_success() {
                let fetch_result: serde_json::Value = response.json().await?;
                debug!("Fetched vector point: {}", fetch_result);
                if let Some(point) = fetch_result.get("result") {
                    if let Ok(fetch_point) = serde_json::from_value::<FetchPoint>(point.clone()) {
                        let scored_point = ScoredPoint {
                            id: fetch_point.id,
                            version: 0,
                            score: 0.0,
                            payload: fetch_point.payload,
                            vector: fetch_point.vector,
                        };
                        all_points.push(scored_point);
                    }
                }
            }
        }

        Ok(all_points)
    }
}

impl Clone for VectorStore {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
        }
    }
}
