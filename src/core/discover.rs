use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Recursively discover all .spec files in a directory tree, optionally excluding one path.
/// Returns a map of spec name -> path.
pub fn discover_specs(dir: &Path, exclude: Option<&Path>) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    for result in walkdir::WalkDir::new(dir).follow_links(false).into_iter() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: {}", e);
                continue;
            }
        };
        let path = entry.path().to_path_buf();
        if let Some(excl) = exclude
            && path == excl
        {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("spec")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            map.insert(stem.to_string(), path);
        }
    }
    map
}

/// Recursively discover all .spec files in a directory tree.
/// Returns sorted paths, or an error if duplicate spec names are found.
pub fn discover_spec_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = Vec::new();
    let mut names_seen: HashMap<String, PathBuf> = HashMap::new();

    for result in walkdir::WalkDir::new(dir).follow_links(false).into_iter() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: {}", e);
                continue;
            }
        };
        let path = entry.path().to_path_buf();
        if path.extension().and_then(|e| e.to_str()) == Some("spec") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(prev_path) = names_seen.get(stem) {
                    return Err(format!(
                        "duplicate spec name '{}' found in:\n  {}\n  {}",
                        stem,
                        prev_path.display(),
                        path.display()
                    ));
                }
                names_seen.insert(stem.to_string(), path.clone());
            }
            spec_files.push(path);
        }
    }

    spec_files.sort();
    Ok(spec_files)
}

/// Recursively discover all .nfr files in a directory tree.
/// Returns sorted paths.
pub fn discover_nfr_files(dir: &Path) -> Vec<PathBuf> {
    let mut nfr_files = Vec::new();

    for result in walkdir::WalkDir::new(dir).follow_links(false).into_iter() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: {}", e);
                continue;
            }
        };
        let path = entry.path().to_path_buf();
        if path.extension().and_then(|e| e.to_str()) == Some("nfr") {
            nfr_files.push(path);
        }
    }

    nfr_files.sort();
    nfr_files
}

/// Recursively discover all .spec and .nfr files in a directory tree.
/// Returns sorted paths, or an error if duplicate spec names are found.
pub fn discover_all_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = discover_spec_files(dir)?;
    let nfr_files = discover_nfr_files(dir);
    spec_files.extend(nfr_files);
    spec_files.sort();
    Ok(spec_files)
}
