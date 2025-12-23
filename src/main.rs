use clap::Parser;
use colored::*;
use humansize::{DECIMAL, format_size};
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Number of items to display
    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    /// Filter items smaller than this size (e.g., "100MB", "1GB", "1024")
    #[arg(long)]
    min_size: Option<String>,

    /// Maximum recursion depth
    #[arg(short, long)]
    depth: Option<usize>,
}

#[derive(Debug)]
struct Item {
    path: PathBuf,
    size: u64,
    is_dir: bool,
}

fn parse_size(s: &str) -> Option<u64> {
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

fn main() {
    let args = Args::parse();
    let root_path = &args.path;

    if !root_path.exists() {
        eprintln!(
            "{} {}",
            "Error:".red().bold(),
            format!("Path {:?} does not exist", root_path).white()
        );
        std::process::exit(1);
    }

    // Parse min_size if provided
    let min_bytes = if let Some(ref s) = args.min_size {
        match parse_size(s) {
            Some(b) => Some(b),
            None => {
                eprintln!("{} Invalid size format: {}", "Error:".red().bold(), s);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    println!(
        "{} {}",
        "Scanning:".blue().bold(),
        root_path.display().to_string().cyan()
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message("Measuring storage...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Map content: Path -> accumulated size
    let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();
    let mut file_items: Vec<Item> = Vec::new();

    let mut total_scanned_size = 0;
    let mut errors = 0;

    let walker = WalkDir::new(root_path);
    let walker = if let Some(depth) = args.depth {
        walker.max_depth(depth)
    } else {
        walker
    };

    for entry in walker.into_iter() {
        match entry {
            Ok(entry) => {
                let path = entry.path().to_path_buf();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => {
                        errors += 1;
                        continue;
                    }
                };

                if metadata.is_file() {
                    let size = metadata.len();
                    total_scanned_size += size;

                    // Add to file list
                    file_items.push(Item {
                        path: path.clone(),
                        size,
                        is_dir: false,
                    });

                    // Aggregate to all ancestors
                    if let Some(parent) = path.parent() {
                        let mut current = parent;
                        loop {
                            *dir_sizes.entry(current.to_path_buf()).or_insert(0) += size;

                            if current == root_path {
                                break;
                            }
                            match current.parent() {
                                Some(p) => current = p,
                                None => break, // Reached system root
                            }
                        }
                    }
                }

                if file_items.len() % 1000 == 0 {
                    spinner.set_message(format!("Found {} files...", file_items.len()));
                }
            }
            Err(_) => errors += 1,
        }
    }

    spinner.finish_with_message("Scan complete!");

    // Combine files and directories into one list
    let mut all_items: Vec<Item> = file_items;

    for (path, size) in dir_sizes {
        // Exclude the root path itself from the list as it's redundant
        if &path == root_path {
            continue;
        }
        all_items.push(Item {
            path,
            size,
            is_dir: true,
        });
    }

    // Filter by min_size
    if let Some(min) = min_bytes {
        all_items.retain(|i| i.size >= min);
    }

    // Sort by size
    all_items.sort_by_key(|k| Reverse(k.size));

    // Display
    println!("\n{}", "Top Space Hogs:".green().bold().underline());
    println!("{:<15} {:<}", "Size".bold(), "Path".bold());

    let limit = args.limit;
    let mut count = 0;

    for item in all_items.iter() {
        if count >= limit {
            break;
        }

        // Skip small items filter if implemented
        // (Leaving empty for now as per MVP plan refactor)

        let size_str = format_size(item.size, DECIMAL);
        let icon = if item.is_dir { "📁" } else { "📄" };
        let path_display = item.path.display().to_string();

        // Colorize
        let size_colored = if item.size > 1_000_000_000 {
            // >1GB
            size_str.red().bold()
        } else if item.size > 100_000_000 {
            // >100MB
            size_str.yellow()
        } else {
            size_str.green()
        };

        let path_colored = if item.is_dir {
            path_display.blue().bold()
        } else {
            path_display.white()
        };

        println!("{:<15} {} {}", size_colored, icon, path_colored);
        count += 1;
    }

    if errors > 0 {
        eprintln!(
            "\n{} {} errors encountered (permission denied, etc)",
            "Warning:".yellow(),
            errors
        );
    }

    println!(
        "\n{} {}",
        "Total size:".blue(),
        format_size(total_scanned_size, DECIMAL).green().bold()
    );
}
