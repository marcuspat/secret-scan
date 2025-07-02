pub mod patterns;
pub mod entropy;
pub mod scanner;
pub mod output;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub pattern_name: String,
    pub matched_text: String,
    pub entropy: Option<f64>,
}

pub use scanner::Scanner;
pub use patterns::*;
pub use entropy::*;
pub use output::*;