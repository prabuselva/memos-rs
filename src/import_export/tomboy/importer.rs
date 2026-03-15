use crate::import_export::tomboy::parser::{TomboyError, TomboyNote, TomboyResult};
use glob::glob;
use std::path::PathBuf;

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
        let mut files: Vec<PathBuf> = Vec::new();

        let pattern = self.base_dir.join("**/*.note");
        let glob_pattern = pattern.to_string_lossy();

        for entry in glob(&glob_pattern).map_err(|_| TomboyError::MissingField("note files"))? {
            if let Ok(file) = entry {
                files.push(file);
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importer_creation() {
        let importer = TomboyImporter::new("/tmp/test-tomboy");
        assert_eq!(importer.base_dir, PathBuf::from("/tmp/test-tomboy"));
    }
}
