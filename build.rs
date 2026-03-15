use std::env;
use std::fs;
use std::path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let version = get_version();

    let dest_path = path::Path::new(&out_dir).join("version.rs");
    fs::write(
        &dest_path,
        format!(
            r#"pub const VERSION: &str = "{}";
pub const VERSION_SHORT: &str = "{}";"#,
            version,
            version.trim_start_matches('v')
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    let dist_path = path::Path::new(&manifest_dir).join("dist");

    if !dist_path.exists() {
        println!("Warning: dist directory not found. Frontend must be built first.");
    }
}

fn get_version() -> String {
    if let Ok(version) = env::var("CARGO_PKG_VERSION") {
        if !version.is_empty() {
            return version;
        }
    }

    if let Ok(output) = std::process::Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
    {
        if !output.stdout.is_empty() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    "unknown".to_string()
}
