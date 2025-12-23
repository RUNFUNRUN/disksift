use crate::args::Args;
use crate::scanner::Item;
use colored::*;
use humansize::{DECIMAL, format_size};
use std::cmp::Reverse;

pub fn display_results(items: Vec<Item>, total_size: u64, errors: u64, args: &Args) {
    let root_path = &args.path;
    let min_bytes = args
        .min_size
        .as_deref()
        .and_then(|s| crate::args::parse_size(s));

    let mut filtered_items = items;

    // Filter by min_size
    if let Some(min) = min_bytes {
        filtered_items.retain(|i| i.size >= min);
    }

    // Filter by depth (Display Depth)
    if let Some(d) = args.depth {
        let root_depth = root_path.components().count();
        filtered_items.retain(|i| {
            let item_depth = i.path.components().count();
            // relative depth.
            // root is depth 0.
            if item_depth > root_depth {
                (item_depth - root_depth) <= d
            } else {
                true
            }
        });
    }

    // Sort by size (Desc)
    filtered_items.sort_by_key(|k| Reverse(k.size));

    // Refined Dedup Logic: "Prefer Children / Details"
    let candidate_limit = args.limit * 5;
    let candidates: Vec<Item> = filtered_items.into_iter().take(candidate_limit).collect();
    let mut indices_to_hide = std::collections::HashSet::new();

    for i in 0..candidates.len() {
        for j in 0..candidates.len() {
            if i == j {
                continue;
            }

            let parent = &candidates[i];
            let child = &candidates[j];

            if child.path.starts_with(&parent.path) && child.path != parent.path {
                indices_to_hide.insert(i);
            }
        }
    }

    let mut displayed_items = Vec::new();
    for (i, item) in candidates.into_iter().enumerate() {
        if !indices_to_hide.contains(&i) {
            displayed_items.push(item);
        }
    }

    // Display
    println!("\n{}", "Top Space Hogs:".green().bold().underline());
    println!("{:<15} {:<}", "Size".bold(), "Path".bold());

    for item in displayed_items.iter().take(args.limit) {
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
        format_size(total_size, DECIMAL).green().bold()
    );
}
