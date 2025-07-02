use regex::Regex;
use std::collections::HashMap;

pub fn aws_access_key_pattern() -> Regex {
    Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()
}

pub fn github_token_pattern() -> Regex {
    Regex::new(r"ghp_[0-9A-Za-z]{36}").unwrap()
}

pub fn google_api_key_pattern() -> Regex {
    Regex::new(r"AIza[0-9A-Za-z\-_]{33,}").unwrap()
}

pub fn get_all_patterns() -> HashMap<String, Regex> {
    let mut patterns = HashMap::new();

    patterns.insert("AWS Access Key".to_string(), aws_access_key_pattern());
    patterns.insert("GitHub Token".to_string(), github_token_pattern());
    patterns.insert("Google API Key".to_string(), google_api_key_pattern());

    patterns
}
