use std::path::Path;

#[test]
fn test_import_my_tomboy_note() {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sample/my-tomboy.note");

    if !sample_path.exists() {
        eprintln!("Sample file not found at {:?}", sample_path);
        return;
    }

    let note = memos_rs::import_export::tomboy::TomboyNote::parse_xml(&sample_path).unwrap();

    assert_eq!(note.title, "ESP32-C3");
    assert!(note.content.contains("ESP32-C3"));
    assert!(note.content.contains("2022-08-04T22:27:00+00:00"));
    assert!(note.content.contains("Lolin/Wemos/NodeMCU C3-Mini boards"));
    assert!(note.content.contains("serial.disableDTR=true"));
    assert!(note.content.contains("serial.disableRTS=true"));
    assert!(note.content.contains("Serial Monitor works perfectly"));
    assert!(note.content.contains("https://www.reddit.com/r/esp32/"));
    assert!(note
        .content
        .contains("# Own Python based Serial Monitor to debug"));
    assert!(note.content.contains("serial_monitor.py"));
    assert!(note.content.contains("CPP_Practice/python_utils repo"));
    assert!(note.create_date.is_some());
    assert!(note.last_change_date.is_some());
    assert_eq!(note.tags, vec!["system:notebook:Computer_works"]);

    let memo_note = note.clone().to_memo_rs_note();
    assert_eq!(memo_note.title, "ESP32-C3");
    assert!(memo_note
        .tags
        .contains(&"system:notebook:Computer_works".to_string()));
    assert!(memo_note.metadata.get("tomboy").is_some());
}

#[test]
fn test_importer_integration() {
    let sample_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sample");

    if !sample_dir.exists() {
        eprintln!("Sample directory not found at {:?}", sample_dir);
        return;
    }

    let importer =
        memos_rs::import_export::tomboy::TomboyImporter::new(sample_dir.to_str().unwrap());

    let notes = importer.import_all().unwrap();

    assert!(!notes.is_empty());

    for note in &notes {
        assert!(!note.title.is_empty());
        assert!(!note.raw_content.is_empty());
        // Timestamps are optional in some Tomboy formats
    }
}

#[test]
fn test_importer_recursive() {
    let sample_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sample");

    if !sample_dir.exists() {
        eprintln!("Sample directory not found at {:?}", sample_dir);
        return;
    }

    let importer =
        memos_rs::import_export::tomboy::TomboyImporter::new(sample_dir.to_str().unwrap());

    let notes = importer.import_all_recursive().unwrap();

    assert!(!notes.is_empty());
}

#[test]
fn test_gnote_format_integration() {
    let content = r#"<?xml version="1.0" encoding="utf-8"?>
<note version="0.3" xmlns:link="http://beatniksoftware.com/tomboy/link" xmlns:size="http://beatniksoftware.com/tomboy/size" xmlns="http://beatniksoftware.com/tomboy">
  <title>Gnote Note</title>
  <text xml:space="preserve"><note-content version="0.1">Gnote content with <bold>bold</bold> text</note-content></text>
  <last-change-date>2026-03-02T16:47:38.208019Z</last-change-date>
  <create-date>2026-03-02T16:45:00.0000000Z</create-date>
  <tags>
    <tag>gnote</tag>
    <tag>test</tag>
  </tags>
</note>"#;

    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("memos-rs-test-gnote-12345");
    std::fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("test-gnote.note");
    std::fs::write(&test_file, content).unwrap();

    let note = memos_rs::import_export::tomboy::TomboyNote::parse_xml(&test_file).unwrap();

    assert_eq!(note.title, "Gnote Note");
    assert!(note.content.contains("Gnote content with **bold** text"));
    assert_eq!(note.tags, vec!["gnote", "test"]);
    assert!(note.create_date.is_some());
    assert!(note.last_change_date.is_some());

    std::fs::remove_file(test_file).ok();
    std::fs::remove_dir(&test_dir).ok();
}

#[test]
fn test_full_workflow() {
    let content = r#"<?xml version="1.0" encoding="utf-8"?>
<note version="0.3">
  <title>Workflow Test</title>
  <text xml:space="preserve"><note-content version="0.1">
<size:x-large>Workflow Test Note</size:x-large>

<datetime>Thursday, August 4, 2022, 10:27 PM</datetime>

This is a <bold>test</bold> of the full import workflow.

<link:url>https://wiki.gnome.org/Apps/Tomboy</link:url>
  </note-content></text>
  <create-date>2022-08-04T22:27:09.7663000+08:00</create-date>
  <last-change-date>2022-08-04T22:30:04.7620040+08:00</last-change-date>
  <tags>
    <tag>workflow</tag>
    <tag>test</tag>
  </tags>
</note>"#;

    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("memos-rs-test-workflow-12345");
    std::fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("test-workflow.note");
    std::fs::write(&test_file, content).unwrap();

    let importer = memos_rs::import_export::tomboy::TomboyImporter::new(test_dir.to_str().unwrap());

    let notes = importer.import_all().unwrap();
    assert_eq!(notes.len(), 1);

    let note = &notes[0];
    assert_eq!(note.title, "Workflow Test");
    assert!(note.content.contains("# Workflow Test Note"));
    assert!(note.content.contains("2022-08-04T22:27:00+00:00"));
    assert!(note
        .content
        .contains("This is a **test** of the full import workflow"));
    assert!(note
        .content
        .contains("[https://wiki.gnome.org/Apps/Tomboy](https://wiki.gnome.org/Apps/Tomboy)"));
    assert_eq!(note.tags, vec!["workflow", "test"]);

    let memo_note = note.clone().to_memo_rs_note();
    assert_eq!(memo_note.title, "Workflow Test");
    assert_eq!(memo_note.tags, vec!["workflow", "test"]);
    assert!(memo_note.metadata.get("tomboy").is_some());

    std::fs::remove_file(test_file).ok();
    std::fs::remove_dir(&test_dir).ok();
}

#[test]
fn test_system_notebook_tag_parsing() {
    let content = r#"<?xml version="1.0" encoding="utf-8"?>
<note version="0.3">
  <title>Notebook Test</title>
  <text xml:space="preserve"><note-content version="0.1">Test content</note-content></text>
  <create-date>2022-08-04T22:27:09.7663000+08:00</create-date>
  <last-change-date>2022-08-04T22:30:04.7620040+08:00</last-change-date>
  <tags>
    <tag>system:notebook:MyNotebook</tag>
    <tag>regular-tag</tag>
  </tags>
</note>"#;

    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("memos-rs-test-notebook-12345");
    std::fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("test-notebook.note");
    std::fs::write(&test_file, content).unwrap();

    let note = memos_rs::import_export::tomboy::TomboyNote::parse_xml(&test_file).unwrap();

    assert_eq!(note.title, "Notebook Test");
    assert_eq!(note.tags, vec!["system:notebook:MyNotebook", "regular-tag"]);

    let memo_note = note.to_memo_rs_note();
    assert_eq!(memo_note.title, "Notebook Test");
    assert!(memo_note.metadata.get("tomboy").is_some());

    std::fs::remove_file(test_file).ok();
    std::fs::remove_dir(&test_dir).ok();
}

#[test]
fn test_my_tomboy_2_note() {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sample/my-tomboy-2.note");

    if !sample_path.exists() {
        eprintln!("Sample file not found at {:?}", sample_path);
        return;
    }

    let note = memos_rs::import_export::tomboy::TomboyNote::parse_xml(&sample_path).unwrap();

    assert_eq!(note.title, "Matlab: Embracing Complexity");

    // Check date transformation
    assert!(note.content.contains("2013-06-18T10:58:00+00:00"));

    // Check bold text transformation
    assert!(note.content.contains("**Platform for Collaboration**"));
    assert!(note.content.contains("**Modeling & Simulation:**"));

    // Check list items
    assert!(note.content.contains("\n- Car Adaptive cruise control:"));
    assert!(note
        .content
        .contains("\n- Neural Imaging of Brain activity"));

    // Check tags
    assert!(note
        .tags
        .contains(&"system:notebook:Vision_Project".to_string()));
}

#[test]
fn test_datetime_and_list_transformation() {
    let content = r#"<?xml version="1.0" encoding="utf-8"?>
<note version="0.3">
  <title>Datetime and List Test</title>
  <text xml:space="preserve"><note-content version="0.1">
<datetime>Thursday, August 4, 2022, 10:27 PM</datetime>

This is a list:
<list>
<list-item>First item</list-item>
<list-item>Second item</list-item>
<list-item>Third item</list-item>
</list>

End of note.
  </note-content></text>
  <create-date>2022-08-04T22:27:09.7663000+08:00</create-date>
  <last-change-date>2022-08-04T22:30:04.7620040+08:00</last-change-date>
  <tags>
    <tag>test</tag>
  </tags>
</note>"#;

    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("memos-rs-test-datetime-list-12345");
    std::fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("test-datetime-list.note");
    std::fs::write(&test_file, content).unwrap();

    let importer = memos_rs::import_export::tomboy::TomboyImporter::new(test_dir.to_str().unwrap());

    let notes = importer.import_all().unwrap();
    assert_eq!(notes.len(), 1);

    let note = &notes[0];
    assert_eq!(note.title, "Datetime and List Test");

    // Check datetime transformation
    assert!(note.content.contains("2022-08-04T22:27:00+00:00"));

    // Check list transformation
    assert!(note.content.contains("\n- First item"));
    assert!(note.content.contains("\n- Second item"));
    assert!(note.content.contains("\n- Third item"));

    assert_eq!(note.tags, vec!["test"]);

    std::fs::remove_file(test_file).ok();
    std::fs::remove_dir(&test_dir).ok();
}
