use anyhow::Result;
use memos_rs::config::Config;
use memos_rs::db::Database;
use memos_rs::embeddings::{BERTModel, ModelDownloader};
use memos_rs::vector::store::VectorStore;
use memos_rs::{generate_random_notes, import_wikipedia_notes, initialize_vector_store_with_notes};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::default();

    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;

    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();
    let model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )?;

    let db = Database::new(&config).await?;

    eprintln!("Generating random test notes...");
    let random_notes = generate_random_notes(100, "test-user");
    eprintln!("Generated {} random notes", random_notes.len());

    for note in &random_notes {
        db.create_note_with_user(note.clone()).await?;
    }
    eprintln!("Saved random notes to database");

    eprintln!("Importing Wikipedia notes...");
    let wiki_notes = import_wikipedia_notes("technology", 50, "test-user").await?;
    for note in &wiki_notes {
        db.create_note_with_user(note.clone()).await?;
    }
    eprintln!("Saved {} Wikipedia notes", wiki_notes.len());

    let vector_store = VectorStore::new(&config.vector.url).await?;
    initialize_vector_store_with_notes(&vector_store, &model, &random_notes).await?;
    initialize_vector_store_with_notes(&vector_store, &model, &wiki_notes).await?;
    eprintln!("Initialized vector store with all notes");

    Ok(())
}
