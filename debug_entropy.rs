use std::collections::HashMap;

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    
    let mut frequency = HashMap::new();
    let len = s.len() as f64;
    
    for c in s.chars() {
        *frequency.entry(c).or_insert(0) += 1;
    }
    
    let mut entropy = 0.0;
    for &count in frequency.values() {
        let probability = count as f64 / len;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }
    
    entropy
}

fn main() {
    let test_strings = vec\![
        "Zm9vYmFyMTIz",
        "dGVzdDEyMzQ1Ng==",
        "aHR0cDovL2V4YW1wbGUuY29t",
        "c3VwZXJzZWNyZXRwYXNzd29yZA==",
    ];
    
    for s in test_strings {
        println\!("{}: {:.2}", s, shannon_entropy(s));
    }
}
