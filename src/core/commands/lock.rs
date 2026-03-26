use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use crate::core::commands::coverage::{
    build_behavior_index, build_nfr_index, scan_for_tags, validate_tags,
};
use crate::core::graph::cache::content_hash;
use crate::core::{discover, io, parser};
use crate::model::NfrSpec;

/// Run the lock command. Returns exit code.
pub fn run_lock(config: &crate::core::config::ProjectConfig) -> i32 {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot determine working directory: {}", e);
            return 1;
        }
    };

    let specs_dir = &config.specs;
    let test_dirs = &config.tests;

    // Discover spec files
    let spec_files = match discover::discover_spec_files(specs_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };

    if spec_files.is_empty() {
        eprintln!("error: no spec files found in {}", specs_dir.display());
        return 1;
    }

    // Parse all specs, collecting errors
    let mut parsed_specs: Vec<(PathBuf, String, crate::model::Spec)> = Vec::new();
    let mut has_errors = false;

    for path in &spec_files {
        let source = match io::read_file_safe(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {}", path.display(), e);
                has_errors = true;
                continue;
            }
        };

        match parser::parse(&source) {
            Ok(spec) => {
                parsed_specs.push((path.clone(), source, spec));
            }
            Err(errors) => {
                for e in &errors {
                    eprintln!("{}: {}", path.display(), e);
                }
                has_errors = true;
            }
        }
    }

    if has_errors {
        return 1;
    }

    // Discover and parse NFR files
    let nfr_files = discover::discover_nfr_files(specs_dir);
    let mut nfr_specs: HashMap<String, NfrSpec> = HashMap::new();
    let mut nfr_sources: HashMap<String, (PathBuf, String)> = HashMap::new();

    for path in &nfr_files {
        let source = match io::read_file_safe(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        match parser::parse_nfr(&source) {
            Ok(nfr) => {
                let cat = nfr.category.clone();
                nfr_specs.insert(cat.clone(), nfr);
                nfr_sources.insert(cat, (path.clone(), source));
            }
            Err(_) => continue,
        }
    }

    // Build behavior index for tag validation
    let specs_for_index: Vec<(String, crate::model::Spec)> = parsed_specs
        .iter()
        .map(|(_, _, spec)| (spec.name.clone(), spec.clone()))
        .collect();

    let behavior_index = build_behavior_index(&specs_for_index);
    let nfr_index = build_nfr_index(&nfr_specs);

    // Scan test directories for @minter tags
    let existing_test_dirs: Vec<PathBuf> =
        test_dirs.iter().filter(|p| p.exists()).cloned().collect();

    let tags = if existing_test_dirs.is_empty() {
        Vec::new()
    } else {
        scan_for_tags(&existing_test_dirs)
    };

    // Validate tags
    let (_valid_tags, tag_errors, _warnings) =
        validate_tags(&tags, &behavior_index, &nfr_index, &specs_for_index);

    if !tag_errors.is_empty() {
        for e in &tag_errors {
            eprintln!("error: {}", e);
        }
        return 1;
    }

    // Build the lock structure

    // Map: spec_name -> (path, behaviors, dependencies, nfr_refs, content_hash)
    let mut lock_specs: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    // Build a mapping from spec_name to its file path (relative)
    let mut spec_name_to_rel_path: HashMap<String, String> = HashMap::new();
    // And from dep_name to relative path
    let mut dep_name_to_path: HashMap<String, String> = HashMap::new();

    for (path, source, spec) in &parsed_specs {
        let rel_path = io::make_relative(path, &cwd);
        spec_name_to_rel_path.insert(spec.name.clone(), rel_path.clone());
        dep_name_to_path.insert(spec.name.clone(), rel_path.clone());

        let hash = content_hash(source);

        let behaviors: Vec<String> = spec.behaviors.iter().map(|b| b.name.clone()).collect();

        // Dependencies: resolve to relative paths
        let dependencies: Vec<String> = spec
            .dep_names()
            .iter()
            .filter_map(|dep_name| {
                // Find the spec file for this dependency
                parsed_specs
                    .iter()
                    .find(|(_, _, s)| s.name == *dep_name)
                    .map(|(p, _, _)| io::make_relative(p, &cwd))
            })
            .collect();

        // NFR refs: collect all "category#anchor" strings
        let mut nfr_refs: BTreeSet<String> = BTreeSet::new();
        for nfr_ref in &spec.nfr_refs {
            if let Some(anchor) = &nfr_ref.anchor {
                nfr_refs.insert(format!("{}#{}", nfr_ref.category, anchor));
            }
        }
        for behavior in &spec.behaviors {
            for nfr_ref in &behavior.nfr_refs {
                nfr_refs.insert(format!("{}#{}", nfr_ref.category, nfr_ref.anchor));
            }
        }

        lock_specs.insert(
            rel_path,
            serde_json::json!({
                "hash": hash,
                "behaviors": behaviors,
                "dependencies": dependencies,
                "nfrs": nfr_refs.into_iter().collect::<Vec<_>>(),
                "test_files": {}
            }),
        );
    }

    // Build the mapping from raw tags + behavior index
    // For each raw tag that passes validation, map it to its spec and behaviors.
    // Re-walk the tags to build the coverage map.
    let mut test_file_behaviors: HashMap<(String, String), BTreeSet<String>> = HashMap::new();
    // key: (spec_rel_path, test_file_rel_path) -> set of behavior names

    // Also collect test file content for hashing
    let mut test_file_hashes: HashMap<String, String> = HashMap::new();

    // Process each tag: for valid behavioral tags, track the mapping
    // We need to re-validate in a way that preserves file info.
    // Since validate_tags doesn't return file info, let me do a simpler approach:
    // walk the raw tags and use the behavior_index directly.

    for tag in &tags {
        if tag.tag_type.is_empty() || tag.ids.is_empty() {
            continue;
        }
        if !["unit", "integration", "e2e", "benchmark"].contains(&tag.tag_type.as_str()) {
            continue;
        }
        if tag.tag_type == "benchmark" {
            continue; // benchmarks are NFR-only, not behavior coverage
        }

        let test_rel_path = io::make_relative(&tag.file, &cwd);

        // Hash the test file if not already done
        if !test_file_hashes.contains_key(&test_rel_path) {
            if let Ok(content) = io::read_file_safe(&tag.file) {
                test_file_hashes.insert(test_rel_path.clone(), content_hash(&content));
            }
        }

        for id in &tag.ids {
            if id.starts_with('#') {
                continue; // NFR reference
            }

            // Resolve the behavior to its spec(s)
            if let Some((spec_name, behavior_name)) = id.split_once('/') {
                // Qualified name
                if let Some(spec_rel_path) = spec_name_to_rel_path.get(spec_name) {
                    test_file_behaviors
                        .entry((spec_rel_path.clone(), test_rel_path.clone()))
                        .or_default()
                        .insert(behavior_name.to_string());
                }
            } else {
                // Unqualified name
                if let Some(spec_names) = behavior_index.get(id.as_str()) {
                    if spec_names.len() == 1 {
                        if let Some(spec_rel_path) = spec_name_to_rel_path.get(&spec_names[0]) {
                            test_file_behaviors
                                .entry((spec_rel_path.clone(), test_rel_path.clone()))
                                .or_default()
                                .insert(id.clone());
                        }
                    }
                    // If ambiguous (len > 1), the tag validation would have caught it
                }
            }
        }
    }

    // Populate test_files into lock_specs
    for ((spec_rel_path, test_rel_path), behaviors) in &test_file_behaviors {
        if let Some(spec_entry) = lock_specs.get_mut(spec_rel_path) {
            let test_files = spec_entry
                .get_mut("test_files")
                .expect("test_files must exist");
            let test_files_obj = test_files
                .as_object_mut()
                .expect("test_files must be object");

            let hash = test_file_hashes
                .get(test_rel_path)
                .cloned()
                .unwrap_or_default();

            let covers: Vec<String> = behaviors.iter().cloned().collect();

            test_files_obj.insert(
                test_rel_path.clone(),
                serde_json::json!({
                    "hash": hash,
                    "covers": covers,
                }),
            );
        }
    }

    // Build NFR section
    let mut lock_nfrs: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for (cat, (path, source)) in &nfr_sources {
        let rel_path = io::make_relative(path, &cwd);
        let hash = content_hash(source);
        lock_nfrs.insert(
            rel_path,
            serde_json::json!({
                "hash": hash,
            }),
        );
        let _ = cat; // category used only to organize nfr_sources
    }

    // Build the final lock object
    let lock = serde_json::json!({
        "version": 1,
        "specs": lock_specs,
        "nfrs": lock_nfrs,
    });

    // Write atomically
    let lock_path = cwd.join("minter.lock");
    let json = match serde_json::to_string_pretty(&lock) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("error: failed to serialize lock file: {}", e);
            return 1;
        }
    };

    let tmp_path = lock_path.with_extension("lock.tmp");
    if let Err(e) = std::fs::write(&tmp_path, &json) {
        eprintln!("error: failed to write lock file: {}", e);
        return 1;
    }
    if let Err(e) = std::fs::rename(&tmp_path, &lock_path) {
        eprintln!("error: failed to finalize lock file: {}", e);
        return 1;
    }

    println!("lock: minter.lock written");
    0
}
