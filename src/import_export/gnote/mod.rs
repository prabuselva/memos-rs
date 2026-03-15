use crate::import_export::tomboy::{TomboyImporter, TomboyExporter, TomboyNote};
use crate::models::note::Note;
use crate::db::Database;
use anyhow::Result;
use std::path::Path;

pub async fn import_from_gnote(
    db: &Database,
    gnote_dir: &str,
) -> Result<Vec<Note>> {
    // Gnote uses same format as Tomboy
    let importer = TomboyImporter::new(gnote_dir);
    let gnote_notes = importer.import_all_recursive()?;
    
    let mut imported_notes = Vec::new();
    
    for gnote_note in gnote_notes {
        let note = gnote_note.to_memo_rs_note();
        let saved_note = db.create(note).await?;
        imported_notes.push(saved_note);
    }
    
    Ok(imported_notes)
}

pub async fn export_to_gnote(
    db: &Database,
    note_id: &str,
    output_dir: &str,
) -> Result<String> {
    // Gnote uses same format as Tomboy
    export_to_tomboy(db, note_id, output_dir).await
}

pub async fn batch_import_gnote(
    db: &Database,
    files: Vec<String>,
) -> Result<Vec<Note>> {
    // Gnote uses same format as Tomboy
    batch_import_tomboy(db, files).await
}

async fn batch_import_tomboy(
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