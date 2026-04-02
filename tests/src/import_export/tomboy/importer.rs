use crate::import_export::tomboy::parser::{TomboyError, TomboyNote, TomboyResult};
use glob::glob;
use std::path::PathBuf;

// TomboyToMarkdown moved to parser.rs

pub struct TomboyImporter {
    base_dir: PathBuf,
}

impl TomboyImporter {
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    pub fn find_note_files(&self) -> TomboyResult<Vec<PathBuf>> {
        let pattern = self.base_dir.join("*.note");
        let glob_pattern = pattern.to_string_lossy();

        let files: Vec<PathBuf> = glob(&glob_pattern)
            .map_err(|_| TomboyError::MissingField("note files"))?
            .filter_map(|e| e.ok())
            .collect();

        Ok(files)
    }

    pub fn find_note_files_recursive(&self) -> TomboyResult<Vec<PathBuf>> {
        let pattern = self.base_dir.join("**/*.note");
        let glob_pattern = pattern.to_string_lossy();

        let files: Vec<PathBuf> = glob(&glob_pattern)
            .map_err(|_| TomboyError::MissingField("note files"))?
            .filter_map(|e| e.ok())
            .collect();

        Ok(files)
    }

    pub fn import_all(&self) -> TomboyResult<Vec<TomboyNote>> {
        let files = self.find_note_files()?;
        let mut notes = Vec::new();

        for file in files {
            if let Ok(note) = TomboyNote::parse_xml(&file) {
                notes.push(note);
            }
        }

        Ok(notes)
    }

    pub fn import_all_recursive(&self) -> TomboyResult<Vec<TomboyNote>> {
        let files = self.find_note_files_recursive()?;
        let mut notes = Vec::new();

        for file in files {
            if let Ok(note) = TomboyNote::parse_xml(&file) {
                notes.push(note);
            }
        }

        Ok(notes)
    }

    pub fn import_single(&self, filename: &str) -> TomboyResult<TomboyNote> {
        let path = self.base_dir.join(filename);
        TomboyNote::parse_xml(&path)
    }

    pub async fn import_all_as_notes_with_notebooks(
        &self,
        db: &crate::db::Database,
        user_id: &str,
    ) -> Result<Vec<crate::models::note::Note>, anyhow::Error> {
        let tomboy_notes = self.import_all()?;
        let mut notes = Vec::new();
        let mut created_notebooks: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for note in tomboy_notes {
            let mut notebook_name: Option<String> = None;
            let mut tags: Vec<String> = Vec::new();

            for tag in note.tags.clone() {
                if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                    notebook_name = Some(nb_name.to_string());
                } else {
                    tags.push(tag);
                }
            }

            let mut memo_note = note.to_memo_rs_note_with_title_removed();
            memo_note.user_id = Some(user_id.to_string());

            if let Some(nb_name) = notebook_name {
                let notebook_id = if let Some(id) = created_notebooks.get(&nb_name) {
                    id.clone()
                } else {
                    let notebook = crate::models::note::Notebook::new(nb_name.clone())
                        .with_user_id(user_id.to_string());
                    match db.get_notebook_by_name(&nb_name, user_id).await {
                        Ok(existing) => existing.id,
                        Err(_) => {
                            let created = db.create_notebook(notebook).await?;
                            created_notebooks.insert(nb_name.clone(), created.id.clone());
                            created.id
                        }
                    }
                };
                memo_note = memo_note.with_notebook(notebook_id);
                tags.push(nb_name);
            }

            memo_note.tags = tags;
            notes.push(memo_note);
        }

        Ok(notes)
    }

    pub async fn import_all_recursive_as_notes_with_notebooks(
        &self,
        db: &crate::db::Database,
        user_id: &str,
    ) -> Result<Vec<crate::models::note::Note>, anyhow::Error> {
        let tomboy_notes = self.import_all_recursive()?;
        let mut notes = Vec::new();
        let mut created_notebooks: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for note in tomboy_notes {
            let mut notebook_name: Option<String> = None;
            let mut tags: Vec<String> = Vec::new();

            for tag in note.tags.clone() {
                if let Some(nb_name) = tag.strip_prefix("system:notebook:") {
                    notebook_name = Some(nb_name.to_string());
                } else {
                    tags.push(tag);
                }
            }

            let mut memo_note = note.to_memo_rs_note_with_title_removed();
            memo_note.user_id = Some(user_id.to_string());

            if let Some(nb_name) = notebook_name {
                let notebook_id = if let Some(id) = created_notebooks.get(&nb_name) {
                    id.clone()
                } else {
                    let notebook = crate::models::note::Notebook::new(nb_name.clone())
                        .with_user_id(user_id.to_string());
                    match db.get_notebook_by_name(&nb_name, user_id).await {
                        Ok(existing) => existing.id,
                        Err(_) => {
                            let created = db.create_notebook(notebook).await?;
                            created_notebooks.insert(nb_name.clone(), created.id.clone());
                            created.id
                        }
                    }
                };
                memo_note = memo_note.with_notebook(notebook_id);
                tags.push(nb_name);
            }

            memo_note.tags = tags;
            notes.push(memo_note);
        }

        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_export::tomboy::parser::transform_content_to_markdown;

    #[test]
    fn test_importer_creation() {
        let importer = TomboyImporter::new("/tmp/test-tomboy");
        assert_eq!(importer.base_dir, PathBuf::from("/tmp/test-tomboy"));
    }

    #[test]
    fn test_transform_simple() {
        let input = "Test <bold>bold</bold> text";
        let output = transform_content_to_markdown(input);
        assert_eq!(output, "Test **bold** text");
    }

    #[test]
    fn test_transform_with_datetime() {
        let input = "<datetime>Thursday, August 4, 2022, 10:27 PM</datetime>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("2022-08-04T22:27:00+00:00"));
    }

    #[test]
    fn test_transform_size_large() {
        let input = "<size:large>Large Text</size:large>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("\n## Large Text"));
    }

    #[test]
    fn test_transform_link() {
        let input = "<link:url>https://example.com</link:url>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("[https://example.com](https://example.com)"));
    }
}
