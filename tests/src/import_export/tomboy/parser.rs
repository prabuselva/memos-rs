use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomboyNote {
    pub title: String,
    pub raw_content: String,
    pub content: String,
    pub tags: Vec<String>,
    pub attachments: Vec<String>,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_change_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_metadata_change_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl TomboyNote {
    pub fn parse_xml(path: &Path) -> Result<Self, TomboyError> {
        let content = std::fs::read_to_string(path).map_err(TomboyError::Io)?;

        let mut title = String::new();
        let mut raw_content = String::new();
        let mut tags = Vec::new();
        let mut attachments = Vec::new();
        let mut create_date: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut last_change_date: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut last_metadata_change_date: Option<chrono::DateTime<chrono::Utc>> = None;

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
                        raw_content = s.clone();
                    }
                    b"note-content" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        raw_content = s.clone();
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
                    b"create-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        create_date = parse_iso8601(&s);
                    }
                    b"last-change-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        last_change_date = parse_iso8601(&s);
                    }
                    b"last-metadata-change-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        last_metadata_change_date = parse_iso8601(&s);
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

        let transformed_content = transform_content_to_markdown(&raw_content);

        Ok(TomboyNote {
            title,
            raw_content,
            content: transformed_content,
            tags,
            attachments,
            create_date,
            last_change_date,
            last_metadata_change_date,
        })
    }

    pub fn parse_xml_string(content: &str) -> Result<Self, TomboyError> {
        let mut title = String::new();
        let mut raw_content = String::new();
        let mut tags = Vec::new();
        let mut attachments = Vec::new();
        let mut create_date: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut last_change_date: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut last_metadata_change_date: Option<chrono::DateTime<chrono::Utc>> = None;

        let mut xml = quick_xml::Reader::from_str(content);
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
                        raw_content = s.clone();
                    }
                    b"note-content" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        raw_content = s.clone();
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
                    b"create-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        create_date = parse_iso8601(&s);
                    }
                    b"last-change-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        last_change_date = parse_iso8601(&s);
                    }
                    b"last-metadata-change-date" => {
                        let s = match xml.read_text(e.name()) {
                            Ok(text) => text.into_owned(),
                            Err(e) => return Err(TomboyError::Xml(e)),
                        };
                        last_metadata_change_date = parse_iso8601(&s);
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

        let transformed_content = transform_content_to_markdown(&raw_content);

        Ok(TomboyNote {
            title,
            raw_content,
            content: transformed_content,
            tags,
            attachments,
            create_date,
            last_change_date,
            last_metadata_change_date,
        })
    }

    pub fn to_memo_rs_note(self) -> crate::models::note::Note {
        let mut note = crate::models::note::Note::new(self.title, self.content);

        if let Some(date) = self.create_date {
            note.created_at = date;
        }
        if let Some(date) = self.last_change_date {
            note.updated_at = date;
        }

        note.tags = self.tags;

        let metadata = serde_json::json!({
            "tomboy": {
                "raw_content": self.raw_content,
                "attachments": self.attachments,
                "create_date": self.create_date.map(|d| d.to_rfc3339()),
                "last_change_date": self.last_change_date.map(|d| d.to_rfc3339()),
                "last_metadata_change_date": self.last_metadata_change_date.map(|d| d.to_rfc3339())
            }
        });
        note.metadata = metadata;

        note
    }

    pub fn to_memo_rs_note_with_title_removed(mut self) -> crate::models::note::Note {
        let title = self.title.clone();
        let mut note = self.to_memo_rs_note();

        // Remove the first line from content if it matches the title
        // This prevents title duplication since title is shown separately in UI
        if note.content.starts_with(&format!("{}\n", title)) {
            let lines: Vec<&str> = note.content.splitn(2, '\n').collect();
            if lines.len() > 1 {
                note.content = lines[1].to_string();
            } else {
                note.content = String::new();
            }
        }

        note
    }
}

pub fn parse_iso8601(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub fn transform_content_to_markdown(content: &str) -> String {
    use quick_xml::events::Event;

    let mut result = String::new();
    let mut reader = quick_xml::Reader::from_str(content);
    let mut buf = Vec::new();
    let mut in_datetime = false;
    let mut in_link_url = false;
    let mut in_link_internal = false;
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_strikethrough = false;
    let mut in_list_item = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"datetime" => {
                    in_datetime = true;
                }
                b"bold" => {
                    in_bold = true;
                    result.push_str("**");
                }
                b"italic" => {
                    in_italic = true;
                    result.push_str("*");
                }
                b"strikethrough" => {
                    in_strikethrough = true;
                    result.push_str("~~");
                }
                b"size:large" => {
                    result.push_str("\n## ");
                }
                b"size:x-large" => {
                    result.push_str("\n# ");
                }
                b"link:url" => {
                    in_link_url = true;
                }
                b"link:internal" => {
                    in_link_internal = true;
                }
                b"list-item" => {
                    in_list_item = true;
                    result.push_str("\n- ");
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"datetime" => {
                    in_datetime = false;
                }
                b"bold" => {
                    in_bold = false;
                    result.push_str("**");
                }
                b"italic" => {
                    in_italic = false;
                    result.push_str("*");
                }
                b"strikethrough" => {
                    in_strikethrough = false;
                    result.push_str("~~");
                }
                b"size:large" => {
                    result.push('\n');
                }
                b"size:x-large" => {
                    result.push('\n');
                }
                b"link:url" => {
                    in_link_url = false;
                }
                b"link:internal" => {
                    in_link_internal = false;
                }
                b"list-item" => {
                    in_list_item = false;
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if let Ok(text) = e.unescape() {
                    if in_datetime {
                        if let Some(dt) = parse_datetime_text(&text) {
                            result.push_str(&format!("\n{}\n\n", dt.to_rfc3339()));
                        }
                    } else if in_link_url {
                        result.push_str(&format!("[{}]({})", text, text));
                    } else if in_link_internal {
                        let slug = text.to_lowercase().replace(' ', "-");
                        result.push_str(&format!("[{}](#{})", text, slug));
                    } else if in_bold || in_italic || in_strikethrough {
                        result.push_str(&text);
                    } else if in_list_item {
                        result.push_str(&text);
                    } else {
                        result.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    result
}

pub fn parse_datetime_text(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let formats = [
        "%A, %B %d, %Y, %I:%M %p",
        "%A, %B %d, %Y, %H:%M",
        "%A, %B %d, %Y %I:%M %p",
        "%A, %B %d, %Y %H:%M",
        "%B %d, %Y %I:%M %p",
        "%B %d, %Y %H:%M",
    ];

    for format in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, format) {
            return Some(dt.and_utc());
        }
    }

    None
}

#[derive(Debug, thiserror::Error)]
pub enum TomboyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parsing error: {0}")]
    Xml(quick_xml::Error),
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Glob error: {0}")]
    Glob(glob::PatternError),
}

pub type TomboyResult<T> = Result<T, TomboyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_simple() {
        let input = "Test <bold>bold</bold>";
        let output = transform_content_to_markdown(input);
        assert_eq!(output, "Test **bold**");
    }

    #[test]
    fn test_transform_datetime() {
        let input = "<datetime>Thursday, August 4, 2022, 10:27 PM</datetime>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("2022-08-04T22:27:00+00:00"));
    }

    #[test]
    fn test_transform_size() {
        let input = "<size:x-large>Header</size:x-large>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("# Header"));
    }

    #[test]
    fn test_transform_link() {
        let input = "<link:url>https://example.com</link:url>";
        let output = transform_content_to_markdown(input);
        assert!(output.contains("[https://example.com](https://example.com)"));
    }
}
