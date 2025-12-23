use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Number of items to display
    #[arg(short = 'n', long, default_value_t = 10)]
    pub limit: usize,

    /// Filter items smaller than this size (e.g., "100MB", "1GB", "1024")
    #[arg(long)]
    pub min_size: Option<String>,

    /// Maximum recursion depth
    #[arg(short, long)]
    pub depth: Option<usize>,
}

pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, unit) = s.split_at(s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len()));
    let num = num_str.parse::<f64>().ok()?;

    let multiplier = match unit.trim() {
        "KB" | "K" => 1_000.0,
        "MB" | "M" => 1_000_000.0,
        "GB" | "G" => 1_000_000_000.0,
        "TB" | "T" => 1_000_000_000_000.0,
        "" => 1.0,
        _ => return None,
    };

    Some((num * multiplier) as u64)
}
