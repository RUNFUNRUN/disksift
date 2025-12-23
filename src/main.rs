use clap::Parser;
use colored::*;

mod args;
mod display;
mod scanner;

use args::Args;

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

    // Validate min_size before scanning to fail fast
    if let Some(ref s) = args.min_size {
        if args::parse_size(s).is_none() {
            eprintln!("{} Invalid size format: {}", "Error:".red().bold(), s);
            std::process::exit(1);
        }
    }

    println!(
        "{} {}",
        "Scanning:".blue().bold(),
        root_path.display().to_string().cyan()
    );

    let result = scanner::scan(root_path);

    display::display_results(result.items, result.total_size, result.errors, &args);
}
