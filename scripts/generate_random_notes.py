#!/usr/bin/env python3
"""
Random Notes Generator for Test Data Generation
Usage: python generate_random_notes.py <count> <user_id>
"""

import random
import json
import sys

ADJECTIVES = [
    "amazing", "beautiful", "complex", "delightful", "elegant",
    "fascinating", "great", "harmonious", "important", "jubilant",
    "knowledgeable", "magnificent", "noble", "outstanding", "perfect",
    "quality", "remarkable", "spectacular", "terrific", "unique",
    "valuable", "wonderful", "excellent", "superb", "fantastic",
    "incredible", "unbelievable", "phenomenal", "astounding", "breathtaking",
    "captivating", "dazzling", "enchanting", "extraordinary", "magnificent",
    "mesmerizing", "outstanding", "stunning", "sublime", "transcendent",
    "ultimate", "unmatched", "veritable", "whimsical", "youthful",
    "zealous", "ambitious", "brilliant", "creative", "dynamic",
]

NOUNS = [
    "algorithm", "browser", "cache", "database", "element",
    "function", "gateway", "hardware", "interface", "javascript",
    "kernel", "language", "module", "network", "object",
    "protocol", "query", "runtime", "script", "template",
    "utility", "variable", "widget", "xml", "yaml",
    "framework", "compiler", "debugger", "editor", "server",
    "client", "request", "response", "session", "token",
    "authentication", "encryption", "decryption", "compression", "decompression",
    "optimization", "refactoring", "integration", "deployment", "monitoring",
    "logging", "caching", "scheduling", "routing", "filtering",
    "sorting", "searching", "indexing", "partitioning", "sharding",
    "replication", "backup", "recovery", "migration", "upgrade",
]

TOPICS = [
    "technology", "science", "art", "music", "literature",
    "history", "philosophy", "psychology", "physics", "mathematics",
    "chemistry", "biology", "medicine", "engineering", "computer science",
    "artificial intelligence", "machine learning", "data science", "web development", "mobile apps",
]

def generate_random_title():
    adj = random.choice(ADJECTIVES)
    noun = random.choice(NOUNS)
    topic = random.choice(TOPICS)
    return f"{adj.capitalize()} {noun.capitalize()} in {topic.capitalize()}"

def generate_random_content():
    sentences = random.randint(5, 15)
    content = []
    
    for _ in range(sentences):
        words = random.randint(8, 20)
        words_list = random.choices([
            "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
            "This", "is", "a", "sample", "text", "for", "testing", "purposes",
            "Machine", "learning", "algorithms", "can", "process", "vast", "amounts", "of",
            "data", "efficiently", "using", "modern", "computing", "hardware",
            "Vector", "embeddings", "enable", "semantic", "search", "capabilities",
            "Large", "language", "models", "have", "transformed", "artificial", "intelligence",
            "Neural", "networks", "mimic", "the", "human", "brain", "functionality",
        ], k=words)
        sentence = " ".join(words_list)
        content.append(sentence.capitalize() + ".")
    
    return " ".join(content)

def generate_random_tags():
    num_tags = random.randint(1, 3)
    return random.sample(TOPICS, num_tags)

def main():
    count = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    user_id = sys.argv[2] if len(sys.argv) > 2 else "test-user"
    
    notes = []
    for i in range(count):
        note = {
            "id": f"note-{i:04d}",
            "title": generate_random_title(),
            "content": generate_random_content(),
            "tags": generate_random_tags(),
            "user_id": user_id,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }
        notes.append(note)
        
        if (i + 1) % 10 == 0:
            print(f"Generated {i + 1}/{count} notes")
    
    with open(f"random_notes_{count}.json", "w") as f:
        json.dump(notes, f, indent=2)
    
    print(f"\nGenerated {count} random notes to random_notes_{count}.json")

if __name__ == "__main__":
    main()
