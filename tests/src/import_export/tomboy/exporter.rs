use crate::import_export::tomboy::parser::{TomboyNote, TomboyResult};
use crate::models::note::Note;

pub struct TomboyExporter;

impl TomboyExporter {
    pub fn note_to_tomboy(note: &Note) -> TomboyResult<TomboyNote> {
        let content = format!("<note-content>\n{}\n</note-content>", note.content);

        Ok(TomboyNote {
            title: note.title.clone(),
            raw_content: content.clone(),
            content,
            tags: Vec::new(),
            attachments: Vec::new(),
            create_date: Some(note.created_at),
            last_change_date: Some(note.updated_at),
            last_metadata_change_date: None,
        })
    }

    pub fn note_to_xml(note: &Note) -> TomboyResult<String> {
        let tomboy_note = Self::note_to_tomboy(note)?;

        let xml = format!(
            r#"<note version="0.1">
  <title>{}</title>
  <content>
    {}
  </content>
  <tags>
    {}
  </tags>
  <last-modified>{}</last-modified>
</note>"#,
            note.title,
            tomboy_note.content,
            tomboy_note
                .tags
                .iter()
                .map(|t| format!("    <tag>{}</tag>", t))
                .collect::<Vec<_>>()
                .join("\n"),
            note.updated_at.to_rfc3339()
        );

        Ok(xml)
    }
}

pub fn convert_wiki_links_to_html(markdown: &str) -> String {
    // Convert [[Link Name]] to HTML anchor links
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(markdown, |caps: &regex::Captures| {
        let link = &caps[1];
        format!("<a href=\"#{}\">{}</a>", link, link)
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_note() {
        let note = Note::new("Test Note".to_string(), "Test content".to_string());

        let xml = TomboyExporter::note_to_xml(&note).unwrap();
        assert!(xml.contains("<note version=\"0.1\">"));
        assert!(xml.contains("<title>Test Note</title>"));
    }
}
