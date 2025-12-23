use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Item {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub struct ScanResult {
    pub items: Vec<Item>,
    pub total_size: u64,
    pub errors: u64,
}

pub fn scan(root_path: &PathBuf) -> ScanResult {
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

    // NOTE: We scan EVERYTHING to get accurate directory sizes.
    // The `depth` argument is used for filtering the *output*, not the scan.
    let walker = WalkDir::new(root_path);

    for entry in walker.into_iter() {
        match entry {
            Ok(entry) => {
                let path = entry.path().to_path_buf();
                // println!("DEBUG: Visiting {:?}", path);
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
                        // We traverse up. We stop if we go beyond the root scan directory
                        loop {
                            *dir_sizes.entry(current.to_path_buf()).or_insert(0) += size;

                            if current == root_path {
                                break;
                            }
                            match current.parent() {
                                Some(p) => current = p,
                                None => break,
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

    ScanResult {
        items: all_items,
        total_size: total_scanned_size,
        errors,
    }
}
