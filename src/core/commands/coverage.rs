use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::core::{discover, io, parser};
use crate::model::{NfrSpec, Spec};

// ── Tag scanning ────────────────────────────────────────

#[derive(Debug, Clone)]
struct MinterTag {
    file: PathBuf,
    line: usize,
    tag_type: String,
    ids: Vec<String>,
}

#[derive(Debug)]
struct TagError {
    file: PathBuf,
    line: usize,
    message: String,
}

impl std::fmt::Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file.display(), self.line, self.message)
    }
}

// ── Coverage report types ───────────────────────────────

#[derive(Debug, Clone)]
struct BehaviorCoverage {
    behavior_name: String,
    test_types: BTreeSet<String>,
    duplicates: Vec<(String, usize)>, // (type, count) where count > 1
}

#[derive(Debug, Clone)]
struct NfrCoverage {
    category: String,
    constraint: String,
    coverage_type: NfrCoverageType,
}

#[derive(Debug, Clone)]
enum NfrCoverageType {
    Derived,
    Benchmark,
    Uncovered,
}

#[derive(Debug)]
struct CoverageReport {
    specs: BTreeMap<String, SpecCoverage>,
    nfr_coverage: Vec<NfrCoverage>,
    total_behaviors: usize,
    covered_behaviors: usize,
    type_counts: BTreeMap<String, usize>,
}

#[derive(Debug)]
struct SpecCoverage {
    version: String,
    behaviors: Vec<BehaviorCoverage>,
}

// ── Entry point ─────────────────────────────────────────

/// Run coverage analysis and return JSON string. Used by MCP.
pub fn run_coverage_json(spec_path: &Path, scan_paths: &[PathBuf]) -> Result<String, String> {
    if !spec_path.exists() {
        return Err(format!("spec path not found: {}", spec_path.display()));
    }

    for scan in scan_paths {
        if !scan.exists() {
            return Err(format!("scan path not found: {}", scan.display()));
        }
    }

    let (specs, nfr_specs) = load_specs(spec_path)?;

    if specs.is_empty() {
        return Err(format!("no spec files found in {}", spec_path.display()));
    }

    let scan_dirs: Vec<PathBuf> = if scan_paths.is_empty() {
        if spec_path.is_dir() {
            vec![spec_path.to_path_buf()]
        } else {
            vec![
                spec_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from(".")),
            ]
        }
    } else {
        scan_paths.to_vec()
    };

    let tags = scan_for_tags(&scan_dirs);
    let behavior_index = build_behavior_index(&specs);
    let nfr_index = build_nfr_index(&nfr_specs);
    let (valid_tags, tag_errors, _warnings) =
        validate_tags(&tags, &behavior_index, &nfr_index, &specs);

    if !tag_errors.is_empty() {
        let errors: Vec<String> = tag_errors.iter().map(|e| e.to_string()).collect();
        return Err(errors.join("\n"));
    }

    let report = build_report(&specs, &nfr_specs, &valid_tags);
    Ok(format_json_report(&report))
}

/// Run the coverage command. Returns exit code (0 = full coverage, 1 = failure).
pub fn run_coverage(
    spec_path: &Path,
    scan_paths: &[PathBuf],
    format: Option<&str>,
    verbose: bool,
) -> i32 {
    // Validate format
    if let Some(fmt) = format {
        if fmt != "human" && fmt != "json" {
            eprintln!(
                "error: invalid format '{}'. Valid formats: human, json",
                fmt
            );
            return 1;
        }
    }

    let is_json = format == Some("json");

    // Validate spec path exists
    if !spec_path.exists() {
        if is_json {
            let err = serde_json::json!({
                "errors": [format!("spec path not found: {}", spec_path.display())]
            });
            println!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            eprintln!("error: spec path not found: {}", spec_path.display());
        }
        return 1;
    }

    // Validate scan paths exist
    for scan in scan_paths {
        if !scan.exists() {
            if is_json {
                let err = serde_json::json!({
                    "errors": [format!("scan path not found: {}", scan.display())]
                });
                println!("{}", serde_json::to_string_pretty(&err).unwrap());
            } else {
                eprintln!("error: scan path not found: {}", scan.display());
            }
            return 1;
        }
    }

    // Parse specs
    let (specs, nfr_specs) = match load_specs(spec_path) {
        Ok(result) => result,
        Err(msg) => {
            if is_json {
                let err = serde_json::json!({ "errors": [msg] });
                println!("{}", serde_json::to_string_pretty(&err).unwrap());
            } else {
                eprintln!("error: {}", msg);
            }
            return 1;
        }
    };

    if specs.is_empty() {
        if is_json {
            let err = serde_json::json!({ "errors": ["no spec files found"] });
            println!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            eprintln!("error: no spec files found in {}", spec_path.display());
        }
        return 1;
    }

    // Determine scan directories
    let scan_dirs: Vec<PathBuf> = if scan_paths.is_empty() {
        vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    } else {
        scan_paths.to_vec()
    };

    // Scan for tags
    let tags = scan_for_tags(&scan_dirs);

    // Build behavior index: name -> vec of (spec_name, behavior_name)
    let behavior_index = build_behavior_index(&specs);

    // Build NFR constraint index: "category#constraint" -> true
    let nfr_index = build_nfr_index(&nfr_specs);

    // Validate tags
    let (valid_tags, tag_errors, warnings) =
        validate_tags(&tags, &behavior_index, &nfr_index, &specs);

    // Print warnings to stderr
    for w in &warnings {
        eprintln!("warning: {}", w);
    }

    // If there are errors, report and exit
    if !tag_errors.is_empty() {
        if is_json {
            let errors: Vec<String> = tag_errors.iter().map(|e| e.to_string()).collect();
            let err = serde_json::json!({ "errors": errors });
            println!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            for e in &tag_errors {
                eprintln!("error: {}", e);
            }
        }
        return 1;
    }

    // Build coverage report
    let report = build_report(&specs, &nfr_specs, &valid_tags);

    // Output
    if is_json {
        print_json_report(&report);
    } else {
        print_human_report(&report, verbose);
    }

    if report.covered_behaviors == report.total_behaviors {
        0
    } else {
        1
    }
}

// ── Spec loading ────────────────────────────────────────

type SpecSet = (Vec<(String, Spec)>, HashMap<String, NfrSpec>);

fn load_specs(spec_path: &Path) -> Result<SpecSet, String> {
    let mut specs = Vec::new();
    let mut nfr_specs = HashMap::new();

    if spec_path.is_file() {
        let ext = spec_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "spec" {
            let source = std::fs::read_to_string(spec_path)
                .map_err(|e| format!("cannot read {}: {}", spec_path.display(), e))?;
            let spec = parser::parse(&source)
                .map_err(|errs| format!("parse error in {}: {}", spec_path.display(), errs[0]))?;
            let name = spec.name.clone();
            specs.push((name, spec));
        } else {
            return Err(format!("not a .spec file: {}", spec_path.display()));
        }

        // Also load NFR files from the same directory
        if let Some(parent) = spec_path.parent() {
            load_nfrs_from_dir(parent, &mut nfr_specs);
        }
    } else {
        // Directory: discover all specs
        let spec_files = discover::discover_spec_files(spec_path).map_err(|e| e.to_string())?;

        for file in &spec_files {
            let source = match io::read_file_safe(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("warning: cannot read {}: {}", file.display(), e);
                    continue;
                }
            };
            match parser::parse(&source) {
                Ok(spec) => {
                    let name = spec.name.clone();
                    specs.push((name, spec));
                }
                Err(errs) => {
                    eprintln!("warning: parse error in {}: {}", file.display(), errs[0]);
                }
            }
        }

        // Also load NFR files
        load_nfrs_from_dir(spec_path, &mut nfr_specs);
    }

    Ok((specs, nfr_specs))
}

fn load_nfrs_from_dir(dir: &Path, nfr_specs: &mut HashMap<String, NfrSpec>) {
    let nfr_files = discover::discover_nfr_files(dir);
    for file in &nfr_files {
        let source = match io::read_file_safe(file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        match parser::parse_nfr(&source) {
            Ok(nfr) => {
                nfr_specs.insert(nfr.category.clone(), nfr);
            }
            Err(_) => continue,
        }
    }
}

// ── Tag scanning ────────────────────────────────────────

fn scan_for_tags(scan_dirs: &[PathBuf]) -> Vec<MinterTag> {
    let mut tags = Vec::new();

    for dir in scan_dirs {
        let walker = ignore::WalkBuilder::new(dir)
            .hidden(false)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(false)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }

            let path = entry.path();

            // Skip binary-looking files and spec/nfr files
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "spec" || ext == "nfr" {
                continue;
            }

            // Try to read as text
            let content = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for (line_idx, line) in content.lines().enumerate() {
                if let Some(tag) = parse_tag_line(line, path, line_idx + 1) {
                    tags.push(tag);
                }
            }
        }
    }

    tags
}

fn parse_tag_line(line: &str, file: &Path, line_num: usize) -> Option<MinterTag> {
    let trimmed = line.trim();

    // Match // @minter... or # @minter...
    let after_comment = if let Some(rest) = trimmed.strip_prefix("//") {
        rest.trim_start()
    } else if let Some(rest) = trimmed.strip_prefix('#') {
        rest.trim_start()
    } else {
        return None;
    };

    if !after_comment.starts_with("@minter") {
        return None;
    }

    let minter_part = after_comment.strip_prefix("@minter")?;

    // Must have :type or be bare @minter (error case)
    if minter_part.is_empty() || minter_part.starts_with(char::is_whitespace) {
        // Bare @minter without :type
        return Some(MinterTag {
            file: file.to_path_buf(),
            line: line_num,
            tag_type: String::new(), // empty = missing type
            ids: minter_part.split_whitespace().map(String::from).collect(),
        });
    }

    if !minter_part.starts_with(':') {
        return None;
    }

    let after_colon = &minter_part[1..];
    let mut parts = after_colon.split_whitespace();
    let tag_type = parts.next()?.to_string();
    let ids: Vec<String> = parts.map(String::from).collect();

    Some(MinterTag {
        file: file.to_path_buf(),
        line: line_num,
        tag_type,
        ids,
    })
}

// ── Index building ──────────────────────────────────────

/// Maps behavior name -> vec of spec names that contain it
fn build_behavior_index(specs: &[(String, Spec)]) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for (spec_name, spec) in specs {
        for behavior in &spec.behaviors {
            index
                .entry(behavior.name.clone())
                .or_default()
                .push(spec_name.clone());
        }
    }
    index
}

/// Set of valid "category#constraint" strings
fn build_nfr_index(nfr_specs: &HashMap<String, NfrSpec>) -> HashSet<String> {
    let mut index = HashSet::new();
    for (category, nfr) in nfr_specs {
        for constraint in &nfr.constraints {
            index.insert(format!("{}#{}", category, constraint.name));
        }
    }
    index
}

// ── Tag validation ──────────────────────────────────────

struct ValidTag {
    tag_type: String,
    /// For behavioral: (spec_name, behavior_name)
    /// For benchmark: not used (nfr_refs used instead)
    behavior_ref: Option<(String, String)>,
    /// For benchmark tags: "category#constraint"
    nfr_ref: Option<String>,
}

fn validate_tags(
    tags: &[MinterTag],
    behavior_index: &HashMap<String, Vec<String>>,
    nfr_index: &HashSet<String>,
    specs: &[(String, Spec)],
) -> (Vec<ValidTag>, Vec<TagError>, Vec<String>) {
    let mut valid = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Build a set of spec names for qualified name lookup
    let spec_names: HashSet<String> = specs.iter().map(|(n, _)| n.clone()).collect();

    for tag in tags {
        // Check missing type
        if tag.tag_type.is_empty() {
            errors.push(TagError {
                file: tag.file.clone(),
                line: tag.line,
                message: "@minter tag missing type (expected @minter:<type>, e.g. @minter:unit, @minter:e2e, @minter:benchmark)".to_string(),
            });
            continue;
        }

        // Check empty tag
        if tag.ids.is_empty() {
            warnings.push(format!(
                "{}:{}: empty @minter:{} tag (no IDs specified)",
                tag.file.display(),
                tag.line,
                tag.tag_type,
            ));
            continue;
        }

        let is_benchmark = tag.tag_type == "benchmark";

        for id in &tag.ids {
            let is_nfr_ref = id.starts_with('#');

            if is_benchmark {
                // Benchmark tags must have NFR refs
                if !is_nfr_ref {
                    errors.push(TagError {
                        file: tag.file.clone(),
                        line: tag.line,
                        message: format!(
                            "benchmark tag contains behavior ID '{}' (expected #category#constraint)",
                            id
                        ),
                    });
                    continue;
                }

                // Parse #category#constraint
                let nfr_key = &id[1..]; // strip leading #
                if !nfr_index.contains(nfr_key) {
                    errors.push(TagError {
                        file: tag.file.clone(),
                        line: tag.line,
                        message: format!("unknown NFR constraint '{}' in benchmark tag", nfr_key),
                    });
                    continue;
                }

                valid.push(ValidTag {
                    tag_type: tag.tag_type.clone(),
                    behavior_ref: None,
                    nfr_ref: Some(nfr_key.to_string()),
                });
            } else {
                // Non-benchmark tags must have behavior refs
                if is_nfr_ref {
                    errors.push(TagError {
                        file: tag.file.clone(),
                        line: tag.line,
                        message: format!(
                            "NFR reference '{}' found in {} tag (NFR refs are only valid in benchmark tags)",
                            id, tag.tag_type
                        ),
                    });
                    continue;
                }

                // Check if it's a qualified name: spec-name/behavior-name
                if let Some((spec_name, behavior_name)) = id.split_once('/') {
                    if !spec_names.contains(spec_name) {
                        errors.push(TagError {
                            file: tag.file.clone(),
                            line: tag.line,
                            message: format!(
                                "unknown spec '{}' in qualified name '{}'",
                                spec_name, id
                            ),
                        });
                        continue;
                    }

                    // Check if behavior exists in that spec
                    let found = specs.iter().any(|(sn, s)| {
                        sn == spec_name && s.behaviors.iter().any(|b| b.name == behavior_name)
                    });
                    if !found {
                        errors.push(TagError {
                            file: tag.file.clone(),
                            line: tag.line,
                            message: format!(
                                "unknown behavior '{}' in spec '{}'",
                                behavior_name, spec_name
                            ),
                        });
                        continue;
                    }

                    valid.push(ValidTag {
                        tag_type: tag.tag_type.clone(),
                        behavior_ref: Some((spec_name.to_string(), behavior_name.to_string())),
                        nfr_ref: None,
                    });
                } else {
                    // Unqualified name
                    match behavior_index.get(id.as_str()) {
                        None => {
                            errors.push(TagError {
                                file: tag.file.clone(),
                                line: tag.line,
                                message: format!("unknown behavior '{}'", id),
                            });
                        }
                        Some(spec_list) if spec_list.len() > 1 => {
                            errors.push(TagError {
                                file: tag.file.clone(),
                                line: tag.line,
                                message: format!(
                                    "ambiguous behavior '{}' found in specs: {}. Use qualified name (e.g., {}/{})",
                                    id,
                                    spec_list.join(", "),
                                    spec_list[0],
                                    id
                                ),
                            });
                        }
                        Some(spec_list) => {
                            valid.push(ValidTag {
                                tag_type: tag.tag_type.clone(),
                                behavior_ref: Some((spec_list[0].clone(), id.clone())),
                                nfr_ref: None,
                            });
                        }
                    }
                }
            }
        }
    }

    (valid, errors, warnings)
}

// ── Report building ─────────────────────────────────────

fn build_report(
    specs: &[(String, Spec)],
    _nfr_specs: &HashMap<String, NfrSpec>,
    valid_tags: &[ValidTag],
) -> CoverageReport {
    let mut spec_coverages: BTreeMap<String, SpecCoverage> = BTreeMap::new();

    // Initialize all behaviors as uncovered
    for (spec_name, spec) in specs {
        let behaviors: Vec<BehaviorCoverage> = spec
            .behaviors
            .iter()
            .map(|b| BehaviorCoverage {
                behavior_name: b.name.clone(),
                test_types: BTreeSet::new(),
                duplicates: Vec::new(),
            })
            .collect();

        spec_coverages.insert(
            spec_name.clone(),
            SpecCoverage {
                version: spec.version.clone(),
                behaviors,
            },
        );
    }

    // Track per-behavior per-type counts for duplicate detection
    let mut type_counts_per_behavior: HashMap<(String, String, String), usize> = HashMap::new();

    // Apply tags
    for tag in valid_tags {
        if let Some((ref spec_name, ref behavior_name)) = tag.behavior_ref {
            if let Some(spec_cov) = spec_coverages.get_mut(spec_name) {
                if let Some(beh_cov) = spec_cov
                    .behaviors
                    .iter_mut()
                    .find(|b| b.behavior_name == *behavior_name)
                {
                    beh_cov.test_types.insert(tag.tag_type.clone());
                    let key = (
                        spec_name.clone(),
                        behavior_name.clone(),
                        tag.tag_type.clone(),
                    );
                    *type_counts_per_behavior.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    // Set duplicates
    for ((spec_name, behavior_name, tag_type), count) in &type_counts_per_behavior {
        if *count > 1 {
            if let Some(spec_cov) = spec_coverages.get_mut(spec_name) {
                if let Some(beh_cov) = spec_cov
                    .behaviors
                    .iter_mut()
                    .find(|b| b.behavior_name == *behavior_name)
                {
                    beh_cov.duplicates.push((tag_type.clone(), *count));
                }
            }
        }
    }

    // Build NFR coverage
    let mut nfr_coverage = Vec::new();

    // Collect all NFR refs from specs and their behaviors
    let mut nfr_behavior_map: HashMap<String, Vec<String>> = HashMap::new();
    for (spec_name, spec) in specs {
        // Spec-level NFR refs
        for nfr_ref in &spec.nfr_refs {
            if let Some(anchor) = &nfr_ref.anchor {
                let key = format!("{}#{}", nfr_ref.category, anchor);
                for b in &spec.behaviors {
                    nfr_behavior_map
                        .entry(key.clone())
                        .or_default()
                        .push(format!("{}/{}", spec_name, b.name));
                }
            }
        }
        // Behavior-level NFR refs
        for behavior in &spec.behaviors {
            for nfr_ref in &behavior.nfr_refs {
                let key = format!("{}#{}", nfr_ref.category, nfr_ref.anchor);
                nfr_behavior_map
                    .entry(key)
                    .or_default()
                    .push(format!("{}/{}", spec_name, behavior.name));
            }
        }
    }

    // Benchmark-covered NFRs
    let mut benchmark_nfrs: HashSet<String> = HashSet::new();
    for tag in valid_tags {
        if tag.tag_type == "benchmark" {
            if let Some(ref nfr_ref) = tag.nfr_ref {
                benchmark_nfrs.insert(nfr_ref.clone());
            }
        }
    }

    // Track all NFR constraints we need to report on
    let mut all_nfr_keys: BTreeSet<String> = BTreeSet::new();
    for key in nfr_behavior_map.keys() {
        all_nfr_keys.insert(key.clone());
    }
    for key in &benchmark_nfrs {
        all_nfr_keys.insert(key.clone());
    }

    for key in &all_nfr_keys {
        let parts: Vec<&str> = key.splitn(2, '#').collect();
        if parts.len() != 2 {
            continue;
        }
        let category = parts[0];
        let constraint = parts[1];

        let has_benchmark = benchmark_nfrs.contains(key);
        let linked_behaviors = nfr_behavior_map.get(key);

        if has_benchmark {
            nfr_coverage.push(NfrCoverage {
                category: category.to_string(),
                constraint: constraint.to_string(),
                coverage_type: NfrCoverageType::Benchmark,
            });
        } else if let Some(behaviors) = linked_behaviors {
            // Check if any linked behavior is covered
            let covered_behaviors: Vec<String> = behaviors
                .iter()
                .filter(|b| {
                    let parts: Vec<&str> = b.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        let sn = parts[0];
                        let bn = parts[1];
                        spec_coverages
                            .get(sn)
                            .and_then(|sc| sc.behaviors.iter().find(|bc| bc.behavior_name == bn))
                            .is_some_and(|bc| !bc.test_types.is_empty())
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            if covered_behaviors.is_empty() {
                nfr_coverage.push(NfrCoverage {
                    category: category.to_string(),
                    constraint: constraint.to_string(),
                    coverage_type: NfrCoverageType::Uncovered,
                });
            } else {
                nfr_coverage.push(NfrCoverage {
                    category: category.to_string(),
                    constraint: constraint.to_string(),
                    coverage_type: NfrCoverageType::Derived,
                });
            }
        }
    }

    // Compute summary
    let total_behaviors: usize = spec_coverages.values().map(|sc| sc.behaviors.len()).sum();
    let covered_behaviors: usize = spec_coverages
        .values()
        .flat_map(|sc| &sc.behaviors)
        .filter(|b| !b.test_types.is_empty())
        .count();

    let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();
    for tag in valid_tags {
        if tag.behavior_ref.is_some() {
            *type_counts.entry(tag.tag_type.clone()).or_insert(0) += 1;
        }
    }

    CoverageReport {
        specs: spec_coverages,
        nfr_coverage,
        total_behaviors,
        covered_behaviors,
        type_counts,
    }
}

// ── Human-readable output ───────────────────────────────

fn print_human_report(report: &CoverageReport, verbose: bool) {
    let color = crate::cli::display::use_color();
    let green = if color {
        crate::cli::display::GREEN
    } else {
        ""
    };
    let red = if color { crate::cli::display::RED } else { "" };
    let yellow = if color {
        crate::cli::display::YELLOW
    } else {
        ""
    };
    let cyan = if color { crate::cli::display::CYAN } else { "" };
    let bold = if color { crate::cli::display::BOLD } else { "" };
    let dim = if color { crate::cli::display::DIM } else { "" };
    let reset = if color {
        crate::cli::display::RESET
    } else {
        ""
    };

    println!("\n{bold}Behavior Coverage{reset}");
    for (spec_name, spec_cov) in &report.specs {
        let total = spec_cov.behaviors.len();
        let covered = spec_cov
            .behaviors
            .iter()
            .filter(|b| !b.test_types.is_empty())
            .count();
        let fully_covered = covered == total && total > 0;

        if fully_covered && !verbose {
            // Collapsed single line: ✓ spec-name vX.Y.Z  N/N [type1, type2]
            let all_types: BTreeSet<&str> = spec_cov
                .behaviors
                .iter()
                .flat_map(|b| b.test_types.iter().map(|s| s.as_str()))
                .collect();
            let types_str: Vec<&str> = all_types.into_iter().collect();
            println!(
                "  {green}\u{2713}{reset} {bold}{} v{}{reset}  {}/{} {cyan}[{}]{reset}",
                spec_name,
                spec_cov.version,
                covered,
                total,
                types_str.join(", ")
            );
        } else {
            // Expanded: header then individual behaviors
            println!("\n{bold}{} v{}{reset}", spec_name, spec_cov.version);

            for beh in &spec_cov.behaviors {
                if beh.test_types.is_empty() {
                    println!(
                        "  {red}\u{2717}{reset} {} {dim}uncovered{reset}",
                        beh.behavior_name
                    );
                } else {
                    let types_str: Vec<&str> = beh.test_types.iter().map(String::as_str).collect();
                    let dup_info = if beh.duplicates.is_empty() {
                        String::new()
                    } else {
                        let dup_parts: Vec<String> = beh
                            .duplicates
                            .iter()
                            .map(|(t, c)| format!("{} x{} duplicate", t, c))
                            .collect();
                        format!(" {yellow}({}){reset}", dup_parts.join(", "))
                    };
                    println!(
                        "  {green}\u{2713}{reset} {} {cyan}[{}]{reset}{}",
                        beh.behavior_name,
                        types_str.join(", "),
                        dup_info
                    );
                }
            }
        }
    }

    // NFR section
    if !report.nfr_coverage.is_empty() {
        println!("\n{bold}NFR Coverage{reset}");
        for nfr_cov in &report.nfr_coverage {
            match &nfr_cov.coverage_type {
                NfrCoverageType::Derived => {
                    println!(
                        "  {green}\u{2713}{reset} {}#{} {dim}[derived]{reset}",
                        nfr_cov.category, nfr_cov.constraint
                    );
                }
                NfrCoverageType::Benchmark => {
                    println!(
                        "  {green}\u{2713}{reset} {}#{} {cyan}[benchmark]{reset}",
                        nfr_cov.category, nfr_cov.constraint
                    );
                }
                NfrCoverageType::Uncovered => {
                    println!(
                        "  {red}\u{2717}{reset} {}#{} {dim}uncovered{reset}",
                        nfr_cov.category, nfr_cov.constraint
                    );
                }
            }
        }
    }

    // Summary
    let pct = if report.total_behaviors > 0 {
        (report.covered_behaviors as f64 / report.total_behaviors as f64 * 100.0) as usize
    } else {
        0
    };

    println!(
        "\n{bold}Summary{reset}: {}/{} behaviors covered ({}%)",
        report.covered_behaviors, report.total_behaviors, pct
    );

    if !report.type_counts.is_empty() {
        let type_parts: Vec<String> = report
            .type_counts
            .iter()
            .map(|(t, c)| format!("{}: {}", t, c))
            .collect();
        println!("  {}", type_parts.join(", "));
    }
}

// ── JSON output ─────────────────────────────────────────

fn print_json_report(report: &CoverageReport) {
    println!("{}", format_json_report(report));
}

fn format_json_report(report: &CoverageReport) -> String {
    let pct = if report.total_behaviors > 0 {
        (report.covered_behaviors as f64 / report.total_behaviors as f64 * 100.0) as usize
    } else {
        0
    };

    let mut specs_json = serde_json::Map::new();
    for (spec_name, spec_cov) in &report.specs {
        let behaviors: Vec<serde_json::Value> = spec_cov
            .behaviors
            .iter()
            .map(|b| {
                let status = if b.test_types.is_empty() {
                    "uncovered"
                } else {
                    "covered"
                };
                let types: Vec<String> = b.test_types.iter().cloned().collect();
                serde_json::json!({
                    "name": b.behavior_name,
                    "status": status,
                    "test_types": types,
                })
            })
            .collect();

        specs_json.insert(
            spec_name.clone(),
            serde_json::json!({
                "version": spec_cov.version,
                "behaviors": behaviors,
            }),
        );
    }

    let nfr_json: Vec<serde_json::Value> = report
        .nfr_coverage
        .iter()
        .map(|n| {
            let status = match &n.coverage_type {
                NfrCoverageType::Derived => "derived",
                NfrCoverageType::Benchmark => "benchmark",
                NfrCoverageType::Uncovered => "uncovered",
            };
            serde_json::json!({
                "category": n.category,
                "constraint": n.constraint,
                "status": status,
            })
        })
        .collect();

    let output = serde_json::json!({
        "total_behaviors": report.total_behaviors,
        "covered_behaviors": report.covered_behaviors,
        "coverage_percentage": pct,
        "specs": specs_json,
        "nfr_coverage": nfr_json,
        "type_counts": report.type_counts,
    });

    serde_json::to_string_pretty(&output).unwrap()
}
