use crate::models::note::Note;
use crate::db::Database;
use anyhow::Result;

pub async fn import_from_tomboy_ng_markdown(
    db: &Database,
    markdown_content: &str,
    metadata: Option<TomboyNgMetadata>,
) -> Result<Note> {
    let title = metadata.map(|m| m.title).unwrap_or_else(|| "Untitled".to_string());
    let content = parse_markdown_content(markdown_content);
    
    let note = Note::new(title, content);
    let saved_note = db.create(note).await?;
    
    Ok(saved_note)
}

fn parse_markdown_content(markdown: &str) -> String {
    // Tomboy-NG exports clean Markdown, just need to preserve content
    markdown.to_string()
}

pub async fn export_to_tomboy_ng_markdown(
    db: &Database,
    note_id: &str,
) -> Result<(String, TomboyNgMetadata)> {
    let note = db.get_by_id(note_id).await?;
    
    let metadata = TomboyNgMetadata {
        title: note.title.clone(),
        tags: Vec::new(),
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    };
    
    Ok((note.content, metadata))
}

#[derive(Debug, Clone)]
pub struct TomboyNgMetadata {
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn import_from_tomboy_ng_rtf(
    db: &Database,
    rtf_content: &str,
    metadata: Option<TomboyNgMetadata>,
) -> Result<Note> {
    // Convert RTF to Markdown using rtf-dom
    let markdown = convert_rtf_to_markdown(rtf_content)?;
    
    import_from_tomboy_ng_markdown(db, &markdown, metadata).await
}

fn convert_rtf_to_markdown(rtf: &str) -> Result<String> {
    // For now, return RTF as-is (in production, use rtf-dom crate)
    Ok(rtf.to_string())
}

pub async fn import_batch_from_tomboy_ng(
    db: &Database,
    files: Vec<String>,
) -> Result<Vec<Note>> {
    let mut imported_notes = Vec::new();
    
    for file_path in files {
        let content = std::fs::read_to_string(&file_path)?;
        
        // Try to parse as Markdown
        if let Ok(note) = import_from_tomboy_ng_markdown(db, &content, None).await {
            imported_notes.push(note);
        }
    }
    
    Ok(imported_notes)
}