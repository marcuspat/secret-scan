use crate::context::ContextFilter;
use crate::entropy::shannon_entropy;
use crate::patterns::get_all_patterns_owned;
use crate::Finding;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

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
    context_filter: ContextFilter,
}

impl Scanner {
    pub fn new() -> Result<Self, ScannerError> {
        Ok(Scanner {
            patterns: get_all_patterns_owned(),
            context_filter: ContextFilter::new(),
        })
    }

    pub fn with_patterns(patterns: Vec<(String, Regex)>) -> Result<Self, ScannerError> {
        let mut pattern_map = HashMap::new();
        for (name, pattern) in patterns {
            pattern_map.insert(name, pattern);
        }

        Ok(Scanner {
            patterns: pattern_map,
            context_filter: ContextFilter::new(),
        })
    }

    /// Create scanner with custom context filter
    pub fn with_context_filter(context_filter: ContextFilter) -> Result<Self, ScannerError> {
        Ok(Scanner {
            patterns: get_all_patterns_owned(),
            context_filter,
        })
    }

    /// Set context filter for this scanner
    pub fn set_context_filter(&mut self, context_filter: ContextFilter) {
        self.context_filter = context_filter;
    }

    /// Get a reference to the context filter
    pub fn context_filter(&self) -> &ContextFilter {
        &self.context_filter
    }

    pub fn scan_directory(&self, path: &Path) -> Result<Vec<Finding>, ScannerError> {
        self.scan_directory_optimized(path)
    }

    /// Optimized parallel scanning with rayon
    pub fn scan_directory_optimized(&self, path: &Path) -> Result<Vec<Finding>, ScannerError> {
        // Build file list
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
            .filter(|entry| {
                // Skip .git directory files
                if entry.path().components().any(|c| c.as_os_str() == ".git") {
                    return false;
                }
                // Apply context filtering
                !self.context_filter.should_skip_path(entry.path())
            })
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();

        // Create owned copies for thread safety
        let patterns = Arc::new(self.patterns.clone());
        let context_filter = Arc::new(self.context_filter.clone());

        // Process files in parallel using rayon
        let all_findings: Vec<Finding> = walker
            .par_iter()
            .filter_map(|file_path| {
                Self::scan_file_static(file_path, &patterns, &context_filter).ok()
            })
            .flatten()
            .collect();

        Ok(all_findings)
    }

    /// Legacy parallel scanning method (for comparison)
    pub fn scan_directory_rayon(&self, path: &Path) -> Result<Vec<Finding>, ScannerError> {
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
                if entry.path().components().any(|c| c.as_os_str() == ".git") {
                    return false;
                }

                // Apply context filtering
                !self.context_filter.should_skip_path(entry.path())
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
        self.scan_file_streaming(file_path)
    }

    /// Memory-optimized file scanning using BufReader and streaming
    fn scan_file_streaming(&self, file_path: &Path) -> Result<Vec<Finding>, ScannerError> {
        let file = File::open(file_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        // For very large files (>10MB), use chunked streaming
        if file_size > 10 * 1024 * 1024 {
            self.scan_large_file_chunked(file_path, file)
        } else {
            self.scan_file_buffered(file_path, file)
        }
    }

    /// Scan regular files with BufReader for memory efficiency
    fn scan_file_buffered(
        &self,
        file_path: &Path,
        file: File,
    ) -> Result<Vec<Finding>, ScannerError> {
        let reader = BufReader::with_capacity(8192, file); // 8KB buffer
        let mut findings = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line_number = line_number + 1; // Convert to 1-indexed
            let line = line_result?;

            // Check against all patterns
            for (pattern_name, pattern) in &self.patterns {
                if let Some(mat) = pattern.find(&line) {
                    let matched_text = mat.as_str().to_string();

                    // Apply context filtering for line content
                    if self.context_filter.should_skip_line(&line, &matched_text) {
                        continue;
                    }

                    let entropy = shannon_entropy(&matched_text);

                    findings.push(Finding {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        line_content: line.clone(),
                        pattern_name: pattern_name.clone(),
                        matched_text,
                        entropy: Some(entropy),
                    });
                }
            }
        }

        Ok(findings)
    }

    /// Scan very large files in chunks to minimize memory usage
    fn scan_large_file_chunked(
        &self,
        file_path: &Path,
        mut file: File,
    ) -> Result<Vec<Finding>, ScannerError> {
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

        let mut findings = Vec::new();
        let mut global_line_number = 0;
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut overlap_buffer = String::new();

        loop {
            // Read chunk
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }

            // Convert to string, handling potential UTF-8 boundary issues
            let chunk = match std::str::from_utf8(&buffer[..bytes_read]) {
                Ok(s) => s.to_string(),
                Err(e) => {
                    // Handle UTF-8 boundary issues by truncating at last valid UTF-8 boundary
                    let valid_up_to = e.valid_up_to();
                    if valid_up_to > 0 {
                        // Seek back to handle remaining bytes in next iteration
                        let seek_back = bytes_read - valid_up_to;
                        file.seek(SeekFrom::Current(-(seek_back as i64)))?;
                        String::from_utf8_lossy(&buffer[..valid_up_to]).into_owned()
                    } else {
                        continue; // Skip malformed chunk
                    }
                }
            };

            // Prepend overlap from previous chunk
            let content = if overlap_buffer.is_empty() {
                chunk
            } else {
                format!("{}{}", overlap_buffer, chunk)
            };

            // Process lines in this chunk
            let lines: Vec<&str> = content.lines().collect();
            let lines_to_process = if bytes_read < CHUNK_SIZE {
                // Last chunk, process all lines
                lines.len()
            } else {
                // Not last chunk, save last line for overlap
                lines.len().saturating_sub(1)
            };

            for line in lines.iter().take(lines_to_process) {
                global_line_number += 1;

                // Check against all patterns
                for (pattern_name, pattern) in &self.patterns {
                    if let Some(mat) = pattern.find(line) {
                        let matched_text = mat.as_str().to_string();

                        // Apply context filtering for line content
                        if self.context_filter.should_skip_line(line, &matched_text) {
                            continue;
                        }

                        let entropy = shannon_entropy(&matched_text);

                        findings.push(Finding {
                            file_path: file_path.to_path_buf(),
                            line_number: global_line_number,
                            line_content: line.to_string(),
                            pattern_name: pattern_name.clone(),
                            matched_text,
                            entropy: Some(entropy),
                        });
                    }
                }
            }

            // Prepare overlap for next chunk
            overlap_buffer = if lines_to_process < lines.len() {
                lines[lines_to_process].to_string()
            } else {
                String::new()
            };

            // If we read less than chunk size, we've reached EOF
            if bytes_read < CHUNK_SIZE {
                // Process the final overlap line if it exists
                if !overlap_buffer.is_empty() {
                    global_line_number += 1;

                    for (pattern_name, pattern) in &self.patterns {
                        if let Some(mat) = pattern.find(&overlap_buffer) {
                            let matched_text = mat.as_str().to_string();

                            if !self
                                .context_filter
                                .should_skip_line(&overlap_buffer, &matched_text)
                            {
                                let entropy = shannon_entropy(&matched_text);

                                findings.push(Finding {
                                    file_path: file_path.to_path_buf(),
                                    line_number: global_line_number,
                                    line_content: overlap_buffer.clone(),
                                    pattern_name: pattern_name.clone(),
                                    matched_text,
                                    entropy: Some(entropy),
                                });
                            }
                        }
                    }
                }
                break;
            }
        }

        Ok(findings)
    }

    /// Static version of scan_file for use with Arc references in parallel processing
    fn scan_file_static(
        file_path: &Path,
        patterns: &HashMap<String, Regex>,
        context_filter: &ContextFilter,
    ) -> Result<Vec<Finding>, ScannerError> {
        Self::scan_file_static_streaming(file_path, patterns, context_filter)
    }

    /// Memory-optimized static scan_file method
    fn scan_file_static_streaming(
        file_path: &Path,
        patterns: &HashMap<String, Regex>,
        context_filter: &ContextFilter,
    ) -> Result<Vec<Finding>, ScannerError> {
        let file = File::open(file_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        // For very large files (>10MB), use chunked streaming
        if file_size > 10 * 1024 * 1024 {
            Self::scan_file_static_chunked(file_path, file, patterns, context_filter)
        } else {
            Self::scan_file_static_buffered(file_path, file, patterns, context_filter)
        }
    }

    /// Static buffered file scanning
    fn scan_file_static_buffered(
        file_path: &Path,
        file: File,
        patterns: &HashMap<String, Regex>,
        context_filter: &ContextFilter,
    ) -> Result<Vec<Finding>, ScannerError> {
        let reader = BufReader::with_capacity(8192, file);
        let mut findings = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line_number = line_number + 1; // Convert to 1-indexed
            let line = line_result?;

            // Check against all patterns
            for (pattern_name, pattern) in patterns {
                if let Some(mat) = pattern.find(&line) {
                    let matched_text = mat.as_str().to_string();

                    // Apply context filtering for line content
                    if context_filter.should_skip_line(&line, &matched_text) {
                        continue;
                    }

                    let entropy = shannon_entropy(&matched_text);

                    findings.push(Finding {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        line_content: line.clone(),
                        pattern_name: pattern_name.clone(),
                        matched_text,
                        entropy: Some(entropy),
                    });
                }
            }
        }

        Ok(findings)
    }

    /// Static chunked file scanning for very large files
    fn scan_file_static_chunked(
        file_path: &Path,
        mut file: File,
        patterns: &HashMap<String, Regex>,
        context_filter: &ContextFilter,
    ) -> Result<Vec<Finding>, ScannerError> {
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

        let mut findings = Vec::new();
        let mut global_line_number = 0;
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut overlap_buffer = String::new();

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = match std::str::from_utf8(&buffer[..bytes_read]) {
                Ok(s) => s.to_string(),
                Err(e) => {
                    let valid_up_to = e.valid_up_to();
                    if valid_up_to > 0 {
                        let seek_back = bytes_read - valid_up_to;
                        file.seek(SeekFrom::Current(-(seek_back as i64)))?;
                        String::from_utf8_lossy(&buffer[..valid_up_to]).into_owned()
                    } else {
                        continue;
                    }
                }
            };

            let content = if overlap_buffer.is_empty() {
                chunk
            } else {
                format!("{}{}", overlap_buffer, chunk)
            };

            let lines: Vec<&str> = content.lines().collect();
            let lines_to_process = if bytes_read < CHUNK_SIZE {
                lines.len()
            } else {
                lines.len().saturating_sub(1)
            };

            for line in lines.iter().take(lines_to_process) {
                global_line_number += 1;

                for (pattern_name, pattern) in patterns {
                    if let Some(mat) = pattern.find(line) {
                        let matched_text = mat.as_str().to_string();

                        if !context_filter.should_skip_line(line, &matched_text) {
                            let entropy = shannon_entropy(&matched_text);

                            findings.push(Finding {
                                file_path: file_path.to_path_buf(),
                                line_number: global_line_number,
                                line_content: line.to_string(),
                                pattern_name: pattern_name.clone(),
                                matched_text,
                                entropy: Some(entropy),
                            });
                        }
                    }
                }
            }

            overlap_buffer = if lines_to_process < lines.len() {
                lines[lines_to_process].to_string()
            } else {
                String::new()
            };

            if bytes_read < CHUNK_SIZE {
                if !overlap_buffer.is_empty() {
                    global_line_number += 1;

                    for (pattern_name, pattern) in patterns {
                        if let Some(mat) = pattern.find(&overlap_buffer) {
                            let matched_text = mat.as_str().to_string();

                            if !context_filter.should_skip_line(&overlap_buffer, &matched_text) {
                                let entropy = shannon_entropy(&matched_text);

                                findings.push(Finding {
                                    file_path: file_path.to_path_buf(),
                                    line_number: global_line_number,
                                    line_content: overlap_buffer.clone(),
                                    pattern_name: pattern_name.clone(),
                                    matched_text,
                                    entropy: Some(entropy),
                                });
                            }
                        }
                    }
                }
                break;
            }
        }

        Ok(findings)
    }

    /// Get estimated memory usage for scanning a given number of files
    pub fn estimate_memory_usage(num_files: usize, avg_file_size: usize) -> (f64, String) {
        // Base memory for patterns and context filter (roughly 50KB)
        let base_memory = 50.0 * 1024.0;

        // Per-file overhead in parallel processing (roughly 16KB per thread)
        let thread_overhead = 16.0 * 1024.0 * rayon::current_num_threads() as f64;

        // Buffer memory per thread (8KB buffer * num_threads)
        let buffer_memory = 8.0 * 1024.0 * rayon::current_num_threads() as f64;

        // Estimated findings storage (assume 1% of files have findings, avg 200 bytes per finding)
        let findings_memory = (num_files as f64 * 0.01) * 200.0;

        // Large file chunking overhead (1MB buffer if any large files)
        let large_file_overhead = if avg_file_size > 10 * 1024 * 1024 {
            1024.0 * 1024.0 // 1MB chunk buffer
        } else {
            0.0
        };

        let total_bytes =
            base_memory + thread_overhead + buffer_memory + findings_memory + large_file_overhead;
        let total_mb = total_bytes / (1024.0 * 1024.0);

        let status = if total_mb > 100.0 {
            "WARNING: Estimated memory usage exceeds 100MB target".to_string()
        } else {
            "✓ Memory usage within target (<100MB)".to_string()
        };

        (total_mb, status)
    }

    /// Monitor actual memory usage during scanning (requires external memory monitoring)
    pub fn get_memory_stats() -> Option<(f64, f64)> {
        // This would require external crates like `psutil` or system calls
        // For now, return None to indicate it's not implemented
        // In a real implementation, this would return (current_mb, peak_mb)
        None
    }
}
