use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25 {
    k1: f32,
    b: f32,
    docs: Vec<Vec<String>>,
    tf: Vec<HashMap<String, u32>>,
    df: HashMap<String, u32>,
    doc_lens: Vec<usize>,
    n: usize,
    avgdl: f32,
    index_to_id: HashMap<usize, String>,
}

impl BM25 {
    pub fn new(k1: f32, b: f32) -> Self {
        Self {
            k1,
            b,
            docs: Vec::new(),
            tf: Vec::new(),
            df: HashMap::new(),
            doc_lens: Vec::new(),
            n: 0,
            avgdl: 0.0,
            index_to_id: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, doc: &str, point_id: String) {
        let tokens: Vec<String> = doc
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        self.docs.push(tokens.clone());
        self.index_to_id.insert(self.n, point_id);
        self.n += 1;

        let mut tf_doc = HashMap::new();
        for token in tokens {
            *tf_doc.entry(token).or_insert(0) += 1;
        }
        self.tf.push(tf_doc);

        for word in self.tf[self.n - 1].keys() {
            *self.df.entry(word.clone()).or_insert(0) += 1;
        }

        let doc_len = self.docs[self.n - 1].len();
        self.doc_lens.push(doc_len);
        self.avgdl = self.doc_lens.iter().sum::<usize>() as f32 / self.n as f32;
    }

    pub fn idf(&self, term: &str) -> f32 {
        let df = *self.df.get(term).unwrap_or(&0);
        if df == 0 {
            return 0.0;
        }
        ((self.n as f32 - df as f32 + 0.5) / (df as f32 + 0.5) + 1.0).ln()
    }

    pub fn score(&self, query: &str, index: usize) -> f32 {
        let query_terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut score = 0.0;
        let doc_len = self.doc_lens[index] as f32;

        for term in query_terms {
            if let Some(&f) = self.tf[index].get(&term) {
                let idf = self.idf(&term);
                let denom = f as f32 + self.k1 * (1.0 - self.b + self.b * doc_len / self.avgdl);
                score += idf * (f as f32 * (self.k1 + 1.0)) / denom;
            }
        }

        score
    }

    pub fn get_scores(&self, query: &str) -> Vec<f32> {
        (0..self.n).map(|i| self.score(query, i)).collect()
    }

    pub fn get_top_results(&self, query: &str, top_k: usize) -> Vec<(usize, f32)> {
        let scores = self.get_scores(query);
        let mut indexed_scores: Vec<(usize, f32)> = scores.into_iter().enumerate().collect();

        indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        indexed_scores.truncate(top_k);
        indexed_scores
    }

    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "k1": self.k1,
            "b": self.b,
            "docs": self.docs,
            "tf": self.tf.iter().map(|m| {
                m.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
            "df": self.df.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>(),
            "doc_lens": self.doc_lens,
            "n": self.n,
            "avgdl": self.avgdl,
            "index_to_id": self.index_to_id.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>()
        })
    }

    pub fn from_dict(value: &serde_json::Value) -> Self {
        let k1 = value.get("k1").and_then(|v| v.as_f64()).unwrap_or(1.5) as f32;
        let b = value.get("b").and_then(|v| v.as_f64()).unwrap_or(0.75) as f32;

        let mut docs = Vec::new();
        if let Some(docs_arr) = value.get("docs").and_then(|v| v.as_array()) {
            for doc in docs_arr {
                if let Some(tokens) = doc.as_array() {
                    docs.push(
                        tokens
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        let mut tf = Vec::new();
        if let Some(tf_arr) = value.get("tf").and_then(|v| v.as_array()) {
            for tf_doc in tf_arr {
                let mut doc_map = HashMap::new();
                if let Some(tokens_arr) = tf_doc.as_array() {
                    for item in tokens_arr {
                        if let Some(arr) = item.as_array() {
                            if arr.len() >= 2 {
                                let key = arr[0].as_str().unwrap_or("").to_string();
                                let value = arr[1].as_u64().unwrap_or(0) as u32;
                                doc_map.insert(key, value);
                            }
                        }
                    }
                }
                tf.push(doc_map);
            }
        }

        let mut df = HashMap::new();
        if let Some(df_arr) = value.get("df").and_then(|v| v.as_array()) {
            for item in df_arr {
                if let Some(arr) = item.as_array() {
                    if arr.len() >= 2 {
                        let key = arr[0].as_str().unwrap_or("").to_string();
                        let value = arr[1].as_u64().unwrap_or(0) as u32;
                        df.insert(key, value);
                    }
                }
            }
        }

        let mut doc_lens = Vec::new();
        if let Some(len_arr) = value.get("doc_lens").and_then(|v| v.as_array()) {
            for len in len_arr {
                doc_lens.push(len.as_u64().unwrap_or(0) as usize);
            }
        }

        let n = value.get("n").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let avgdl = value.get("avgdl").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

        let mut index_to_id = HashMap::new();
        if let Some(idx_arr) = value.get("index_to_id").and_then(|v| v.as_array()) {
            for item in idx_arr {
                if let Some(arr) = item.as_array() {
                    if arr.len() >= 2 {
                        let key = arr[0].as_u64().unwrap_or(0) as usize;
                        let value = arr[1].as_str().unwrap_or("").to_string();
                        index_to_id.insert(key, value);
                    }
                }
            }
        }

        Self {
            k1,
            b,
            docs,
            tf,
            df,
            doc_lens,
            n,
            avgdl,
            index_to_id,
        }
    }

    pub fn get_point_id(&self, index: usize) -> Option<&String> {
        self.index_to_id.get(&index)
    }

    pub fn get_index(&self, point_id: &str) -> Option<usize> {
        self.index_to_id
            .iter()
            .find(|(_, id)| id.as_str() == point_id)
            .map(|(idx, _)| *idx)
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn delete_document(&mut self, note_id: &str) -> bool {
        if let Some(&index) =
            self.index_to_id
                .iter()
                .find_map(|(k, v)| if v == note_id { Some(k) } else { None })
        {
            self.docs.remove(index);
            self.tf.remove(index);
            self.doc_lens.remove(index);
            self.index_to_id.remove(&index);

            let mut new_index_to_id = HashMap::new();
            for (i, doc) in self.docs.iter().enumerate() {
                if let Some(original_id) = self.index_to_id.get(&i) {
                    new_index_to_id.insert(i, original_id.clone());
                }
            }
            self.index_to_id = new_index_to_id;

            self.n = self.docs.len();
            self.avgdl = self.doc_lens.iter().sum::<usize>() as f32 / self.n as f32;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_basic() {
        let mut bm25 = BM25::new(1.5, 0.75);

        bm25.add_document("Rust is a systems programming language", "1".to_string());
        bm25.add_document("Python is a high-level language", "2".to_string());
        bm25.add_document("Rust and Python are programming languages", "3".to_string());

        assert_eq!(bm25.len(), 3);
        assert!(bm25.len() > 0);

        let scores = bm25.get_scores("Rust programming");
        assert_eq!(scores.len(), 3);

        assert!(scores[0] > 0.0, "First doc should have score for 'Rust'");
        assert!(
            scores[2] > 0.0,
            "Third doc should have score for both terms"
        );
    }

    #[test]
    fn test_bm25_serialization() {
        let mut bm25 = BM25::new(1.5, 0.75);
        bm25.add_document("test document", "doc1".to_string());

        let json = bm25.to_dict();
        let bm25_restored = BM25::from_dict(&json);

        assert_eq!(bm25.len(), bm25_restored.len());
        assert_eq!(bm25.avgdl, bm25_restored.avgdl);
    }

    #[test]
    fn test_bm25_idf() {
        let mut bm25 = BM25::new(1.5, 0.75);
        bm25.add_document("hello world", "1".to_string());
        bm25.add_document("hello rust", "2".to_string());
        bm25.add_document("world python", "3".to_string());

        let idf_hello = bm25.idf("hello");
        let idf_rust = bm25.idf("rust");

        assert!(idf_hello > 0.0, "IDF should be positive");
        assert!(idf_rust > idf_hello, "Rare term should have higher IDF");
    }
}
