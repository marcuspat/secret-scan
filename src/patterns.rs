use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref AWS_ACCESS_KEY: Regex = Regex::new(r"AKIA[0-9A-Z]{16}").unwrap();
    static ref GITHUB_TOKEN: Regex = Regex::new(r"ghp_[0-9A-Za-z]{36}").unwrap();
    static ref GOOGLE_API_KEY: Regex = Regex::new(r"AIza[0-9A-Za-z\-_]{33,}").unwrap();
    static ref JWT_TOKEN: Regex =
        Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap();
    static ref RSA_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN RSA PRIVATE KEY-----").unwrap();
    static ref EC_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN EC PRIVATE KEY-----").unwrap();
    static ref POSTGRES_URL: Regex = Regex::new(r"postgres://[^\s\x22\x27]+").unwrap();
    static ref MYSQL_URL: Regex = Regex::new(r"mysql://[^\s\x22\x27]+").unwrap();
    static ref MONGODB_URL: Regex = Regex::new(r"mongodb://[^\s\x22\x27]+").unwrap();
    static ref OPENAI_API_KEY: Regex = Regex::new(r"sk-[0-9A-Za-z]{32,48}").unwrap();
    static ref ALL_PATTERNS: HashMap<String, &'static Regex> = {
        let mut patterns = HashMap::new();
        patterns.insert("AWS Access Key".to_string(), &*AWS_ACCESS_KEY);
        patterns.insert("GitHub Token".to_string(), &*GITHUB_TOKEN);
        patterns.insert("Google API Key".to_string(), &*GOOGLE_API_KEY);
        patterns.insert("JWT Token".to_string(), &*JWT_TOKEN);
        patterns.insert("RSA Private Key".to_string(), &*RSA_PRIVATE_KEY);
        patterns.insert("EC Private Key".to_string(), &*EC_PRIVATE_KEY);
        patterns.insert("PostgreSQL URL".to_string(), &*POSTGRES_URL);
        patterns.insert("MySQL URL".to_string(), &*MYSQL_URL);
        patterns.insert("MongoDB URL".to_string(), &*MONGODB_URL);
        patterns.insert("OpenAI API Key".to_string(), &*OPENAI_API_KEY);
        patterns
    };
}

pub fn get_all_patterns() -> &'static HashMap<String, &'static Regex> {
    &ALL_PATTERNS
}

pub fn get_all_patterns_owned() -> HashMap<String, Regex> {
    let mut patterns = HashMap::new();
    patterns.insert("AWS Access Key".to_string(), AWS_ACCESS_KEY.clone());
    patterns.insert("GitHub Token".to_string(), GITHUB_TOKEN.clone());
    patterns.insert("Google API Key".to_string(), GOOGLE_API_KEY.clone());
    patterns.insert("JWT Token".to_string(), JWT_TOKEN.clone());
    patterns.insert("RSA Private Key".to_string(), RSA_PRIVATE_KEY.clone());
    patterns.insert("EC Private Key".to_string(), EC_PRIVATE_KEY.clone());
    patterns.insert("PostgreSQL URL".to_string(), POSTGRES_URL.clone());
    patterns.insert("MySQL URL".to_string(), MYSQL_URL.clone());
    patterns.insert("MongoDB URL".to_string(), MONGODB_URL.clone());
    patterns.insert("OpenAI API Key".to_string(), OPENAI_API_KEY.clone());
    patterns
}
