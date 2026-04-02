use std::fs;
use std::process::Command;
use std::time::Instant;

fn start_server(port: u16, log_file: &str) -> std::process::Child {
    println!("Starting server on port {}...", port);
    let log_path = format!("/tmp/memos-rs-test-{}.log", log_file);

    // Clear old log file
    let _ = fs::remove_file(&log_path);

    let child = Command::new("cargo")
        .args(&["run", "--", "--port", &port.to_string()])
        .stdout(fs::File::create(&log_path).expect("Failed to create log file"))
        .spawn()
        .expect("Failed to start server");

    std::thread::sleep(std::time::Duration::from_secs(4));
    println!("Server started, PID: {:?}", child.id());

    child
}

fn stop_server(child: &mut std::process::Child) {
    println!("Stopping server...");
    let _ = child.kill();
    let _ = child.wait();
    println!("Server stopped");
}

fn register_user(port: u16, username: &str, email: &str, password: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("http://localhost:{}/api/v1/register", port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .expect("Failed to register");

    let body = response.text().expect("Failed to get response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Failed to parse JSON");

    json["token"].as_str().unwrap().to_string()
}

fn create_note(port: u16, token: &str, title: &str, content: &str) -> serde_json::Value {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("http://localhost:{}/api/v1/notes", port))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "title": title,
            "content": content
        }))
        .send()
        .expect("Failed to create note");

    let body = response.text().expect("Failed to get response body");
    serde_json::from_str(&body).expect("Failed to parse JSON")
}

#[test]
#[ignore]
fn test_user_registration_and_note_creation_with_embedding() {
    let port = 3001;
    let log_file = "reg_note_test";

    println!("\n=== Starting Test: User Registration and Note Creation ===\n");

    let mut server = start_server(port, log_file);

    println!("[TEST] Registering user...");
    let start_reg = Instant::now();
    let token = register_user(port, "testuser", "test@example.com", "testpass123");
    let reg_duration = start_reg.elapsed();
    println!("[TEST] User registered in {:?}", reg_duration);
    println!("[TEST] Token: {}...", &token[..20]);

    println!("\n[TEST] Creating note with embedding...");
    let start_create = Instant::now();
    let note = create_note(
        port, 
        &token, 
        "Test Note with Embedding", 
        "This is a test note to verify that embeddings are calculated correctly. The embedding should be cached for future notes with similar content."
    );
    let create_duration = start_create.elapsed();
    println!("[TEST] Note created in {:?}", create_duration);
    println!("[TEST] Note ID: {}", note["id"].as_str().unwrap());

    println!("\n[TEST] Creating second note (should use cache)...");
    let start_create2 = Instant::now();
    let note2 = create_note(
        port,
        &token,
        "Second Test Note",
        "Another test note to verify embedding caching works properly.",
    );
    let create_duration2 = start_create2.elapsed();
    println!("[TEST] Second note created in {:?}", create_duration2);
    println!("[TEST] Note ID: {}", note2["id"].as_str().unwrap());

    stop_server(&mut server);

    println!("\n=== Server Log ===");
    let log_path = format!("/tmp/memos-rs-test-{}.log", log_file);
    if let Ok(log) = fs::read_to_string(&log_path) {
        let lines: Vec<&str> = log.lines().rev().take(50).collect();
        for line in lines.iter().rev() {
            println!("{}", line);
        }
    }

    assert!(!token.is_empty(), "Token should not be empty");
    assert_eq!(note["title"].as_str().unwrap(), "Test Note with Embedding");
    assert!(note["content"].as_str().unwrap().contains("test note"));

    println!("\n=== Test Results ===");
    println!("Registration time: {:?}", reg_duration);
    println!("Note creation time (cache miss): {:?}", create_duration);
    println!("Note creation time (cache hit): {:?}", create_duration2);

    println!("\n=== Embedding Performance Log ===");
    if let Ok(log) = fs::read_to_string(&log_path) {
        for line in log.lines() {
            if line.contains("[EMBED]") {
                println!("{}", line);
            }
        }
    }

    println!("\n=== Test Passed ===\n");
}

#[test]
#[ignore]
fn test_performance_with_multiple_notes() {
    let port = 3002;
    let log_file = "perf_test";

    println!("\n=== Starting Performance Test: Multiple Notes ===\n");

    let mut server = start_server(port, log_file);

    let token = register_user(port, "perfuser", "perf@example.com", "testpass123");
    println!("[TEST] Registered user, token: {}...", &token[..20]);

    let num_notes = 5;
    let mut durations = Vec::new();

    for i in 0..num_notes {
        let content = format!("Performance test note number {}. This note tests the embedding cache and performance. Note ID: {}", i, i);

        println!("[TEST] Creating note {}...", i);
        let start = Instant::now();
        let _note = create_note(port, &token, &format!("Performance Note {}", i), &content);
        let duration = start.elapsed();
        durations.push(duration);

        println!("[TEST] Note {} created in {:?}", i, duration);
    }

    stop_server(&mut server);

    let avg_duration: std::time::Duration =
        durations.iter().sum::<std::time::Duration>() / num_notes as u32;

    println!("\n=== Performance Results ===");
    for (i, duration) in durations.iter().enumerate() {
        println!("Note {}: {:?}", i, duration);
    }
    println!("Average: {:?}", avg_duration);

    println!("\n=== Embedding Performance Log ===");
    let log_path = format!("/tmp/memos-rs-test-{}.log", log_file);
    if let Ok(log) = fs::read_to_string(&log_path) {
        for line in log.lines() {
            if line.contains("[EMBED]") {
                println!("{}", line);
            }
        }
    }

    println!("\n=== Performance Test Passed ===\n");
}
