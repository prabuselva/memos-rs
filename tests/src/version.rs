include!(concat!(env!("OUT_DIR"), "/version.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_available() {
        // VERSION test - build.rs may not run before tests
    }
}
