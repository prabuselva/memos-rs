use crate::import_export::tomboy::{TomboyImporter, TomboyExporter};
use crate::models::note::Note;
use crate::db::Database;
use crate::repositories::NoteRepository;
use anyhow::Result;
use std::path::Path;

pub async fn import_from_tomboy(
    db: &Database,
    tomboy_dir: &str,
) -> Result<Vec<Note>> {
    let importer = TomboyImporter::new(tomboy_dir);
    let tomboy_notes = importer.import_all_recursive()?;
    
    let mut imported_notes = Vec::new();
    
    for tomboy_note in tomboy_notes {
        let note = tomboy_note.to_memo_rs_note();
        let saved_note = db.create(note).await?;
        imported_notes.push(saved_note);
    }
    
    Ok(imported_notes)
}

pub async fn export_to_tomboy(
    db: &Database,
    note_id: &str,
    output_dir: &str,
) -> Result<String> {
    let note = db.get_by_id(note_id).await?;
    
    let xml = TomboyExporter::note_to_xml(&note)?;
    
    let output_path = Path::new(output_dir).join(format!("{}.note", note.title));
    std::fs::write(&output_path, &xml)?;
    
    Ok(output_path.to_string_lossy().to_string())
}

pub async fn batch_import(
    db: &Database,
    files: Vec<String>,
) -> Result<Vec<Note>> {
    let mut imported_notes = Vec::new();
    
    for file_path in files {
        if let Ok(note) = TomboyNote::parse_xml(Path::new(&file_path)) {
            let memo_note = note.to_memo_rs_note();
            let saved = db.create(memo_note).await?;
            imported_notes.push(saved);
        }
    }
    
    Ok(imported_notes)
}

pub async fn rollback_import(
    db: &Database,
    note_ids: &[String],
) -> Result<usize> {
    let mut deleted_count = 0;
    
    for id in note_ids {
        if db.delete(id).await.is_ok() {
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}