use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomboyNote {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub attachments: Vec<String>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

impl TomboyNote {
    pub fn parse_xml(path: &Path) -> Result<Self, TomboyError> {
        let content = std::fs::read_to_string(path).map_err(TomboyError::Io)?;

        let mut title = String::new();
        let mut content_str = String::new();
        let mut tags = Vec::new();
        let mut attachments = Vec::new();

        let mut xml = quick_xml::Reader::from_str(&content);
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match xml.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => match e.name().as_ref() {
                    b"title" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        title = s;
                    }
                    b"content" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        content_str = s;
                    }
                    b"note-content" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        content_str = s;
                    }
                    b"tag" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        tags.push(s);
                    }
                    b"attachment" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        attachments.push(s);
                    }
                    _ => {}
                },
                Ok(quick_xml::events::Event::End(e)) => {
                    if e.name().as_ref() == b"note" {
                        break;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(TomboyError::Xml(e)),
                _ => {}
            }
        }

        Ok(TomboyNote {
            title,
            content: content_str,
            tags,
            attachments,
            last_modified: chrono::Utc::now(),
        })
    }

    pub fn to_memo_rs_note(self) -> crate::models::note::Note {
        crate::models::note::Note::new(self.title, self.content)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TomboyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parsing error: {0}")]
    Xml(quick_xml::Error),
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

pub type TomboyResult<T> = Result<T, TomboyError>;

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_gnote_format() {
        let content = r#"<?xml version="1.0"?>
<note version="0.3" xmlns:link="http://beatniksoftware.com/tomboy/link" xmlns:size="http://beatniksoftware.com/tomboy/size" xmlns="http://beatniksoftware.com/tomboy"><title>Gnote First Note1</title><text xml:space="preserve"><note-content version="0.1" xmlns:link="http://beatniksoftware.com/tomboy/link" xmlns:size="http://beatniksoftware.com/tomboy/size">Gnote First Note1

Here is the update of the first note</note-content>
</text><last-change-date>2026-03-02T16:47:38.208019Z</last-change-date></note>"#;

        let mut title = String::new();
        let mut content_str = String::new();

        let mut reader = quick_xml::Reader::from_str(content);
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => match e.name().as_ref() {
                    b"title" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            title = text.to_string();
                        }
                    }
                    b"content" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            content_str = text.to_string();
                        }
                    }
                    b"note-content" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            content_str = text.to_string();
                        }
                    }
                    _ => {}
                },
                Ok(quick_xml::events::Event::End(e)) => {
                    if e.name().as_ref() == b"note" {
                        break;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
        }

        assert_eq!(title, "Gnote First Note1");
        assert!(content_str.contains("Gnote First Note1"));
        assert!(content_str.contains("Here is the update of the first note"));
    }
}
