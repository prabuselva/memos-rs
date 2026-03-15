use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_path = Path::new(&manifest_dir).join("dist");

    if !dist_path.exists() {
        println!("Warning: dist directory not found. Frontend must be built first.");
    }
}
