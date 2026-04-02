import math
import json
import requests
from collections import Counter
from sentence_transformers import SentenceTransformer

QDRANT_URL = "http://localhost:6333"
COLLECTION = "hybrid_notes_prod"
BM25_META_ID = "BM25_META"

# =========================================================
# PERSISTENT BM25 WITH INDEX MAPPING
# =========================================================
class BM25:
    def __init__(self, k1=1.5, b=0.75):
        self.k1 = k1
        self.b = b
        self.docs = []  # list of token lists
        self.tf = []
        self.df = {}
        self.doc_lens = []
        self.N = 0
        self.avgdl = 0
        self.index_to_id = {}  # BM25 doc index -> Qdrant point ID

    def add_document(self, doc, point_id):
        tokens = doc.lower().split()
        self.docs.append(tokens)
        self.index_to_id[self.N] = point_id
        self.N += 1

        tf_doc = Counter(tokens)
        self.tf.append(tf_doc)

        for word in set(tokens):
            self.df[word] = self.df.get(word, 0) + 1

        doc_len = len(tokens)
        self.doc_lens.append(doc_len)
        self.avgdl = sum(self.doc_lens) / self.N

    def idf(self, term):
        df = self.df.get(term, 0)
        return math.log((self.N - df + 0.5) / (df + 0.5) + 1)

    def score(self, query, index):
        query_terms = query.lower().split()
        score = 0.0
        doc_len = self.doc_lens[index]

        for term in query_terms:
            if term not in self.tf[index]:
                continue
            f = self.tf[index][term]
            idf = self.idf(term)
            denom = f + self.k1 * (1 - self.b + self.b * doc_len / self.avgdl)
            score += idf * (f * (self.k1 + 1)) / denom
        return score

    def get_scores(self, query):
        return [self.score(query, i) for i in range(self.N)]

    # -------------------------
    # Persistence
    # -------------------------
    def to_dict(self):
        return {
            "k1": self.k1,
            "b": self.b,
            "docs": self.docs,
            "tf": [dict(c) for c in self.tf],
            "df": self.df,
            "doc_lens": self.doc_lens,
            "N": self.N,
            "avgdl": self.avgdl,
            "index_to_id": self.index_to_id
        }

    @classmethod
    def from_dict(cls, d):
        bm25 = cls(d["k1"], d["b"])
        bm25.docs = d["docs"]
        bm25.tf = [Counter(c) for c in d["tf"]]
        bm25.df = d["df"]
        bm25.doc_lens = d["doc_lens"]
        bm25.N = d["N"]
        bm25.avgdl = d["avgdl"]
        bm25.index_to_id = d.get("index_to_id", {})
        return bm25


# =========================================================
# QDRANT FUNCTIONS
# =========================================================
def recreate_collection():
    requests.delete(f"{QDRANT_URL}/collections/{COLLECTION}")
    payload = {"vectors": {"size": 384, "distance": "Cosine"}}
    r = requests.put(f"{QDRANT_URL}/collections/{COLLECTION}", json=payload)
    print("Collection recreated:", r.text)


def insert_note(note_id, note, model, bm25):
    vec = model.encode(note).tolist()
    bm25.add_document(note, note_id)

    point = {
        "id": note_id,
        "vector": vec,
        "payload": {"text": note}
    }

    requests.put(
        f"{QDRANT_URL}/collections/{COLLECTION}/points?wait=true",
        json={"points": [point]}
    )
    save_bm25_state(bm25)
    print(f"Inserted {note_id}: {note}")


def save_bm25_state(bm25):
    state = bm25.to_dict()
    payload = {
        "id": BM25_META_ID,
        "vector": [0.0]*384,
        "payload": {"bm25_state": json.dumps(state)}
    }
    requests.put(
        f"{QDRANT_URL}/collections/{COLLECTION}/points?wait=true",
        json={"points": [payload]}
    )


def load_bm25_state():
    r = requests.get(f"{QDRANT_URL}/collections/{COLLECTION}/points/{BM25_META_ID}?with_payload=true")
    resp = r.json()
    if resp.get("result"):
        bm25_json = resp["result"]["payload"]["bm25_state"]
        return BM25.from_dict(json.loads(bm25_json))
    return None


def fetch_notes_by_ids(ids):
    """Fetch only specific point IDs from Qdrant"""
    if not ids:
        return {}
    notes = {}
    for pid in ids:
        r = requests.get(f"{QDRANT_URL}/collections/{COLLECTION}/points/{pid}?with_payload=true")
        res = r.json().get("result")
        if res:
            notes[pid] = res["payload"]["text"]
    return notes


# =========================================================
# DENSE SEARCH
# =========================================================
def dense_search(query, model, limit=5):
    vec = model.encode(query).tolist()
    payload = {"vector": vec, "limit": limit, "with_payload": False}
    r = requests.post(f"{QDRANT_URL}/collections/{COLLECTION}/points/search", json=payload)
    results = r.json().get("result", [])
    return {res["id"]: res["score"] for res in results}


# =========================================================
# HYBRID SEARCH
# =========================================================
def hybrid_search(query, model, bm25, top_k=5):
    print(f"\n🔍 Query: {query}")

    dense_scores = dense_search(query, model)
    bm25_scores = bm25.get_scores(query)
    max_bm25 = max(bm25_scores) if max(bm25_scores) > 0 else 1
    bm25_scores = [s / max_bm25 for s in bm25_scores]

    # Get top BM25 indexes
    top_bm25_indexes = sorted(range(len(bm25_scores)), key=lambda i: bm25_scores[i], reverse=True)[:top_k]
    top_point_ids = [bm25.index_to_id[i] for i in top_bm25_indexes]

    # Fetch only top notes from Qdrant
    id_to_text = fetch_notes_by_ids(top_point_ids)

    # Merge dense scores
    final_scores = {}
    for pid in top_point_ids:
        bm25_idx = list(bm25.index_to_id.keys())[list(bm25.index_to_id.values()).index(pid)]
        d_score = dense_scores.get(pid, 0)
        final_scores[pid] = 0.7*d_score + 0.3*bm25_scores[bm25_idx]

    ranked_ids = sorted(final_scores, key=lambda x: final_scores[x], reverse=True)[:top_k]

    print("--- Hybrid Results ---")
    for pid in ranked_ids:
        print(f"{id_to_text.get(pid,'<missing>')} | score: {final_scores[pid]:.4f}")


# =========================================================
# MAIN
# =========================================================
def main():
    model = SentenceTransformer("all-MiniLM-L6-v2")
    bm25 = load_bm25_state()
    if bm25:
        print("Loaded BM25 state from previous session.")
    else:
        print("No previous BM25 found. Recreating collection and inserting notes.")
        recreate_collection()
        bm25 = BM25()
        notes = [
            "Learn Golang concurrency patterns",
            "WebRTC streaming low latency optimization",
            "Machine learning basics and regression",
            "Deep learning neural networks",
            "Rust ownership and memory safety",
            "C++ multithreading performance tuning",
            "Vector database indexing and search",
            "Hybrid search using dense and sparse vectors"
        ]
        for idx, note in enumerate(notes):
            insert_note(idx, note, model, bm25)

    # Hybrid queries
    hybrid_search("related to Rust", model, bm25)
    hybrid_search("streaming optimization", model, bm25)
    hybrid_search("vector database", model, bm25)


if __name__ == "__main__":
    main()
