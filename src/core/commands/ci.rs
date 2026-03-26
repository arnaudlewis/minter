use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::core::commands::coverage::scan_for_tags;
use crate::core::graph::cache::content_hash;
use crate::core::{discover, io, parser};

// ── Lock file types ─────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct LockFile {
    #[allow(dead_code)]
    version: u64,
    specs: BTreeMap<String, LockSpec>,
    #[serde(default)]
    nfrs: BTreeMap<String, LockNfr>,
    #[serde(default)]
    benchmark_files: BTreeMap<String, LockBenchmarkFile>,
}

#[derive(Debug, serde::Deserialize)]
struct LockBenchmarkFile {
    hash: String,
}

#[derive(Debug, serde::Deserialize)]
struct LockSpec {
    hash: String,
    behaviors: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    nfrs: Vec<String>,
    #[serde(default)]
    test_files: BTreeMap<String, LockTestFile>,
}

#[derive(Debug, serde::Deserialize)]
struct LockNfr {
    hash: String,
}

#[derive(Debug, serde::Deserialize)]
struct LockTestFile {
    hash: String,
    #[serde(default)]
    #[allow(dead_code)]
    covers: Vec<String>,
}

// ── Check result types ──────────────────────────────────

struct CheckResult {
    name: &'static str,
    passed: bool,
    errors: Vec<String>,
    stats: String,
}

// ── Entry point ─────────────────────────────────────────

/// Run the ci command. Returns exit code.
pub fn run_ci(config: &crate::core::config::ProjectConfig) -> i32 {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot determine working directory: {}", e);
            return 1;
        }
    };

    // Load lock file
    let lock_path = cwd.join("minter.lock");
    if !lock_path.exists() {
        eprintln!("error: minter.lock not found — run `minter lock` to generate it");
        return 1;
    }

    let lock_content = match std::fs::read_to_string(&lock_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read minter.lock: {}", e);
            return 1;
        }
    };

    let lock: LockFile = match serde_json::from_str(&lock_content) {
        Ok(l) => l,
        Err(_) => {
            eprintln!("error: minter.lock is invalid or corrupted");
            return 1;
        }
    };

    let specs_dir = &config.specs;
    let test_dirs = &config.tests;

    // Discover current state
    let disk_spec_files = discover::discover_spec_files(specs_dir).unwrap_or_default();
    let disk_nfr_files = discover::discover_nfr_files(specs_dir);

    // Build relative path -> absolute path maps for specs
    let mut disk_specs_rel: BTreeMap<String, PathBuf> = BTreeMap::new();
    for path in &disk_spec_files {
        let rel = io::make_relative(path, &cwd);
        disk_specs_rel.insert(rel, path.clone());
    }

    // Build relative path -> absolute path maps for nfrs
    let mut disk_nfrs_rel: BTreeMap<String, PathBuf> = BTreeMap::new();
    for path in &disk_nfr_files {
        let rel = io::make_relative(path, &cwd);
        disk_nfrs_rel.insert(rel, path.clone());
    }

    // Parse all disk specs for dependency/behavior/coverage checks
    let mut parsed_specs: Vec<(String, String, crate::model::Spec)> = Vec::new();
    for (rel_path, abs_path) in &disk_specs_rel {
        if let Ok(source) = io::read_file_safe(abs_path) {
            if let Ok(spec) = parser::parse(&source) {
                parsed_specs.push((rel_path.clone(), source, spec));
            }
        }
    }

    // Scan for test tags
    let existing_test_dirs: Vec<PathBuf> =
        test_dirs.iter().filter(|p| p.exists()).cloned().collect();
    let tags = if existing_test_dirs.is_empty() {
        Vec::new()
    } else {
        scan_for_tags(&existing_test_dirs)
    };

    // Build set of tagged test files (relative paths) and their hashes
    let mut disk_tagged_tests: BTreeMap<String, String> = BTreeMap::new();
    // Also collect behavior names per test file for coverage/orphan checks
    let mut test_tag_behaviors: HashMap<String, Vec<String>> = HashMap::new();

    for tag in &tags {
        if tag.tag_type.is_empty() || tag.ids.is_empty() {
            continue;
        }
        if !["unit", "integration", "e2e", "benchmark"].contains(&tag.tag_type.as_str()) {
            continue;
        }

        let rel_path = io::make_relative(&tag.file, &cwd);

        // Hash the file if not already done
        if !disk_tagged_tests.contains_key(&rel_path) {
            if let Ok(content) = std::fs::read_to_string(&tag.file) {
                disk_tagged_tests.insert(rel_path.clone(), content_hash(&content));
            }
        }

        // Collect behavior names (not NFR refs)
        for id in &tag.ids {
            if !id.starts_with('#') && tag.tag_type != "benchmark" {
                test_tag_behaviors
                    .entry(rel_path.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }

    // Run all six checks
    let checks = vec![
        check_spec_integrity(&lock, &disk_specs_rel, &cwd),
        check_nfr_integrity(&lock, &disk_nfrs_rel, &cwd),
        check_dependency_structure(&lock, &parsed_specs),
        check_test_integrity(&lock, &disk_tagged_tests),
        check_coverage(&lock, &test_tag_behaviors),
        check_orphans(&lock, &test_tag_behaviors, &parsed_specs),
    ];

    // Output results
    let any_failed = checks.iter().any(|c| !c.passed);

    // Print errors to stderr
    for check in &checks {
        for err in &check.errors {
            eprintln!("{}", err);
        }
    }

    // Print summary to stdout
    for check in &checks {
        let status = if check.passed { "pass" } else { "FAIL" };
        if check.passed && !check.stats.is_empty() {
            println!("  {} {} ({})", status, check.name, check.stats);
        } else {
            println!("  {} {}", status, check.name);
        }
    }

    if any_failed { 1 } else { 0 }
}

// ── Check 1: Spec integrity ────────────────────────────

fn check_spec_integrity(
    lock: &LockFile,
    disk_specs: &BTreeMap<String, PathBuf>,
    cwd: &Path,
) -> CheckResult {
    let mut errors = Vec::new();

    // Check each spec in lock exists and hash matches
    for (lock_path, lock_spec) in &lock.specs {
        let abs_path = cwd.join(lock_path);
        if !abs_path.exists() {
            errors.push(format!("spec integrity: {} missing from disk", lock_path));
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&abs_path) {
            let current_hash = content_hash(&content);
            if current_hash != lock_spec.hash {
                errors.push(format!("spec integrity: {} hash mismatch", lock_path));
            }
        }
    }

    // Check each spec on disk is in the lock
    for rel_path in disk_specs.keys() {
        if !lock.specs.contains_key(rel_path) {
            errors.push(format!("spec integrity: {} not in lock", rel_path));
        }
    }

    let spec_count = lock.specs.len();
    let nfr_count = lock.nfrs.len();
    let stats = if nfr_count > 0 {
        format!("{} specs, {} nfrs", spec_count, nfr_count)
    } else {
        format!("{} specs", spec_count)
    };

    CheckResult {
        name: "spec integrity",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}

// ── Check 2: NFR integrity ─────────────────────────────

fn check_nfr_integrity(
    lock: &LockFile,
    disk_nfrs: &BTreeMap<String, PathBuf>,
    cwd: &Path,
) -> CheckResult {
    let mut errors = Vec::new();

    // Check each NFR in lock exists and hash matches
    for (lock_path, lock_nfr) in &lock.nfrs {
        let abs_path = cwd.join(lock_path);
        if !abs_path.exists() {
            errors.push(format!("nfr integrity: {} missing from disk", lock_path));
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&abs_path) {
            let current_hash = content_hash(&content);
            if current_hash != lock_nfr.hash {
                errors.push(format!("nfr integrity: {} hash mismatch", lock_path));
            }
        }
    }

    // Check each NFR on disk is in the lock
    for rel_path in disk_nfrs.keys() {
        if !lock.nfrs.contains_key(rel_path) {
            errors.push(format!("nfr integrity: {} not in lock", rel_path));
        }
    }

    let nfr_count = lock.nfrs.len();
    let stats = format!("{} nfrs", nfr_count);

    CheckResult {
        name: "nfr integrity",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}

// ── Check 3: Dependency structure ──────────────────────

fn check_dependency_structure(
    lock: &LockFile,
    parsed_specs: &[(String, String, crate::model::Spec)],
) -> CheckResult {
    let mut errors = Vec::new();

    // Build a name -> rel_path map for resolving dependencies
    let mut name_to_rel: HashMap<String, String> = HashMap::new();
    for (rel_path, _, spec) in parsed_specs {
        name_to_rel.insert(spec.name.clone(), rel_path.clone());
    }

    // For each spec in lock, check if current dependencies match
    for (lock_path, lock_spec) in &lock.specs {
        // Find the parsed spec for this path
        if let Some((_, _, spec)) = parsed_specs.iter().find(|(rp, _, _)| rp == lock_path) {
            // Resolve current dependencies to paths
            let mut current_deps: Vec<String> = spec
                .dep_names()
                .iter()
                .filter_map(|dep_name| name_to_rel.get(dep_name).cloned())
                .collect();
            current_deps.sort();

            let mut locked_deps = lock_spec.dependencies.clone();
            locked_deps.sort();

            if current_deps != locked_deps {
                errors.push(format!(
                    "dependency structure: {} dependency mismatch",
                    lock_path
                ));
            }
        }
    }

    let edge_count: usize = lock.specs.values().map(|s| s.dependencies.len()).sum();
    let stats = format!("{} edges", edge_count);

    CheckResult {
        name: "dependency structure",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}

// ── Check 4: Test integrity ────────────────────────────

fn check_test_integrity(
    lock: &LockFile,
    disk_tagged_tests: &BTreeMap<String, String>,
) -> CheckResult {
    let mut errors = Vec::new();

    // Collect all test files from the lock (spec test_files + benchmark_files)
    let mut lock_test_files: BTreeMap<String, String> = BTreeMap::new();
    for lock_spec in lock.specs.values() {
        for (test_path, test_file) in &lock_spec.test_files {
            lock_test_files
                .entry(test_path.clone())
                .or_insert_with(|| test_file.hash.clone());
        }
    }
    for (bench_path, bench_file) in &lock.benchmark_files {
        lock_test_files
            .entry(bench_path.clone())
            .or_insert_with(|| bench_file.hash.clone());
    }

    // Check each test in lock exists and hash matches
    for (lock_path, lock_hash) in &lock_test_files {
        match disk_tagged_tests.get(lock_path) {
            None => {
                // File might exist but have no tags, or might be deleted
                // Check if file actually exists on disk
                let abs_path = std::env::current_dir()
                    .map(|cwd| cwd.join(lock_path))
                    .unwrap_or_else(|_| PathBuf::from(lock_path));
                if !abs_path.exists() {
                    errors.push(format!("test integrity: {} missing from disk", lock_path));
                } else {
                    // File exists but not tagged — check hash
                    if let Ok(content) = std::fs::read_to_string(&abs_path) {
                        let current_hash = content_hash(&content);
                        if current_hash != *lock_hash {
                            errors.push(format!("test integrity: {} hash mismatch", lock_path));
                        }
                    }
                }
            }
            Some(current_hash) => {
                if current_hash != lock_hash {
                    errors.push(format!("test integrity: {} hash mismatch", lock_path));
                }
            }
        }
    }

    // Check each tagged test on disk is in the lock
    for rel_path in disk_tagged_tests.keys() {
        if !lock_test_files.contains_key(rel_path) {
            errors.push(format!("test integrity: {} not in lock", rel_path));
        }
    }

    let test_file_count = lock_test_files.len();
    let stats = format!("{} test files", test_file_count);

    CheckResult {
        name: "test integrity",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}

// ── Check 5: Coverage ──────────────────────────────────

fn check_coverage(
    lock: &LockFile,
    test_tag_behaviors: &HashMap<String, Vec<String>>,
) -> CheckResult {
    let mut errors = Vec::new();

    // Collect all covered behavior names from tags
    let mut covered_behaviors: HashSet<String> = HashSet::new();
    for behaviors in test_tag_behaviors.values() {
        for b in behaviors {
            // Handle qualified names (spec_name/behavior_name)
            if let Some((_spec, behavior)) = b.split_once('/') {
                covered_behaviors.insert(behavior.to_string());
            } else {
                covered_behaviors.insert(b.clone());
            }
        }
    }

    // Check each behavior in each spec has coverage
    let mut total_behaviors: usize = 0;
    let mut covered_count: usize = 0;
    for lock_spec in lock.specs.values() {
        for behavior in &lock_spec.behaviors {
            total_behaviors += 1;
            if covered_behaviors.contains(behavior) {
                covered_count += 1;
            } else {
                errors.push(format!("coverage: {} uncovered", behavior));
            }
        }
    }

    let pct = if total_behaviors > 0 {
        (covered_count as f64 / total_behaviors as f64 * 100.0) as u64
    } else {
        100
    };
    let stats = format!("{}/{} behaviors, {}%", covered_count, total_behaviors, pct);

    CheckResult {
        name: "coverage",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}

// ── Check 6: Orphan detection ──────────────────────────

fn check_orphans(
    _lock: &LockFile,
    test_tag_behaviors: &HashMap<String, Vec<String>>,
    parsed_specs: &[(String, String, crate::model::Spec)],
) -> CheckResult {
    let mut errors = Vec::new();

    // Build the set of all known behavior names from parsed specs (current disk state)
    let mut known_behaviors: HashSet<String> = HashSet::new();
    for (_, _, spec) in parsed_specs {
        for behavior in &spec.behaviors {
            known_behaviors.insert(behavior.name.clone());
        }
    }

    // Check each tag references a known behavior
    let mut orphan_count: usize = 0;
    for (test_path, behaviors) in test_tag_behaviors {
        for b in behaviors {
            let behavior_name = if let Some((_spec, behavior)) = b.split_once('/') {
                behavior.to_string()
            } else {
                b.clone()
            };

            if !known_behaviors.contains(&behavior_name) {
                orphan_count += 1;
                errors.push(format!(
                    "orphan: {} in {} references unknown behavior",
                    behavior_name, test_path
                ));
            }
        }
    }

    let stats = format!("{} orphaned tags", orphan_count);

    CheckResult {
        name: "orphan",
        passed: errors.is_empty(),
        errors,
        stats,
    }
}
