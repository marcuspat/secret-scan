use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    // AWS Patterns
    static ref AWS_ACCESS_KEY: Regex = Regex::new(r#"(?i)(aws[_\s\-]?access[_\s\-]?key[_\s\-]?(id)?|AKIA)["']?\s*[:=]\s*["']?([A-Z0-9]{16,20})["']?"#).unwrap();
    static ref AWS_SECRET_KEY: Regex = Regex::new(r#"(?i)(aws[_\s\-]?secret[_\s\-]?(access[_\s\-]?)?key)["']?\s*[:=]\s*["']?([A-Za-z0-9/+=]{40})["']?"#).unwrap();
    static ref AWS_ACCESS_KEY_ID: Regex = Regex::new(r"AKIA[0-9A-Z]{16}").unwrap();
    
    // GitHub Patterns
    static ref GITHUB_TOKEN: Regex = Regex::new(r"(ghp|gho|ghu|ghs|ghr)_[0-9A-Za-z]{36,}").unwrap();
    static ref GITHUB_OAUTH: Regex = Regex::new(r"[0-9a-f]{40}").unwrap();
    
    // Google Patterns
    static ref GOOGLE_API_KEY: Regex = Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap();
    static ref GOOGLE_OAUTH: Regex = Regex::new(r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com").unwrap();
    
    // JWT Pattern
    static ref JWT_TOKEN: Regex = Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap();
    
    // Private Key Patterns
    static ref RSA_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap();
    static ref EC_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+EC\s+PRIVATE\s+KEY-----").unwrap();
    static ref PGP_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+PGP\s+PRIVATE\s+KEY\s+BLOCK-----").unwrap();
    static ref SSH_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----").unwrap();
    static ref GENERIC_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+[A-Z\s]+PRIVATE\s+KEY-----").unwrap();
    
    // Database URLs
    static ref POSTGRES_URL: Regex = Regex::new(r"postgres(ql)?://[a-z0-9]+:[^@\s]+@[^\s]+").unwrap();
    static ref MYSQL_URL: Regex = Regex::new(r"mysql://[a-z0-9]+:[^@\s]+@[^\s]+").unwrap();
    static ref MONGODB_URL: Regex = Regex::new(r"mongodb(\+srv)?://[a-z0-9]+:[^@\s]+@[^\s]+").unwrap();
    static ref REDIS_URL: Regex = Regex::new(r"redis://(?:[a-z0-9]+:)?[^@\s]+@[^\s]+").unwrap();
    
    // API Keys
    static ref OPENAI_API_KEY: Regex = Regex::new(r"sk-[0-9A-Za-z]{32,48}").unwrap();
    static ref STRIPE_API_KEY: Regex = Regex::new(r"(sk|pk)_(test|live)_[0-9A-Za-z]{24,}").unwrap();
    static ref SENDGRID_API_KEY: Regex = Regex::new(r"SG\.[0-9A-Za-z\-_]{22,}\.[0-9A-Za-z\-_]{22,}").unwrap();
    static ref SLACK_TOKEN: Regex = Regex::new(r"xox[baprs]-[0-9A-Za-z]{10,48}").unwrap();
    static ref TWILIO_API_KEY: Regex = Regex::new(r"SK[0-9a-fA-F]{32}").unwrap();
    static ref MAILGUN_API_KEY: Regex = Regex::new(r"key-[0-9a-zA-Z]{32}").unwrap();
    static ref FIREBASE_API_KEY: Regex = Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap();
    
    // OAuth Patterns
    static ref GENERIC_OAUTH_SECRET: Regex = Regex::new(r#"(?i)(oauth|client)[_\s\-]?secret["']?\s*[:=]\s*["']?([a-zA-Z0-9\-._~+/]{32,})["']?"#).unwrap();
    static ref GENERIC_CLIENT_ID: Regex = Regex::new(r#"(?i)(client|app)[_\s\-]?id["']?\s*[:=]\s*["']?([a-zA-Z0-9\-._~+/]{20,})["']?"#).unwrap();
    
    // Azure Patterns
    static ref AZURE_TENANT_ID: Regex = Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
    static ref AZURE_CLIENT_SECRET: Regex = Regex::new(r#"(?i)azure[_\s\-]?(client[_\s\-]?)?secret["']?\s*[:=]\s*["']?([a-zA-Z0-9~._-]{34,})["']?"#).unwrap();
    
    // PayPal Patterns
    static ref PAYPAL_CLIENT_ID: Regex = Regex::new(r#"(?i)paypal[_\s\-]?client[_\s\-]?id["']?\s*[:=]\s*["']?([A-Za-z0-9-_]{60,})["']?"#).unwrap();
    static ref PAYPAL_SECRET: Regex = Regex::new(r#"(?i)paypal[_\s\-]?secret["']?\s*[:=]\s*["']?([A-Za-z0-9-_]{60,})["']?"#).unwrap();
    
    // Password Patterns (Context-Aware)
    static ref PASSWORD_IN_JSON: Regex = Regex::new(r#"["']password["']\s*:\s*["']([^"']{8,})["']"#).unwrap();
    static ref PASSWORD_IN_YAML: Regex = Regex::new(r"(?m)^\s*password\s*:\s*(.+)$").unwrap();
    static ref PASSWORD_ENV_VAR: Regex = Regex::new(r#"(?i)(password|passwd|pwd)["']?\s*[:=]\s*["']?([^\s"']{8,})["']?"#).unwrap();
    static ref PASSWORD_IN_URL: Regex = Regex::new(r"://[^:]+:([^@]{8,})@").unwrap();
    static ref GENERIC_SECRET: Regex = Regex::new(r#"(?i)(api[_\s\-]?key|secret[_\s\-]?key|auth[_\s\-]?token|access[_\s\-]?token)["']?\s*[:=]\s*["']?([a-zA-Z0-9\-._~+/]{20,})["']?"#).unwrap();
    
    // Connection String Patterns
    static ref CONNECTION_STRING: Regex = Regex::new(r#"(?i)(connection[_\s\-]?string|conn[_\s\-]?str)["']?\s*[:=]\s*["']?([^"'\s]+)["']?"#).unwrap();
    static ref DATABASE_URL: Regex = Regex::new(r#"(?i)database[_\s\-]?url["']?\s*[:=]\s*["']?([^"'\s]+)["']?"#).unwrap();
    
    // Private Keys (Multi-line support)
    static ref MULTILINE_PRIVATE_KEY: Regex = Regex::new(r"(?s)-----BEGIN[^-]+PRIVATE[^-]+-----.*?-----END[^-]+PRIVATE[^-]+-----").unwrap();
    
    // API Tokens
    static ref DIGITALOCEAN_TOKEN: Regex = Regex::new(r"dop_v1_[a-f0-9]{64}").unwrap();
    static ref HEROKU_API_KEY: Regex = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap();
    static ref DISCORD_TOKEN: Regex = Regex::new(r"[MN][A-Za-z\d]{23}\.[\w-]{6}\.[\w-]{27}").unwrap();
    static ref SHOPIFY_TOKEN: Regex = Regex::new(r"shppa_[a-fA-F0-9]{32}").unwrap();
    static ref GITLAB_TOKEN: Regex = Regex::new(r"glpat-[0-9a-zA-Z\-_]{20}").unwrap();
    static ref ALL_PATTERNS: HashMap<String, &'static Regex> = {
        let mut patterns = HashMap::new();
        // AWS Patterns
        patterns.insert("AWS Access Key".to_string(), &*AWS_ACCESS_KEY);
        patterns.insert("AWS Secret Key".to_string(), &*AWS_SECRET_KEY);
        patterns.insert("AWS Access Key ID".to_string(), &*AWS_ACCESS_KEY_ID);
        
        // GitHub Patterns
        patterns.insert("GitHub Token".to_string(), &*GITHUB_TOKEN);
        patterns.insert("GitHub OAuth".to_string(), &*GITHUB_OAUTH);
        
        // Google Patterns
        patterns.insert("Google API Key".to_string(), &*GOOGLE_API_KEY);
        patterns.insert("Google OAuth".to_string(), &*GOOGLE_OAUTH);
        
        // JWT
        patterns.insert("JWT Token".to_string(), &*JWT_TOKEN);
        
        // Private Keys
        patterns.insert("RSA Private Key".to_string(), &*RSA_PRIVATE_KEY);
        patterns.insert("EC Private Key".to_string(), &*EC_PRIVATE_KEY);
        patterns.insert("PGP Private Key".to_string(), &*PGP_PRIVATE_KEY);
        patterns.insert("SSH Private Key".to_string(), &*SSH_PRIVATE_KEY);
        patterns.insert("Generic Private Key".to_string(), &*GENERIC_PRIVATE_KEY);
        patterns.insert("Multi-line Private Key".to_string(), &*MULTILINE_PRIVATE_KEY);
        
        // Database URLs
        patterns.insert("PostgreSQL URL".to_string(), &*POSTGRES_URL);
        patterns.insert("MySQL URL".to_string(), &*MYSQL_URL);
        patterns.insert("MongoDB URL".to_string(), &*MONGODB_URL);
        patterns.insert("Redis URL".to_string(), &*REDIS_URL);
        
        // API Keys
        patterns.insert("OpenAI API Key".to_string(), &*OPENAI_API_KEY);
        patterns.insert("Stripe API Key".to_string(), &*STRIPE_API_KEY);
        patterns.insert("SendGrid API Key".to_string(), &*SENDGRID_API_KEY);
        patterns.insert("Slack Token".to_string(), &*SLACK_TOKEN);
        patterns.insert("Twilio API Key".to_string(), &*TWILIO_API_KEY);
        patterns.insert("Mailgun API Key".to_string(), &*MAILGUN_API_KEY);
        patterns.insert("Firebase API Key".to_string(), &*FIREBASE_API_KEY);
        patterns.insert("DigitalOcean Token".to_string(), &*DIGITALOCEAN_TOKEN);
        patterns.insert("Heroku API Key".to_string(), &*HEROKU_API_KEY);
        patterns.insert("Discord Token".to_string(), &*DISCORD_TOKEN);
        patterns.insert("Shopify Token".to_string(), &*SHOPIFY_TOKEN);
        patterns.insert("GitLab Token".to_string(), &*GITLAB_TOKEN);
        
        // OAuth Patterns
        patterns.insert("Generic OAuth Secret".to_string(), &*GENERIC_OAUTH_SECRET);
        patterns.insert("Generic Client ID".to_string(), &*GENERIC_CLIENT_ID);
        
        // Azure Patterns
        patterns.insert("Azure Tenant ID".to_string(), &*AZURE_TENANT_ID);
        patterns.insert("Azure Client Secret".to_string(), &*AZURE_CLIENT_SECRET);
        
        // PayPal Patterns
        patterns.insert("PayPal Client ID".to_string(), &*PAYPAL_CLIENT_ID);
        patterns.insert("PayPal Secret".to_string(), &*PAYPAL_SECRET);
        
        // Password Patterns
        patterns.insert("Password in JSON".to_string(), &*PASSWORD_IN_JSON);
        patterns.insert("Password in YAML".to_string(), &*PASSWORD_IN_YAML);
        patterns.insert("Password Environment Variable".to_string(), &*PASSWORD_ENV_VAR);
        patterns.insert("Password in URL".to_string(), &*PASSWORD_IN_URL);
        patterns.insert("Generic Secret".to_string(), &*GENERIC_SECRET);
        
        // Connection Strings
        patterns.insert("Connection String".to_string(), &*CONNECTION_STRING);
        patterns.insert("Database URL".to_string(), &*DATABASE_URL);
        
        patterns
    };
}

pub fn get_all_patterns() -> &'static HashMap<String, &'static Regex> {
    &ALL_PATTERNS
}

pub fn get_all_patterns_owned() -> HashMap<String, Regex> {
    let mut patterns = HashMap::new();
    // AWS Patterns
    patterns.insert("AWS Access Key".to_string(), AWS_ACCESS_KEY.clone());
    patterns.insert("AWS Secret Key".to_string(), AWS_SECRET_KEY.clone());
    patterns.insert("AWS Access Key ID".to_string(), AWS_ACCESS_KEY_ID.clone());
    
    // GitHub Patterns
    patterns.insert("GitHub Token".to_string(), GITHUB_TOKEN.clone());
    patterns.insert("GitHub OAuth".to_string(), GITHUB_OAUTH.clone());
    
    // Google Patterns
    patterns.insert("Google API Key".to_string(), GOOGLE_API_KEY.clone());
    patterns.insert("Google OAuth".to_string(), GOOGLE_OAUTH.clone());
    
    // JWT
    patterns.insert("JWT Token".to_string(), JWT_TOKEN.clone());
    
    // Private Keys
    patterns.insert("RSA Private Key".to_string(), RSA_PRIVATE_KEY.clone());
    patterns.insert("EC Private Key".to_string(), EC_PRIVATE_KEY.clone());
    patterns.insert("PGP Private Key".to_string(), PGP_PRIVATE_KEY.clone());
    patterns.insert("SSH Private Key".to_string(), SSH_PRIVATE_KEY.clone());
    patterns.insert("Generic Private Key".to_string(), GENERIC_PRIVATE_KEY.clone());
    patterns.insert("Multi-line Private Key".to_string(), MULTILINE_PRIVATE_KEY.clone());
    
    // Database URLs
    patterns.insert("PostgreSQL URL".to_string(), POSTGRES_URL.clone());
    patterns.insert("MySQL URL".to_string(), MYSQL_URL.clone());
    patterns.insert("MongoDB URL".to_string(), MONGODB_URL.clone());
    patterns.insert("Redis URL".to_string(), REDIS_URL.clone());
    
    // API Keys
    patterns.insert("OpenAI API Key".to_string(), OPENAI_API_KEY.clone());
    patterns.insert("Stripe API Key".to_string(), STRIPE_API_KEY.clone());
    patterns.insert("SendGrid API Key".to_string(), SENDGRID_API_KEY.clone());
    patterns.insert("Slack Token".to_string(), SLACK_TOKEN.clone());
    patterns.insert("Twilio API Key".to_string(), TWILIO_API_KEY.clone());
    patterns.insert("Mailgun API Key".to_string(), MAILGUN_API_KEY.clone());
    patterns.insert("Firebase API Key".to_string(), FIREBASE_API_KEY.clone());
    patterns.insert("DigitalOcean Token".to_string(), DIGITALOCEAN_TOKEN.clone());
    patterns.insert("Heroku API Key".to_string(), HEROKU_API_KEY.clone());
    patterns.insert("Discord Token".to_string(), DISCORD_TOKEN.clone());
    patterns.insert("Shopify Token".to_string(), SHOPIFY_TOKEN.clone());
    patterns.insert("GitLab Token".to_string(), GITLAB_TOKEN.clone());
    
    // OAuth Patterns
    patterns.insert("Generic OAuth Secret".to_string(), GENERIC_OAUTH_SECRET.clone());
    patterns.insert("Generic Client ID".to_string(), GENERIC_CLIENT_ID.clone());
    
    // Azure Patterns
    patterns.insert("Azure Tenant ID".to_string(), AZURE_TENANT_ID.clone());
    patterns.insert("Azure Client Secret".to_string(), AZURE_CLIENT_SECRET.clone());
    
    // PayPal Patterns
    patterns.insert("PayPal Client ID".to_string(), PAYPAL_CLIENT_ID.clone());
    patterns.insert("PayPal Secret".to_string(), PAYPAL_SECRET.clone());
    
    // Password Patterns
    patterns.insert("Password in JSON".to_string(), PASSWORD_IN_JSON.clone());
    patterns.insert("Password in YAML".to_string(), PASSWORD_IN_YAML.clone());
    patterns.insert("Password Environment Variable".to_string(), PASSWORD_ENV_VAR.clone());
    patterns.insert("Password in URL".to_string(), PASSWORD_IN_URL.clone());
    patterns.insert("Generic Secret".to_string(), GENERIC_SECRET.clone());
    
    // Connection Strings
    patterns.insert("Connection String".to_string(), CONNECTION_STRING.clone());
    patterns.insert("Database URL".to_string(), DATABASE_URL.clone());
    
    patterns
}
