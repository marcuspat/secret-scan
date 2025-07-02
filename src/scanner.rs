use crate::entropy::shannon_entropy;
use crate::patterns::get_all_patterns;
use crate::Finding;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ScannerError {
    message: String,
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Scanner error: {}", self.message)
    }
}

impl Error for ScannerError {}

impl From<std::io::Error> for ScannerError {
    fn from(err: std::io::Error) -> Self {
        ScannerError {
            message: format!("IO error: {}", err),
        }
    }
}

pub struct Scanner {
    patterns: HashMap<String, Regex>,
}

impl Scanner {
    pub fn new() -> Result<Self, ScannerError> {
        Ok(Scanner {
            patterns: get_all_patterns(),
        })
    }

    pub fn with_patterns(patterns: Vec<(String, Regex)>) -> Result<Self, ScannerError> {
        let mut pattern_map = HashMap::new();
        for (name, pattern) in patterns {
            pattern_map.insert(name, pattern);
        }

        Ok(Scanner {
            patterns: pattern_map,
        })
    }

    pub fn scan_directory(&self, path: &Path) -> Result<Vec<Finding>, ScannerError> {
        let mut findings = Vec::new();

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(false)
            .parents(true)
            .ignore(true)
            .build()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect::<Vec<_>>();

        // Process files in parallel with rayon
        let parallel_findings: Vec<Vec<Finding>> = walker
            .par_iter()
            .filter(|entry| {
                // Skip .git directory files
                !entry.path().components().any(|c| c.as_os_str() == ".git")
            })
            .map(|entry| self.scan_file(entry.path()).unwrap_or_else(|_| Vec::new()))
            .collect();

        // Flatten results
        for file_findings in parallel_findings {
            findings.extend(file_findings);
        }

        Ok(findings)
    }

    fn scan_file(&self, file_path: &Path) -> Result<Vec<Finding>, ScannerError> {
        let content = fs::read_to_string(file_path)?;
        let mut findings = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            let line_number = line_number + 1; // 1-indexed

            // Check against all patterns
            for (pattern_name, pattern) in &self.patterns {
                if let Some(mat) = pattern.find(line) {
                    let matched_text = mat.as_str().to_string();
                    let entropy = shannon_entropy(&matched_text);

                    findings.push(Finding {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        line_content: line.to_string(),
                        pattern_name: pattern_name.clone(),
                        matched_text,
                        entropy: Some(entropy),
                    });
                }
            }
        }

        Ok(findings)
    }
}
