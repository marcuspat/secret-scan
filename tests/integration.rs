use secretscanner::Scanner;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_end_to_end_secrets_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(temp_path)
        .output()
        .unwrap();

    // Create a realistic project structure
    fs::create_dir_all(temp_path.join("src")).unwrap();
    fs::create_dir_all(temp_path.join("config")).unwrap();

    // Create files with secrets
    fs::write(
        temp_path.join("src/config.rs"),
        r#"
pub const DATABASE_URL: &str = "postgres://user:pass@localhost/db";
pub const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
pub const API_ENDPOINT: &str = "https://api.example.com";
"#,
    )
    .unwrap();

    fs::write(
        temp_path.join("config/production.env"),
        r#"
GITHUB_TOKEN=ghp_1234567890abcdefghijklmnopqrstuvwxyz
GOOGLE_API_KEY=AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI
DATABASE_PASSWORD=secure_password_123
"#,
    )
    .unwrap();

    // Create normal files without secrets
    fs::write(
        temp_path.join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
    let api_key = std::env::var("API_KEY").expect("API_KEY not set");
    println!("Using API key: {}", api_key);
}
"#,
    )
    .unwrap();

    // Scan the directory
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();

    // Verify we found the expected secrets
    assert!(
        findings.len() >= 3,
        "Should find at least 3 secrets, found {}",
        findings.len()
    );

    // Check that we found specific pattern types
    let aws_keys: Vec<_> = findings
        .iter()
        .filter(|f| f.pattern_name == "AWS Access Key")
        .collect();
    let github_tokens: Vec<_> = findings
        .iter()
        .filter(|f| f.pattern_name == "GitHub Token")
        .collect();
    let google_keys: Vec<_> = findings
        .iter()
        .filter(|f| f.pattern_name == "Google API Key")
        .collect();

    assert_eq!(aws_keys.len(), 1, "Should find exactly 1 AWS access key");
    assert_eq!(github_tokens.len(), 1, "Should find exactly 1 GitHub token");
    assert_eq!(google_keys.len(), 1, "Should find exactly 1 Google API key");

    // Verify file paths are correct
    assert!(aws_keys[0].file_path.ends_with("src/config.rs"));
    assert!(github_tokens[0]
        .file_path
        .ends_with("config/production.env"));
    assert!(google_keys[0].file_path.ends_with("config/production.env"));
}

#[test]
fn test_gitignore_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(temp_path)
        .output()
        .unwrap();

    // Create .gitignore
    fs::write(
        temp_path.join(".gitignore"),
        r#"
# Ignore sensitive files
*.env
secrets/
config/private.conf
"#,
    )
    .unwrap();

    // Create directories
    fs::create_dir_all(temp_path.join("secrets")).unwrap();
    fs::create_dir_all(temp_path.join("config")).unwrap();

    // Create ignored files with secrets
    fs::write(
        temp_path.join("secrets/prod.env"),
        "AWS_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE",
    )
    .unwrap();

    fs::write(
        temp_path.join("config/private.conf"),
        "github_token=ghp_1234567890abcdefghijklmnopqrstuvwxyz",
    )
    .unwrap();

    fs::write(
        temp_path.join("development.env"),
        "GOOGLE_API_KEY=AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI",
    )
    .unwrap();

    // Create non-ignored file with secret
    fs::write(
        temp_path.join("config/public.conf"),
        "api_token=ghp_abcdefghijklmnopqrstuvwxyz1234567890",
    )
    .unwrap();

    // Scan the directory
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();

    // Should only find the secret in the non-ignored file
    assert_eq!(
        findings.len(),
        1,
        "Should find exactly 1 secret (others ignored), found {}",
        findings.len()
    );
    assert!(findings[0].file_path.ends_with("config/public.conf"));
    assert_eq!(findings[0].pattern_name, "GitHub Token");
}

#[test]
fn test_false_positive_rate() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files with strings that look like secrets but aren't
    let false_positive_content = r#"
// These should NOT be flagged as secrets
let placeholder_key = "AKIA_PLACEHOLDER_KEY_123";
let example_token = "ghp_example_token_not_real";
let test_api_key = "AIza_test_key_for_documentation";
let fake_secret = "fake_secret_value";
let example_password = "example_password_123";
let demo_token = "demo_token_abc123";
let sample_api = "sample_api_key_xyz789";

// Template strings
const TEMPLATE = "Your API key: ${API_KEY}";
const EXAMPLE = "AKIA${RANDOM_CHARS}";

// Comments and documentation
# AWS_ACCESS_KEY=your_key_here
# GITHUB_TOKEN=your_token_here
// TODO: Replace with real API_KEY

// Short strings that might match patterns partially
let short = "AKIA";
let partial = "ghp_";
let incomplete = "AIza";
"#;

    fs::write(temp_path.join("examples.rs"), false_positive_content).unwrap();

    // Create a few files with legitimate content
    fs::write(
        temp_path.join("readme.md"),
        r#"
# Secret Scanner

This tool scans for secrets like:
- AWS access keys (format: AKIA...)
- GitHub tokens (format: ghp_...)
- Google API keys (format: AIza...)

## Usage

```bash
secretscanner /path/to/scan
```
"#,
    )
    .unwrap();

    fs::write(
        temp_path.join("constants.rs"),
        r#"
pub const MAX_FILE_SIZE: usize = 1024 * 1024;
pub const DEFAULT_TIMEOUT: u64 = 30;
pub const API_VERSION: &str = "v1";
"#,
    )
    .unwrap();

    // Scan the directory
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();

    // Calculate false positive rate
    let total_lines = false_positive_content.lines().count() + 20; // approximate total lines
    let false_positive_rate = findings.len() as f64 / total_lines as f64;

    // Should have very low false positive rate (< 10%)
    assert!(
        false_positive_rate < 0.1,
        "False positive rate too high: {:.2}% ({} findings in ~{} lines)",
        false_positive_rate * 100.0,
        findings.len(),
        total_lines
    );

    // Print findings for debugging if any
    if !findings.is_empty() {
        eprintln!("False positive findings:");
        for finding in &findings {
            eprintln!("  - {}: {}", finding.pattern_name, finding.matched_text);
        }
    }
}

#[test]
fn test_nested_directory_scanning() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create deep nested structure
    let deep_path = temp_path.join("src/components/auth/providers");
    fs::create_dir_all(&deep_path).unwrap();

    // Place secret in deeply nested file
    fs::write(
        deep_path.join("oauth.rs"),
        r#"
use std::env;

pub struct OAuthProvider {
    client_id: String,
    client_secret: String,
}

impl OAuthProvider {
    pub fn new() -> Self {
        Self {
            client_id: "public_client_id".to_string(),
            client_secret: "AKIAIOSFODNN7EXAMPLE".to_string(), // This should be found
        }
    }
}
"#,
    )
    .unwrap();

    // Scan from root
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].file_path.to_string_lossy().contains("oauth.rs"));
    assert_eq!(findings[0].pattern_name, "AWS Access Key");
}

#[test]
fn test_performance_with_large_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a large file with one secret at the end
    let mut large_content = String::new();
    for i in 0..1000 {
        large_content.push_str(&format!("// Line {} with normal content\n", i));
        large_content.push_str(&format!("let variable_{} = \"some_value_{}\";\n", i, i));
    }
    large_content.push_str("let aws_key = \"AKIAIOSFODNN7EXAMPLE\"; // Secret at the end\n");

    fs::write(temp_path.join("large_file.rs"), large_content).unwrap();

    // Time the scan
    let start = std::time::Instant::now();
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();
    let duration = start.elapsed();

    // Should find the secret
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].pattern_name, "AWS Access Key");
    assert_eq!(findings[0].line_number, 2001); // At the end

    // Should be reasonably fast (less than 1 second for this test)
    assert!(
        duration.as_secs() < 1,
        "Scanning took too long: {:?}",
        duration
    );
}

#[test]
fn test_binary_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a binary file (should be skipped or handled gracefully)
    let binary_data: Vec<u8> = vec![0, 1, 2, 3, 255, 254, 253];
    fs::write(temp_path.join("binary.bin"), binary_data).unwrap();

    // Create a text file with secret
    fs::write(
        temp_path.join("text.txt"),
        "github_token=ghp_1234567890abcdefghijklmnopqrstuvwxyz",
    )
    .unwrap();

    // Scan should not crash and should find the text secret
    let scanner = Scanner::new().unwrap();
    let findings = scanner.scan_directory(temp_path).unwrap();

    // Should find only the text secret, binary file should be handled gracefully
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file_path.ends_with("text.txt"));
}
