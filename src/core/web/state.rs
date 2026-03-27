use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::commands::coverage::{
    build_behavior_index, build_nfr_index, scan_for_tags, validate_tags,
};
use crate::core::graph::cache::content_hash;
use crate::core::{config, deps, discover, io, parser, validation};
use crate::model::{BehaviorCategory, NfrSpec};

// ── Public types ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IntegrityStatus {
    Aligned,
    Drifted,
    NoLock,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrityInfo {
    pub specs: IntegrityStatus,
    pub nfrs: IntegrityStatus,
    pub tests: IntegrityStatus,
    pub lock_status: IntegrityStatus,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DriftDetails {
    pub modified_specs: Vec<String>,
    pub unlocked_specs: Vec<String>,
    pub modified_nfrs: Vec<String>,
    pub unlocked_nfrs: Vec<String>,
    pub modified_tests: Vec<String>,
    pub missing_tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ValidationStatus {
    Unknown,
    Valid,
    Invalid(Vec<String>),
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecInfo {
    pub name: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub path: PathBuf,
    pub behavior_count: usize,
    pub behaviors: Vec<BehaviorInfo>,
    pub validation_status: ValidationStatus,
    pub nfr_refs: Vec<String>,
    pub dependencies: Vec<String>,
    pub dep_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorInfo {
    pub name: String,
    pub description: String,
    pub covered: bool,
    pub test_types: Vec<String>,
    pub category: String,
    pub nfr_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvalidTagInfo {
    pub file: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Validate,
    DeepValidate,
    Coverage,
    Lock,
    Graph,
    Inspect,
    Format,
    Scaffold,
    Guide,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ActionResult {
    Validate {
        output: String,
        has_errors: bool,
    },
    DeepValidate {
        output: String,
        has_errors: bool,
    },
    Coverage {
        covered: usize,
        total: usize,
        percent: usize,
        uncovered_behaviors: Vec<String>,
    },
    Lock {
        success: bool,
        message: String,
    },
    Graph {
        output: String,
    },
    Inspect {
        output: String,
        behavior_count: usize,
        categories: Vec<(String, usize)>,
        dependencies: Vec<(String, String)>,
    },
    Format {
        output: String,
    },
    Scaffold {
        output: String,
    },
    Guide {
        topics: Vec<String>,
    },
}

// ── Lock file types (private) ───────────────────────────

#[derive(Debug, Deserialize)]
struct LockFile {
    #[allow(dead_code)]
    version: u64,
    specs: BTreeMap<String, LockSpec>,
    #[serde(default)]
    nfrs: BTreeMap<String, LockNfr>,
    #[serde(default)]
    benchmark_files: BTreeMap<String, LockBenchmarkFile>,
}

#[derive(Debug, Deserialize)]
struct LockBenchmarkFile {
    hash: String,
}

#[derive(Debug, Deserialize)]
struct LockSpec {
    hash: String,
    #[allow(dead_code)]
    behaviors: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    dependencies: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    nfrs: Vec<String>,
    #[serde(default)]
    test_files: BTreeMap<String, LockTestFile>,
}

#[derive(Debug, Deserialize)]
struct LockNfr {
    hash: String,
}

#[derive(Debug, Deserialize)]
struct LockTestFile {
    hash: String,
    #[serde(default)]
    #[allow(dead_code)]
    covers: Vec<String>,
}

// ── UiState ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct UiState {
    #[serde(skip)]
    working_dir: PathBuf,
    specs: Vec<SpecInfo>,
    nfr_count: usize,
    test_count: usize,
    coverage_covered: usize,
    coverage_total: usize,
    integrity: IntegrityInfo,
    drift: DriftDetails,
    errors: Vec<String>,
    invalid_tags: Vec<InvalidTagInfo>,
    dep_errors: Vec<String>,
}

impl UiState {
    /// Load UI state from the given working directory.
    pub fn load(working_dir: &Path) -> Self {
        let mut state = UiState {
            working_dir: working_dir.to_path_buf(),
            specs: Vec::new(),
            nfr_count: 0,
            test_count: 0,
            coverage_covered: 0,
            coverage_total: 0,
            integrity: IntegrityInfo {
                specs: IntegrityStatus::NoLock,
                nfrs: IntegrityStatus::NoLock,
                tests: IntegrityStatus::NoLock,
                lock_status: IntegrityStatus::NoLock,
            },
            drift: DriftDetails::default(),
            errors: Vec::new(),
            invalid_tags: Vec::new(),
            dep_errors: Vec::new(),
        };

        state.load_from_dir(working_dir);
        state
    }

    fn load_from_dir(&mut self, working_dir: &Path) {
        // Resolve config
        let config = match config::load_config(working_dir) {
            Ok(c) => c,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        let specs_dir = &config.specs;
        let test_dirs = &config.tests;

        // Check if specs directory exists
        if !specs_dir.exists() {
            self.errors.push(format!(
                "specs directory '{}' not found",
                specs_dir.display()
            ));
            return;
        }

        // Discover and parse specs
        let spec_files = match discover::discover_spec_files(specs_dir) {
            Ok(files) => files,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        let mut parsed_specs: Vec<(
            PathBuf,
            String,
            Option<crate::model::Spec>,
            ValidationStatus,
        )> = Vec::new();
        for path in &spec_files {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(e) => {
                    parsed_specs.push((
                        path.clone(),
                        name,
                        None,
                        ValidationStatus::Invalid(vec![e.to_string()]),
                    ));
                    continue;
                }
            };
            match parser::parse(&source) {
                Ok(spec) => {
                    let status = match crate::core::validation::semantic::validate(&spec) {
                        Ok(()) => ValidationStatus::Valid,
                        Err(errors) => ValidationStatus::Invalid(
                            errors.iter().map(|e| e.to_string()).collect(),
                        ),
                    };
                    parsed_specs.push((path.clone(), spec.name.clone(), Some(spec), status));
                }
                Err(errors) => {
                    parsed_specs.push((
                        path.clone(),
                        name,
                        None,
                        ValidationStatus::Invalid(
                            errors.iter().map(|e| e.message.clone()).collect(),
                        ),
                    ));
                }
            }
        }

        // Discover and count NFR constraints (not files)
        let nfr_files = discover::discover_nfr_files(specs_dir);
        let mut nfr_constraint_count: usize = 0;
        let mut nfr_specs_map: HashMap<String, NfrSpec> = HashMap::new();
        for nfr_path in &nfr_files {
            if let Ok(source) = io::read_file_safe(nfr_path) {
                if let Ok(nfr) = parser::parse_nfr(&source) {
                    nfr_constraint_count += nfr.constraints.len();
                    nfr_specs_map.insert(nfr.category.clone(), nfr);
                }
            }
        }
        self.nfr_count = nfr_constraint_count;

        // Scan for test tags
        let existing_test_dirs: Vec<PathBuf> =
            test_dirs.iter().filter(|p| p.exists()).cloned().collect();
        let tags = if existing_test_dirs.is_empty() {
            Vec::new()
        } else {
            scan_for_tags(&existing_test_dirs)
        };

        // Count test tags (each @minter annotation = one test)
        self.test_count = tags
            .iter()
            .filter(|t| !t.tag_type.is_empty() && !t.ids.is_empty())
            .count();

        // Validate tags against known behaviors/NFRs to detect invalid references
        let indexed_specs: Vec<(String, crate::model::Spec)> = parsed_specs
            .iter()
            .filter_map(|(_, name, maybe_spec, _)| {
                maybe_spec.as_ref().map(|s| (name.clone(), s.clone()))
            })
            .collect();
        let behavior_index = build_behavior_index(&indexed_specs);
        let nfr_index = build_nfr_index(&nfr_specs_map);
        let (_valid_tags, tag_errors, _warnings) =
            validate_tags(&tags, &behavior_index, &nfr_index, &indexed_specs);
        self.invalid_tags = tag_errors
            .iter()
            .map(|e| InvalidTagInfo {
                file: e.file.display().to_string(),
                line: e.line,
                message: e.message.clone(),
            })
            .collect();

        // Build coverage data: behavior_name -> set of test types
        let mut behavior_coverage: HashMap<String, BTreeSet<String>> = HashMap::new();
        for tag in &tags {
            if tag.tag_type.is_empty() || tag.ids.is_empty() {
                continue;
            }
            if tag.tag_type == "benchmark" {
                continue; // benchmark tags map to NFR constraints, not behaviors
            }
            for id in &tag.ids {
                if id.starts_with('#') {
                    continue;
                }
                let behavior_name = if let Some((_spec, behavior)) = id.split_once('/') {
                    behavior.to_string()
                } else {
                    id.clone()
                };
                behavior_coverage
                    .entry(behavior_name)
                    .or_default()
                    .insert(tag.tag_type.clone());
            }
        }

        // Build spec info list
        let mut spec_infos: Vec<SpecInfo> = Vec::new();
        let mut total_behaviors: usize = 0;
        let mut covered_behaviors: usize = 0;

        for (path, name, maybe_spec, status) in &parsed_specs {
            if let Some(spec) = maybe_spec {
                let mut behaviors = Vec::new();
                for b in &spec.behaviors {
                    let test_types = behavior_coverage
                        .get(&b.name)
                        .map(|types| types.iter().cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    let covered = !test_types.is_empty();
                    let category = match b.category {
                        BehaviorCategory::HappyPath => "happy_path",
                        BehaviorCategory::ErrorCase => "error_case",
                        BehaviorCategory::EdgeCase => "edge_case",
                    }
                    .to_string();
                    let behavior_nfr_refs: Vec<String> = b
                        .nfr_refs
                        .iter()
                        .map(|r| format!("{}#{}", r.category, r.anchor))
                        .collect();
                    behaviors.push(BehaviorInfo {
                        name: b.name.clone(),
                        description: b.description.clone(),
                        covered,
                        test_types,
                        category,
                        nfr_refs: behavior_nfr_refs,
                    });
                    total_behaviors += 1;
                    if covered {
                        covered_behaviors += 1;
                    }
                }

                let nfr_refs: Vec<String> = spec
                    .nfr_refs
                    .iter()
                    .map(|r| match &r.anchor {
                        Some(anchor) => format!("{}#{}", r.category, anchor),
                        None => r.category.clone(),
                    })
                    .collect();
                let dependencies: Vec<String> = spec
                    .dependencies
                    .iter()
                    .map(|d| format!("{} >= {}", d.spec_name, d.version_constraint))
                    .collect();

                spec_infos.push(SpecInfo {
                    name: name.clone(),
                    version: spec.version.clone(),
                    title: spec.title.clone(),
                    description: spec.description.clone(),
                    path: path.clone(),
                    behavior_count: spec.behaviors.len(),
                    behaviors,
                    validation_status: status.clone(),
                    nfr_refs,
                    dependencies,
                    dep_errors: Vec::new(),
                });
            } else {
                // Spec failed to parse — show it with error status
                spec_infos.push(SpecInfo {
                    name: name.clone(),
                    version: String::new(),
                    title: String::new(),
                    description: String::new(),
                    path: path.clone(),
                    behavior_count: 0,
                    behaviors: Vec::new(),
                    validation_status: status.clone(),
                    nfr_refs: Vec::new(),
                    dependencies: Vec::new(),
                    dep_errors: Vec::new(),
                });
            }
        }

        spec_infos.sort_by(|a, b| a.name.cmp(&b.name));

        self.specs = spec_infos;
        self.coverage_total = total_behaviors;
        self.coverage_covered = covered_behaviors;

        // Check dependency structure
        self.load_dep_errors(&parsed_specs);

        // Load integrity
        self.load_integrity(working_dir, specs_dir, test_dirs);
    }

    fn load_dep_errors(
        &mut self,
        parsed_specs: &[(
            PathBuf,
            String,
            Option<crate::model::Spec>,
            ValidationStatus,
        )],
    ) {
        // Build siblings map: spec name -> path for dependency resolution
        let mut siblings: HashMap<String, PathBuf> = HashMap::new();
        for (path, _, maybe_spec, _) in parsed_specs {
            if let Some(spec) = maybe_spec {
                siblings.insert(spec.name.clone(), path.clone());
            }
        }

        // Resolve dependencies for each spec and store errors per-spec
        for (_, _, maybe_spec, _) in parsed_specs {
            if let Some(spec) = maybe_spec {
                if spec.dependencies.is_empty() {
                    continue;
                }
                let mut ctx = deps::ResolutionContext {
                    siblings: siblings.clone(),
                    resolved: HashMap::new(),
                    stack: vec![spec.name.clone()],
                    errors: Vec::new(),
                };
                deps::resolve_and_collect(&spec.dependencies, &mut ctx, 1);
                if !ctx.errors.is_empty() {
                    // Store errors in the corresponding SpecInfo
                    if let Some(spec_info) = self.specs.iter_mut().find(|s| s.name == spec.name) {
                        spec_info.dep_errors = ctx.errors.clone();
                    }
                    // Also keep in global list for backward compat
                    for err in &ctx.errors {
                        self.dep_errors.push(format!("{}: {}", spec.name, err));
                    }
                }
            }
        }
    }

    fn load_integrity(&mut self, working_dir: &Path, specs_dir: &Path, test_dirs: &[PathBuf]) {
        let lock_path = working_dir.join("minter.lock");
        if !lock_path.exists() {
            self.integrity = IntegrityInfo {
                specs: IntegrityStatus::NoLock,
                nfrs: IntegrityStatus::NoLock,
                tests: IntegrityStatus::NoLock,
                lock_status: IntegrityStatus::NoLock,
            };
            return;
        }

        let lock_content = match std::fs::read_to_string(&lock_path) {
            Ok(c) => c,
            Err(_) => {
                self.integrity = IntegrityInfo {
                    specs: IntegrityStatus::NoLock,
                    nfrs: IntegrityStatus::NoLock,
                    tests: IntegrityStatus::NoLock,
                    lock_status: IntegrityStatus::NoLock,
                };
                return;
            }
        };

        let lock: LockFile = match serde_json::from_str(&lock_content) {
            Ok(l) => l,
            Err(_) => {
                self.integrity = IntegrityInfo {
                    specs: IntegrityStatus::NoLock,
                    nfrs: IntegrityStatus::NoLock,
                    tests: IntegrityStatus::NoLock,
                    lock_status: IntegrityStatus::NoLock,
                };
                return;
            }
        };

        let mut drift = DriftDetails::default();

        // Discover current disk state
        let disk_spec_files = discover::discover_spec_files(specs_dir).unwrap_or_default();
        let disk_nfr_files = discover::discover_nfr_files(specs_dir);

        // Build relative path maps
        let mut disk_specs_rel: BTreeMap<String, PathBuf> = BTreeMap::new();
        for path in &disk_spec_files {
            let rel = io::make_relative(path, working_dir);
            disk_specs_rel.insert(rel, path.clone());
        }

        let mut disk_nfrs_rel: BTreeMap<String, PathBuf> = BTreeMap::new();
        for path in &disk_nfr_files {
            let rel = io::make_relative(path, working_dir);
            disk_nfrs_rel.insert(rel, path.clone());
        }

        // Check spec integrity
        let mut spec_drifted = false;
        for (lock_path_str, lock_spec) in &lock.specs {
            let abs_path = working_dir.join(lock_path_str);
            if !abs_path.exists() {
                drift.modified_specs.push(lock_path_str.clone());
                spec_drifted = true;
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&abs_path) {
                let current_hash = content_hash(&content);
                if current_hash != lock_spec.hash {
                    drift.modified_specs.push(lock_path_str.clone());
                    spec_drifted = true;
                }
            }
        }
        for rel_path in disk_specs_rel.keys() {
            if !lock.specs.contains_key(rel_path) {
                drift.unlocked_specs.push(rel_path.clone());
                spec_drifted = true;
            }
        }

        // Check NFR integrity
        let mut nfr_drifted = false;
        for (lock_path_str, lock_nfr) in &lock.nfrs {
            let abs_path = working_dir.join(lock_path_str);
            if !abs_path.exists() {
                drift.modified_nfrs.push(lock_path_str.clone());
                nfr_drifted = true;
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&abs_path) {
                let current_hash = content_hash(&content);
                if current_hash != lock_nfr.hash {
                    drift.modified_nfrs.push(lock_path_str.clone());
                    nfr_drifted = true;
                }
            }
        }
        for rel_path in disk_nfrs_rel.keys() {
            if !lock.nfrs.contains_key(rel_path) {
                drift.unlocked_nfrs.push(rel_path.clone());
                nfr_drifted = true;
            }
        }

        // Check test integrity
        let mut test_drifted = false;

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

        // Scan for current tagged test files on disk
        let existing_test_dirs: Vec<PathBuf> =
            test_dirs.iter().filter(|p| p.exists()).cloned().collect();
        let tags = if existing_test_dirs.is_empty() {
            Vec::new()
        } else {
            scan_for_tags(&existing_test_dirs)
        };

        let mut disk_tagged_tests: BTreeMap<String, String> = BTreeMap::new();
        for tag in &tags {
            if tag.tag_type.is_empty() || tag.ids.is_empty() {
                continue;
            }

            let rel_path = io::make_relative(&tag.file, working_dir);
            if let std::collections::btree_map::Entry::Vacant(e) = disk_tagged_tests.entry(rel_path)
            {
                if let Ok(content) = std::fs::read_to_string(&tag.file) {
                    e.insert(content_hash(&content));
                }
            }
        }

        for (lock_test_path, lock_hash) in &lock_test_files {
            match disk_tagged_tests.get(lock_test_path) {
                None => {
                    let abs_path = working_dir.join(lock_test_path);
                    if !abs_path.exists() {
                        drift.missing_tests.push(lock_test_path.clone());
                        test_drifted = true;
                    } else if let Ok(content) = std::fs::read_to_string(&abs_path) {
                        let current_hash = content_hash(&content);
                        if current_hash != *lock_hash {
                            drift.modified_tests.push(lock_test_path.clone());
                            test_drifted = true;
                        }
                    }
                }
                Some(current_hash) => {
                    if current_hash != lock_hash {
                        drift.modified_tests.push(lock_test_path.clone());
                        test_drifted = true;
                    }
                }
            }
        }

        for rel_path in disk_tagged_tests.keys() {
            if !lock_test_files.contains_key(rel_path) {
                test_drifted = true;
            }
        }

        let specs_status = if spec_drifted {
            IntegrityStatus::Drifted
        } else {
            IntegrityStatus::Aligned
        };
        let nfrs_status = if nfr_drifted {
            IntegrityStatus::Drifted
        } else {
            IntegrityStatus::Aligned
        };
        let tests_status = if test_drifted {
            IntegrityStatus::Drifted
        } else {
            IntegrityStatus::Aligned
        };

        let any_drifted = spec_drifted || nfr_drifted || test_drifted;
        let lock_status = if any_drifted {
            IntegrityStatus::Drifted
        } else {
            IntegrityStatus::Aligned
        };

        self.integrity = IntegrityInfo {
            specs: specs_status,
            nfrs: nfrs_status,
            tests: tests_status,
            lock_status,
        };
        self.drift = drift;
    }

    // ── Public accessors ────────────────────────────────

    pub fn spec_count(&self) -> usize {
        self.specs.len()
    }

    pub fn behavior_count(&self) -> usize {
        self.coverage_total
    }

    pub fn nfr_count(&self) -> usize {
        self.nfr_count
    }

    pub fn test_count(&self) -> usize {
        self.test_count
    }

    pub fn coverage_percent(&self) -> usize {
        if self.coverage_total == 0 {
            return 0;
        }
        (self.coverage_covered as f64 / self.coverage_total as f64 * 100.0) as usize
    }

    pub fn lock_aligned(&self) -> bool {
        self.integrity.lock_status == IntegrityStatus::Aligned
    }

    pub fn specs_list(&self) -> &[SpecInfo] {
        &self.specs
    }

    pub fn integrity(&self) -> &IntegrityInfo {
        &self.integrity
    }

    pub fn drift_details(&self) -> &DriftDetails {
        &self.drift
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.errors.first().map(|s| s.as_str())
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn invalid_tags(&self) -> &[InvalidTagInfo] {
        &self.invalid_tags
    }

    pub fn dep_errors(&self) -> &[String] {
        &self.dep_errors
    }

    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    // ── Refresh ─────────────────────────────────────────

    pub fn refresh(&mut self) {
        let wd = self.working_dir.clone();
        self.specs.clear();
        self.nfr_count = 0;
        self.test_count = 0;
        self.coverage_covered = 0;
        self.coverage_total = 0;
        self.errors.clear();
        self.invalid_tags.clear();
        self.dep_errors.clear();
        self.drift = DriftDetails::default();
        self.load_from_dir(&wd);
    }

    // ── Validation ─────────────────────────────────────

    /// Validate all specs and update their `validation_status`.
    pub fn validate_all(&mut self) {
        for spec in &mut self.specs {
            let source = match io::read_file_safe(&spec.path) {
                Ok(s) => s,
                Err(e) => {
                    spec.validation_status = ValidationStatus::Invalid(vec![e.to_string()]);
                    continue;
                }
            };
            match parser::parse(&source) {
                Ok(parsed) => match validation::validate(&parsed) {
                    Ok(()) => {
                        spec.validation_status = ValidationStatus::Valid;
                    }
                    Err(errors) => {
                        spec.validation_status = ValidationStatus::Invalid(
                            errors.iter().map(|e| e.to_string()).collect(),
                        );
                    }
                },
                Err(errors) => {
                    spec.validation_status = ValidationStatus::Invalid(
                        errors.iter().map(|e| e.message.clone()).collect(),
                    );
                }
            }
        }
    }

    // ── Actions ─────────────────────────────────────────

    /// Get the spec path for a given index (for targeted actions).
    pub fn spec_path(&self, index: usize) -> Option<&Path> {
        self.specs.get(index).map(|s| s.path.as_path())
    }

    /// Find a spec by its file path.
    pub fn spec_path_by_file(&self, file_path: &Path) -> Option<&Path> {
        self.specs
            .iter()
            .find(|s| s.path == file_path)
            .map(|s| s.path.as_path())
    }

    pub fn run_action(&self, action: Action, spec_path: Option<&Path>) -> ActionResult {
        match action {
            Action::Validate => self.action_validate(spec_path),
            Action::DeepValidate => self.action_deep_validate(spec_path),
            Action::Coverage => self.action_coverage(spec_path),
            Action::Lock => self.action_lock(),
            Action::Graph => self.action_graph(spec_path),
            Action::Inspect => self.action_inspect(spec_path),
            Action::Format => self.action_format(),
            Action::Scaffold => self.action_scaffold(),
            Action::Guide => self.action_guide(),
        }
    }

    fn action_validate(&self, spec_path: Option<&Path>) -> ActionResult {
        let spec_files = if let Some(path) = spec_path {
            vec![path.to_path_buf()]
        } else {
            let config = match config::load_config(&self.working_dir) {
                Ok(c) => c,
                Err(e) => {
                    return ActionResult::Validate {
                        output: e,
                        has_errors: true,
                    };
                }
            };

            let specs_dir = &config.specs;
            if !specs_dir.exists() {
                return ActionResult::Validate {
                    output: format!("specs directory '{}' not found", specs_dir.display()),
                    has_errors: true,
                };
            }

            match discover::discover_spec_files(specs_dir) {
                Ok(files) => files,
                Err(e) => {
                    return ActionResult::Validate {
                        output: e,
                        has_errors: true,
                    };
                }
            }
        };

        let mut output_lines: Vec<String> = Vec::new();
        let mut has_errors = false;

        for path in &spec_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(e) => {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    output_lines.push(format!("{}: error reading file: {}", name, e));
                    has_errors = true;
                    continue;
                }
            };

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            match parser::parse(&source) {
                Ok(spec) => {
                    if let Err(errors) = crate::core::validation::semantic::validate(&spec) {
                        for e in &errors {
                            output_lines.push(format!("{}: {}", name, e));
                        }
                        has_errors = true;
                    } else {
                        output_lines.push(format!("{}: ok", name));
                    }
                }
                Err(errors) => {
                    for e in &errors {
                        output_lines.push(format!("{}: {}", name, e));
                    }
                    has_errors = true;
                }
            }
        }

        ActionResult::Validate {
            output: output_lines.join("\n"),
            has_errors,
        }
    }

    fn action_deep_validate(&self, spec_path: Option<&Path>) -> ActionResult {
        let config = match config::load_config(&self.working_dir) {
            Ok(c) => c,
            Err(e) => {
                return ActionResult::DeepValidate {
                    output: e,
                    has_errors: true,
                };
            }
        };

        let specs_dir = &config.specs;
        if !specs_dir.exists() {
            return ActionResult::DeepValidate {
                output: format!("specs directory '{}' not found", specs_dir.display()),
                has_errors: true,
            };
        }

        let spec_files = if let Some(path) = spec_path {
            vec![path.to_path_buf()]
        } else {
            match discover::discover_spec_files(specs_dir) {
                Ok(files) => files,
                Err(e) => {
                    return ActionResult::DeepValidate {
                        output: e,
                        has_errors: true,
                    };
                }
            }
        };

        // Discover sibling specs for dependency resolution
        let all_spec_files = discover::discover_spec_files(specs_dir).unwrap_or_default();
        let mut siblings: HashMap<String, PathBuf> = HashMap::new();
        for p in &all_spec_files {
            if let Ok(src) = io::read_file_safe(p) {
                if let Ok(s) = parser::parse(&src) {
                    siblings.insert(s.name.clone(), p.clone());
                }
            }
        }

        // Discover NFR specs for cross-reference validation
        let nfr_files = discover::discover_nfr_files(specs_dir);
        let mut nfr_map: HashMap<String, crate::model::NfrSpec> = HashMap::new();
        for nfr_path in &nfr_files {
            if let Ok(nfr_source) = io::read_file_safe(nfr_path) {
                if let Ok(nfr) = parser::parse_nfr(&nfr_source) {
                    nfr_map.insert(nfr.category.clone(), nfr);
                }
            }
        }

        let mut output_lines: Vec<String> = Vec::new();
        let mut has_errors = false;

        for path in &spec_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(e) => {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    output_lines.push(format!("{}: error reading file: {}", name, e));
                    has_errors = true;
                    continue;
                }
            };

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let mut spec_ok = true;

            match parser::parse(&source) {
                Ok(spec) => {
                    // Semantic validation
                    if let Err(errors) = crate::core::validation::semantic::validate(&spec) {
                        for e in &errors {
                            output_lines.push(format!("{}: {}", name, e));
                        }
                        has_errors = true;
                        spec_ok = false;
                    }

                    // Dependency resolution
                    if !spec.dependencies.is_empty() {
                        let mut ctx = crate::core::deps::ResolutionContext {
                            siblings: siblings.clone(),
                            resolved: HashMap::new(),
                            stack: vec![spec.name.clone()],
                            errors: Vec::new(),
                        };
                        crate::core::deps::resolve_and_collect(&spec.dependencies, &mut ctx, 1);
                        if !ctx.errors.is_empty() {
                            for e in &ctx.errors {
                                output_lines.push(format!("{}: {}", name, e));
                            }
                            has_errors = true;
                            spec_ok = false;
                        }
                    }

                    // NFR cross-reference validation
                    if let Err(crossref_errors) =
                        crate::core::validation::crossref::cross_validate(&spec, &nfr_map)
                    {
                        for e in &crossref_errors {
                            output_lines.push(format!("{}: {}", name, e));
                        }
                        has_errors = true;
                        spec_ok = false;
                    }

                    if spec_ok {
                        output_lines.push(format!("{}: ok (deep)", name));
                    }
                }
                Err(errors) => {
                    for e in &errors {
                        output_lines.push(format!("{}: {}", name, e));
                    }
                    has_errors = true;
                }
            }
        }

        ActionResult::DeepValidate {
            output: output_lines.join("\n"),
            has_errors,
        }
    }

    fn action_coverage(&self, spec_path: Option<&Path>) -> ActionResult {
        let config = match config::load_config(&self.working_dir) {
            Ok(c) => c,
            Err(_) => {
                return ActionResult::Coverage {
                    covered: 0,
                    total: 0,
                    percent: 0,
                    uncovered_behaviors: Vec::new(),
                };
            }
        };

        let specs_dir = &config.specs;
        let test_dirs = &config.tests;

        if !specs_dir.exists() {
            return ActionResult::Coverage {
                covered: 0,
                total: 0,
                percent: 0,
                uncovered_behaviors: Vec::new(),
            };
        }

        let spec_files = if let Some(path) = spec_path {
            vec![path.to_path_buf()]
        } else {
            match discover::discover_spec_files(specs_dir) {
                Ok(files) => files,
                Err(_) => {
                    return ActionResult::Coverage {
                        covered: 0,
                        total: 0,
                        percent: 0,
                        uncovered_behaviors: Vec::new(),
                    };
                }
            }
        };

        let mut all_behaviors: Vec<String> = Vec::new();
        for path in &spec_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Ok(spec) = parser::parse(&source) {
                for b in &spec.behaviors {
                    all_behaviors.push(b.name.clone());
                }
            }
        }

        let existing_test_dirs: Vec<PathBuf> =
            test_dirs.iter().filter(|p| p.exists()).cloned().collect();
        let tags = if existing_test_dirs.is_empty() {
            Vec::new()
        } else {
            scan_for_tags(&existing_test_dirs)
        };

        let mut covered_set: HashSet<String> = HashSet::new();
        for tag in &tags {
            if tag.tag_type.is_empty() || tag.ids.is_empty() {
                continue;
            }
            if tag.tag_type == "benchmark" {
                continue; // benchmark tags map to NFR constraints, not behaviors
            }
            for id in &tag.ids {
                if id.starts_with('#') {
                    continue;
                }
                let behavior_name = if let Some((_spec, behavior)) = id.split_once('/') {
                    behavior.to_string()
                } else {
                    id.clone()
                };
                covered_set.insert(behavior_name);
            }
        }

        let total = all_behaviors.len();
        let covered = all_behaviors
            .iter()
            .filter(|b| covered_set.contains(b.as_str()))
            .count();
        let uncovered_behaviors: Vec<String> = all_behaviors
            .iter()
            .filter(|b| !covered_set.contains(b.as_str()))
            .cloned()
            .collect();
        let percent = if total > 0 {
            (covered as f64 / total as f64 * 100.0) as usize
        } else {
            0
        };

        ActionResult::Coverage {
            covered,
            total,
            percent,
            uncovered_behaviors,
        }
    }

    fn action_lock(&self) -> ActionResult {
        let config = match config::load_config(&self.working_dir) {
            Ok(c) => c,
            Err(e) => {
                return ActionResult::Lock {
                    success: false,
                    message: e,
                };
            }
        };

        let result = self.generate_lock(&config);
        ActionResult::Lock {
            success: result.0,
            message: result.1,
        }
    }

    fn generate_lock(&self, config: &config::ProjectConfig) -> (bool, String) {
        let specs_dir = &config.specs;
        let test_dirs = &config.tests;

        let spec_files = match discover::discover_spec_files(specs_dir) {
            Ok(files) => files,
            Err(e) => return (false, e),
        };

        if spec_files.is_empty() {
            return (false, "no spec files found".to_string());
        }

        // Parse all specs
        let mut parsed_specs: Vec<(PathBuf, String, crate::model::Spec)> = Vec::new();
        for path in &spec_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(e) => return (false, format!("cannot read {}: {}", path.display(), e)),
            };
            match parser::parse(&source) {
                Ok(spec) => {
                    parsed_specs.push((path.clone(), source, spec));
                }
                Err(errors) => {
                    return (false, format!("{}: {}", path.display(), errors[0]));
                }
            }
        }

        // Discover NFR files
        let nfr_files = discover::discover_nfr_files(specs_dir);
        let mut nfr_sources: HashMap<String, (PathBuf, String)> = HashMap::new();
        for path in &nfr_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Ok(nfr) = parser::parse_nfr(&source) {
                nfr_sources.insert(nfr.category.clone(), (path.clone(), source));
            }
        }

        // Build behavior index for tag validation
        let specs_for_index: Vec<(String, crate::model::Spec)> = parsed_specs
            .iter()
            .map(|(_, _, spec)| (spec.name.clone(), spec.clone()))
            .collect();

        let behavior_index =
            crate::core::commands::coverage::build_behavior_index(&specs_for_index);

        // Scan test directories
        let existing_test_dirs: Vec<PathBuf> =
            test_dirs.iter().filter(|p| p.exists()).cloned().collect();
        let tags = if existing_test_dirs.is_empty() {
            Vec::new()
        } else {
            scan_for_tags(&existing_test_dirs)
        };

        // Build spec name -> rel path mapping
        let mut spec_name_to_rel_path: HashMap<String, String> = HashMap::new();
        for (path, _, spec) in &parsed_specs {
            let rel_path = io::make_relative(path, &self.working_dir);
            spec_name_to_rel_path.insert(spec.name.clone(), rel_path);
        }

        // Build lock specs
        let mut lock_specs: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        for (path, source, spec) in &parsed_specs {
            let rel_path = io::make_relative(path, &self.working_dir);
            let hash = content_hash(source);
            let behaviors: Vec<String> = spec.behaviors.iter().map(|b| b.name.clone()).collect();
            let dependencies: Vec<String> = spec
                .dep_names()
                .iter()
                .filter_map(|dep_name| {
                    parsed_specs
                        .iter()
                        .find(|(_, _, s)| s.name == *dep_name)
                        .map(|(p, _, _)| io::make_relative(p, &self.working_dir))
                })
                .collect();
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

        // Process tags to build test_files mapping
        let mut test_file_behaviors: HashMap<(String, String), BTreeSet<String>> = HashMap::new();
        let mut test_file_hashes: HashMap<String, String> = HashMap::new();
        let mut benchmark_file_hashes: BTreeMap<String, String> = BTreeMap::new();

        for tag in &tags {
            if tag.tag_type.is_empty() || tag.ids.is_empty() {
                continue;
            }

            let test_rel_path = io::make_relative(&tag.file, &self.working_dir);
            if !test_file_hashes.contains_key(&test_rel_path)
                && !benchmark_file_hashes.contains_key(&test_rel_path)
            {
                if let Ok(content) = io::read_file_safe(&tag.file) {
                    let hash = content_hash(&content);
                    if tag.tag_type == "benchmark" {
                        benchmark_file_hashes.insert(test_rel_path.clone(), hash);
                    } else {
                        test_file_hashes.insert(test_rel_path.clone(), hash);
                    }
                }
            }

            if tag.tag_type == "benchmark" {
                continue; // benchmarks are NFR-only, not behavior coverage
            }

            for id in &tag.ids {
                if id.starts_with('#') {
                    continue;
                }
                if let Some((spec_name, behavior_name)) = id.split_once('/') {
                    if let Some(spec_rel_path) = spec_name_to_rel_path.get(spec_name) {
                        test_file_behaviors
                            .entry((spec_rel_path.clone(), test_rel_path.clone()))
                            .or_default()
                            .insert(behavior_name.to_string());
                    }
                } else if let Some(spec_names) = behavior_index.get(id.as_str()) {
                    if spec_names.len() == 1 {
                        if let Some(spec_rel_path) = spec_name_to_rel_path.get(&spec_names[0]) {
                            test_file_behaviors
                                .entry((spec_rel_path.clone(), test_rel_path.clone()))
                                .or_default()
                                .insert(id.clone());
                        }
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
        for (path, source) in nfr_sources.values() {
            let rel_path = io::make_relative(path, &self.working_dir);
            let hash = content_hash(source);
            lock_nfrs.insert(rel_path, serde_json::json!({ "hash": hash }));
        }

        // Build benchmark_files section
        let mut lock_benchmarks: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        for (rel_path, hash) in &benchmark_file_hashes {
            lock_benchmarks.insert(rel_path.clone(), serde_json::json!({ "hash": hash }));
        }

        let lock = serde_json::json!({
            "version": 1,
            "specs": lock_specs,
            "nfrs": lock_nfrs,
            "benchmark_files": lock_benchmarks,
        });

        let json = match serde_json::to_string_pretty(&lock) {
            Ok(j) => j,
            Err(e) => return (false, format!("failed to serialize lock: {}", e)),
        };

        let lock_path = self.working_dir.join("minter.lock");
        let tmp_path = lock_path.with_extension("lock.tmp");
        if let Err(e) = std::fs::write(&tmp_path, &json) {
            return (false, format!("failed to write lock file: {}", e));
        }
        if let Err(e) = std::fs::rename(&tmp_path, &lock_path) {
            return (false, format!("failed to finalize lock file: {}", e));
        }

        (true, "lock file generated successfully".to_string())
    }

    fn action_graph(&self, spec_path: Option<&Path>) -> ActionResult {
        let config = match config::load_config(&self.working_dir) {
            Ok(c) => c,
            Err(e) => {
                return ActionResult::Graph { output: e };
            }
        };

        let specs_dir = &config.specs;
        if !specs_dir.exists() {
            return ActionResult::Graph {
                output: "specs directory not found".to_string(),
            };
        }

        let spec_files = if let Some(path) = spec_path {
            vec![path.to_path_buf()]
        } else {
            match discover::discover_spec_files(specs_dir) {
                Ok(files) => files,
                Err(e) => {
                    return ActionResult::Graph { output: e };
                }
            }
        };

        let mut output_lines: Vec<String> = Vec::new();
        let mut specs: Vec<(String, String, Vec<String>)> = Vec::new();

        for path in &spec_files {
            let source = match io::read_file_safe(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Ok(spec) = parser::parse(&source) {
                let deps = spec.dep_names();
                specs.push((spec.name.clone(), spec.version.clone(), deps));
            }
        }

        for (name, version, deps) in &specs {
            if deps.is_empty() {
                output_lines.push(format!("{} v{}", name, version));
            } else {
                let dep_strs: Vec<String> = deps.iter().map(|d| format!("  -> {}", d)).collect();
                output_lines.push(format!("{} v{}", name, version));
                output_lines.extend(dep_strs);
            }
        }

        ActionResult::Graph {
            output: output_lines.join("\n"),
        }
    }

    fn action_inspect(&self, spec_path: Option<&Path>) -> ActionResult {
        let path = match spec_path {
            Some(p) => p.to_path_buf(),
            None => {
                return ActionResult::Inspect {
                    output: "inspect requires a selected spec".to_string(),
                    behavior_count: 0,
                    categories: Vec::new(),
                    dependencies: Vec::new(),
                };
            }
        };

        let source = match io::read_file_safe(&path) {
            Ok(s) => s,
            Err(e) => {
                return ActionResult::Inspect {
                    output: format!("error reading file: {}", e),
                    behavior_count: 0,
                    categories: Vec::new(),
                    dependencies: Vec::new(),
                };
            }
        };

        let spec = match parser::parse(&source) {
            Ok(s) => s,
            Err(errors) => {
                return ActionResult::Inspect {
                    output: errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    behavior_count: 0,
                    categories: Vec::new(),
                    dependencies: Vec::new(),
                };
            }
        };

        let result = crate::core::commands::inspect_core::inspect_spec(&spec);

        let mut output_lines: Vec<String> = Vec::new();
        output_lines.push(format!("{} v{}", result.name, result.version));
        output_lines.push(format!("title: {}", result.title));
        output_lines.push(format!(
            "{} {}",
            result.behavior_count,
            if result.behavior_count == 1 {
                "behavior"
            } else {
                "behaviors"
            }
        ));
        for (cat, n) in &result.categories {
            output_lines.push(format!("  {}: {}", cat, n));
        }
        if !result.dependencies.is_empty() {
            output_lines.push("dependencies:".to_string());
            for (name, ver) in &result.dependencies {
                output_lines.push(format!("  {} >= {}", name, ver));
            }
        }

        ActionResult::Inspect {
            output: output_lines.join("\n"),
            behavior_count: result.behavior_count,
            categories: result.categories,
            dependencies: result.dependencies,
        }
    }

    fn action_format(&self) -> ActionResult {
        let mut output = String::new();
        output.push_str(crate::core::content::fr_grammar());
        output.push_str("\n\n");
        output.push_str(crate::core::content::nfr_grammar());
        ActionResult::Format { output }
    }

    fn action_scaffold(&self) -> ActionResult {
        ActionResult::Scaffold {
            output: crate::core::content::fr_scaffold().to_string(),
        }
    }

    fn action_guide(&self) -> ActionResult {
        ActionResult::Guide {
            topics: crate::model::VALID_GUIDE_TOPICS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}
