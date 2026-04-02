use crate::models::Note;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

const ADJECTIVES: [&str; 50] = [
    "amazing",
    "beautiful",
    "complex",
    "delightful",
    "elegant",
    "fascinating",
    "great",
    "harmonious",
    "important",
    "jubilant",
    "knowledgeable",
    "magnificent",
    "noble",
    "outstanding",
    "perfect",
    "quality",
    "remarkable",
    "spectacular",
    "terrific",
    "unique",
    "valuable",
    "wonderful",
    "excellent",
    "superb",
    "fantastic",
    "incredible",
    "unbelievable",
    "phenomenal",
    "astounding",
    "breathtaking",
    "captivating",
    "dazzling",
    "enchanting",
    "extraordinary",
    "magnificent",
    "mesmerizing",
    "outstanding",
    "stunning",
    "sublime",
    "transcendent",
    "ultimate",
    "unmatched",
    "veritable",
    "whimsical",
    "youthful",
    "zealous",
    "ambitious",
    "brilliant",
    "creative",
    "dynamic",
];

const NOUNS: [&str; 60] = [
    "algorithm",
    "browser",
    "cache",
    "database",
    "element",
    "function",
    "gateway",
    "hardware",
    "interface",
    "javascript",
    "kernel",
    "language",
    "module",
    "network",
    "object",
    "protocol",
    "query",
    "runtime",
    "script",
    "template",
    "utility",
    "variable",
    "widget",
    "xml",
    "yaml",
    "framework",
    "compiler",
    "debugger",
    "editor",
    "server",
    "client",
    "request",
    "response",
    "session",
    "token",
    "authentication",
    "encryption",
    "decryption",
    "compression",
    "decompression",
    "optimization",
    "refactoring",
    "integration",
    "deployment",
    "monitoring",
    "logging",
    "caching",
    "scheduling",
    "routing",
    "filtering",
    "sorting",
    "searching",
    "indexing",
    "partitioning",
    "sharding",
    "replication",
    "backup",
    "recovery",
    "migration",
    "upgrade",
];

const TOPICS: [&str; 20] = [
    "technology",
    "science",
    "art",
    "music",
    "literature",
    "history",
    "philosophy",
    "psychology",
    "physics",
    "mathematics",
    "chemistry",
    "biology",
    "medicine",
    "engineering",
    "computer science",
    "artificial intelligence",
    "machine learning",
    "data science",
    "web development",
    "mobile apps",
];

pub fn generate_random_notes(count: usize, user_id: &str) -> Vec<Note> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut notes = Vec::new();
    let mut generated_titles = HashSet::new();

    for _i in 0..count {
        let title = generate_random_title(&mut rng, &mut generated_titles);
        let content = generate_random_content(&mut rng);
        let tags = generate_random_tags(&mut rng);

        let note = Note::new(title, content)
            .with_user_id(user_id.to_string())
            .with_tags(tags);

        notes.push(note);
    }

    notes
}

fn generate_random_title(rng: &mut StdRng, used_titles: &mut HashSet<String>) -> String {
    loop {
        let adj_idx = rng.gen_range(0..ADJECTIVES.len());
        let noun_idx = rng.gen_range(0..NOUNS.len());
        let topic_idx = rng.gen_range(0..TOPICS.len());

        let title = format!(
            "{} {} in {}",
            ADJECTIVES[adj_idx], NOUNS[noun_idx], TOPICS[topic_idx]
        );

        if !used_titles.contains(&title) {
            used_titles.insert(title.clone());
            return title;
        }
    }
}

fn generate_random_content(rng: &mut StdRng) -> String {
    let sentences = rng.gen_range(5..15);
    let mut content = String::new();

    for _ in 0..sentences {
        let words = rng.gen_range(8..20);
        let sentence = generate_random_sentence(rng, words);
        content.push_str(&sentence);
        content.push('.');
        content.push(' ');
    }

    content.trim().to_string()
}

fn generate_random_sentence(rng: &mut StdRng, word_count: usize) -> String {
    let words = vec![
        "The",
        "quick",
        "brown",
        "fox",
        "jumps",
        "over",
        "the",
        "lazy",
        "dog",
        "This",
        "is",
        "a",
        "sample",
        "text",
        "for",
        "testing",
        "purposes",
        "Machine",
        "learning",
        "algorithms",
        "can",
        "process",
        "vast",
        "amounts",
        "of",
        "data",
        "efficiently",
        "using",
        "modern",
        "computing",
        "hardware",
        "Vector",
        "embeddings",
        "enable",
        "semantic",
        "search",
        "capabilities",
        "Large",
        "language",
        "models",
        "have",
        "transformed",
        "artificial",
        "intelligence",
        "Neural",
        "networks",
        "mimic",
        "the",
        "human",
        "brain",
        "functionality",
    ];

    let mut sentence = String::new();
    for i in 0..word_count {
        if i > 0 {
            sentence.push(' ');
        }
        let word_idx = rng.gen_range(0..words.len());
        sentence.push_str(words[word_idx]);
    }

    sentence
}

fn generate_random_tags(rng: &mut StdRng) -> Vec<String> {
    let num_tags = rng.gen_range(1..4);
    let mut tags = HashSet::new();

    while tags.len() < num_tags {
        let topic_idx = rng.gen_range(0..TOPICS.len());
        tags.insert(TOPICS[topic_idx].to_string());
    }

    tags.into_iter().collect()
}
