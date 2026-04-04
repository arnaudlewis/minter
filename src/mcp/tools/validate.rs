use std::collections::HashMap;
use std::path::Path;

use rmcp::model::*;

use crate::core::commands::validate_core;
use crate::core::graph::{self, discover_and_parse_nfrs};
use crate::core::{discover, parser::fr::ParseError, validation::semantic::SemanticError};
use crate::mcp::{next_steps, response};

use super::{MAX_FILE_SIZE, format_dep_constraint, mcp_error, read_file_checked, tool_error};

// ── Change detection ──────────────────────────────────

/// Compare baseline vs current behaviors to detect changes.
fn compute_spec_changes(cached: &graph::CachedEntry) -> Option<response::SpecChanges> {
    let baseline = cached.baseline.as_ref()?;
    let current = &cached.behaviors;

    if baseline == current {
        return None;
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = 0usize;

    // Find removed and modified
    for (name, baseline_snap) in baseline {
        match current.get(name) {
            None => {
                removed.push(response::BehaviorChange {
                    name: name.clone(),
                    category: baseline_snap.category.clone(),
                });
            }
            Some(current_snap) => {
                if baseline_snap.hash != current_snap.hash {
                    let mut sections = Vec::new();
                    sections.push("content".to_string());
                    if baseline_snap.category != current_snap.category {
                        sections.push("category".to_string());
                    }
                    modified.push(response::ModifiedBehavior {
                        name: name.clone(),
                        sections,
                    });
                } else {
                    unchanged += 1;
                }
            }
        }
    }

    // Find added
    for (name, current_snap) in current {
        if !baseline.contains_key(name) {
            added.push(response::BehaviorChange {
                name: name.clone(),
                category: current_snap.category.clone(),
            });
        }
    }

    if added.is_empty() && removed.is_empty() && modified.is_empty() {
        return None;
    }

    let version_change = cached
        .baseline_version
        .as_ref()
        .filter(|old_v| old_v.as_str() != cached.version)
        .map(|old_v| response::VersionChange {
            from: old_v.clone(),
            to: cached.version.clone(),
        });

    Some(response::SpecChanges {
        version_change,
        added,
        removed,
        modified,
        unchanged,
    })
}

/// Update a single spec in the cache, detect changes, and acknowledge baseline.
/// Returns `(cache_was_updated, detected_changes)`.
fn upsert_spec_and_detect_changes(
    cache: &mut graph::GraphCache,
    spec: &crate::model::Spec,
    source: &str,
    path_str: &str,
    is_valid: bool,
) -> (bool, Option<response::SpecChanges>) {
    let content_hash = graph::content_hash(source);
    if !cache.is_changed(&spec.name, &content_hash) {
        return (false, None);
    }

    let behaviors = graph::compute_behaviors(spec);
    let existing_baseline = cache.specs.get(&spec.name).and_then(|e| e.baseline.clone());
    let existing_baseline_version = cache.resolve_baseline_version(&spec.name);

    cache.upsert(
        spec.name.clone(),
        graph::CachedEntry {
            content_hash,
            version: spec.version.clone(),
            behavior_count: spec.behaviors.len(),
            valid: is_valid,
            dependencies: spec.dep_names(),
            path: path_str.to_string(),
            nfr_categories: spec.all_nfr_categories(),
            behaviors,
            baseline: existing_baseline,
            baseline_version: existing_baseline_version,
        },
    );

    let changes = cache.specs.get(&spec.name).and_then(compute_spec_changes);
    // Acknowledge baseline after reporting changes. This is intentionally
    // one-shot: the agent sees changes once, acts on them, and the next
    // validate starts from a fresh baseline.
    cache.acknowledge_baseline(&spec.name);
    (true, changes)
}

/// Update the graph cache for a single validated spec and return changes if any.
fn update_graph_for_spec(
    v: &validate_core::SpecValidation,
    source: &str,
    path_str: &str,
) -> Option<HashMap<String, response::SpecChanges>> {
    let spec = v.spec.as_ref()?;
    if !v.semantic_errors.is_empty() {
        return None;
    }

    let mut graph_state = graph::GraphState::load_or_build();
    let (updated, changes) =
        upsert_spec_and_detect_changes(&mut graph_state.cache, spec, source, path_str, v.is_valid);
    if updated {
        graph_state.dirty = true;
        graph_state.save_if_dirty();
    }

    changes.map(|c| HashMap::from([(spec.name.clone(), c)]))
}

// ── Result builders ────────────────────────────────────

/// Build a `ValidateResult` for a parse/semantic failure.
fn make_fail_result(
    file: Option<String>,
    result_type: &str,
    errors: Vec<response::ValidationError>,
) -> response::ValidateResult {
    response::ValidateResult {
        file,
        name: String::new(),
        version: String::new(),
        result_type: result_type.to_string(),
        status: "fail".to_string(),
        behavior_count: None,
        constraint_count: None,
        errors,
        dependencies: None,
    }
}

/// Builder for `ValidateResult` with sensible defaults for parsed specs/NFRs.
struct ResultBuilder {
    inner: response::ValidateResult,
}

impl ResultBuilder {
    /// Start building a result for a parsed spec or NFR.
    fn new(file: Option<String>, name: String, version: String, result_type: &str) -> Self {
        Self {
            inner: response::ValidateResult {
                file,
                name,
                version,
                result_type: result_type.to_string(),
                status: "pass".to_string(),
                behavior_count: None,
                constraint_count: None,
                errors: vec![],
                dependencies: None,
            },
        }
    }

    fn status(mut self, status: &str) -> Self {
        self.inner.status = status.to_string();
        self
    }

    fn behavior_count(mut self, count: usize) -> Self {
        self.inner.behavior_count = Some(count);
        self
    }

    fn constraint_count(mut self, count: usize) -> Self {
        self.inner.constraint_count = Some(count);
        self
    }

    fn errors(mut self, errors: Vec<response::ValidationError>) -> Self {
        self.inner.errors = errors;
        self
    }

    fn dependencies(mut self, deps: Vec<response::DependencyRef>) -> Self {
        self.inner.dependencies = Some(deps);
        self
    }

    fn build(self) -> response::ValidateResult {
        self.inner
    }
}

/// Build a ValidateResponse from a list of results and serialize to a CallToolResult.
fn results_to_response(
    results: Vec<response::ValidateResult>,
    changes: Option<HashMap<String, response::SpecChanges>>,
) -> Result<CallToolResult, ErrorData> {
    let passed = results.iter().filter(|r| r.status == "pass").count();
    let failed = results.iter().filter(|r| r.status == "fail").count();
    let any_fail = failed > 0;
    let has_changes = changes.as_ref().is_some_and(|c| !c.is_empty());
    let resp = response::ValidateResponse {
        results,
        summary: response::ValidateSummary {
            total: passed + failed,
            passed,
            failed,
        },
        changes: if has_changes { changes } else { None },
        next_steps: if any_fail {
            next_steps::after_validate_fail()
        } else if has_changes {
            next_steps::after_validate_pass_with_changes()
        } else {
            next_steps::after_validate_pass()
        },
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wrap a single result into a full `ValidateResponse` JSON string as a `CallToolResult`.
fn single_result_response(
    result: response::ValidateResult,
    changes: Option<HashMap<String, response::SpecChanges>>,
) -> Result<CallToolResult, ErrorData> {
    let is_pass = result.status == "pass";
    let has_changes = changes.as_ref().is_some_and(|c| !c.is_empty());
    let resp = response::ValidateResponse {
        results: vec![result],
        summary: response::ValidateSummary {
            total: 1,
            passed: if is_pass { 1 } else { 0 },
            failed: if is_pass { 0 } else { 1 },
        },
        changes: if has_changes { changes } else { None },
        next_steps: if is_pass {
            if has_changes {
                next_steps::after_validate_pass_with_changes()
            } else {
                next_steps::after_validate_pass()
            }
        } else {
            next_steps::after_validate_fail()
        },
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ── Fix suggestion helper ─────────────────────────────

fn suggest_fix(error_message: &str) -> Option<String> {
    if error_message.contains("Expected 'motivation'") {
        Some(
            "Add a 'motivation' section after 'description' explaining why this spec exists"
                .to_string(),
        )
    } else if error_message.contains("Expected 'description'") {
        Some("Add a 'description' section after 'title' explaining what this spec does".to_string())
    } else if error_message.contains("contains space")
        || error_message.contains("not kebab-case")
        || error_message.contains("kebab-case")
    {
        Some(
            "Use kebab-case for names: lowercase letters separated by hyphens (e.g., 'my-behavior')"
                .to_string(),
        )
    } else if error_message.contains("Expected 'behavior'") {
        Some(
            "Add at least one behavior with a category tag: behavior my-behavior [happy_path]"
                .to_string(),
        )
    } else if error_message.contains("Expected 'given'") {
        Some("Add a 'given' section with preconditions for this behavior".to_string())
    } else if error_message.contains("Expected 'when'") {
        Some("Add a 'when' section describing the action that triggers this behavior".to_string())
    } else if error_message.contains("Expected 'then'") {
        Some("Add a 'then' section with postconditions and assertions".to_string())
    } else if error_message.contains("tab") || error_message.contains("Tab") {
        Some("Use 2 spaces for indentation, not tabs".to_string())
    } else if error_message.contains("trailing content")
        || error_message.contains("Unexpected content")
    {
        Some(
            "Remove the extra content after the last valid section (depends on or behavior)"
                .to_string(),
        )
    } else if error_message.contains("version") {
        Some("Use semantic versioning format: v1.0.0".to_string())
    } else {
        None
    }
}

// ── Error converters ──────────────────────────────────

/// Convert parse errors into `ValidationError` vec.
fn parse_errors_to_validation(
    errors: &[ParseError],
    file: Option<String>,
) -> Vec<response::ValidationError> {
    errors
        .iter()
        .map(|e| response::ValidationError {
            file: file.clone(),
            line: e.line,
            message: e.message.clone(),
            fix: suggest_fix(&e.message),
        })
        .collect()
}

/// Convert semantic errors into `ValidationError` vec.
fn semantic_errors_to_validation(
    errors: &[SemanticError],
    file: Option<String>,
) -> Vec<response::ValidationError> {
    errors
        .iter()
        .map(|e| {
            let msg = e.to_string();
            let fix = suggest_fix(&msg);
            response::ValidationError {
                file: file.clone(),
                line: 0,
                message: msg,
                fix,
            }
        })
        .collect()
}

/// Convert ToString errors into `ValidationError` vec.
fn errors_to_validation(
    errors: &[impl ToString],
    file: Option<String>,
) -> Vec<response::ValidationError> {
    errors
        .iter()
        .map(|e| {
            let msg = e.to_string();
            let fix = suggest_fix(&msg);
            response::ValidationError {
                file: file.clone(),
                line: 0,
                message: msg,
                fix,
            }
        })
        .collect()
}

// ── Validate helpers ───────────────────────────────────

pub(super) fn validate_inline(
    content_str: &str,
    content_type: Option<&str>,
) -> Result<CallToolResult, ErrorData> {
    if content_str.len() > MAX_FILE_SIZE as usize {
        return Ok(tool_error(
            "Inline content exceeds maximum size of 10MB".to_string(),
        ));
    }

    let ct = content_type.unwrap_or("spec");
    match ct {
        "spec" => {
            let v = validate_core::validate_spec(content_str, None, None, None);
            single_result_response(spec_validation_to_result(v, None, false), None)
        }
        "nfr" => {
            let v = validate_core::validate_nfr(content_str);
            single_result_response(nfr_validation_to_result(v, None), None)
        }
        other => Ok(tool_error(format!(
            "Unknown content_type '{}'. Valid types: spec, nfr",
            other
        ))),
    }
}

pub(super) fn validate_file(
    path: &Path,
    path_str: &str,
    deep: bool,
) -> Result<CallToolResult, ErrorData> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "nfr" {
        return validate_nfr_file(path, path_str);
    }

    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let (siblings, nfr_specs_map) = if deep {
        let tree_root = path.parent().unwrap_or(Path::new("."));
        let siblings = discover::discover_specs(tree_root, Some(path));
        let nfr_discovery = discover_and_parse_nfrs(tree_root);
        (Some(siblings), Some(nfr_discovery.specs))
    } else {
        (None, None)
    };

    let v = validate_core::validate_spec(&source, None, siblings.as_ref(), nfr_specs_map.as_ref());

    if !deep {
        // Shallow mode: compute changes for this single spec if it passed
        let changes = update_graph_for_spec(&v, &source, path_str);
        return single_result_response(
            spec_validation_to_result(v, Some(path_str.to_string()), false),
            changes,
        );
    }

    // Deep mode: build multi-result response (main spec + resolved deps)
    let file = Some(path_str.to_string());

    // Spec didn't parse or has semantic errors — no deep resolution happened
    if v.spec.is_none() || !v.semantic_errors.is_empty() {
        return single_result_response(spec_validation_to_result(v, file, true), None);
    }

    let spec = v.spec.as_ref().unwrap();
    let mut errors = Vec::new();
    let mut status = "pass";

    if !v.crossref_errors.is_empty() {
        status = "fail";
        errors.extend(errors_to_validation(&v.crossref_errors, file.clone()));
    }
    if !v.dep_errors.is_empty() {
        status = "fail";
        for err in &v.dep_errors {
            let fix = suggest_fix(err);
            errors.push(response::ValidationError {
                file: file.clone(),
                line: 0,
                message: err.clone(),
                fix,
            });
        }
    }

    let mut results = vec![
        ResultBuilder::new(file, spec.name.clone(), spec.version.clone(), "spec")
            .status(status)
            .behavior_count(spec.behaviors.len())
            .errors(errors)
            .dependencies(
                spec.dependencies
                    .iter()
                    .map(format_dep_constraint)
                    .collect(),
            )
            .build(),
    ];

    // Append resolved dep entries
    let siblings_ref = siblings.as_ref();
    for (dep_name, rd) in &v.resolved_deps {
        let dep_path = siblings_ref
            .and_then(|s| s.get(dep_name))
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        results.push(
            ResultBuilder::new(
                Some(dep_path),
                dep_name.clone(),
                rd.spec.version.clone(),
                "spec",
            )
            .status(if rd.valid { "pass" } else { "fail" })
            .behavior_count(rd.spec.behaviors.len())
            .dependencies(
                rd.spec
                    .dependencies
                    .iter()
                    .map(format_dep_constraint)
                    .collect(),
            )
            .build(),
        );
    }

    // Compute changes for the main spec
    let changes = update_graph_for_spec(&v, &source, path_str);

    results_to_response(results, changes)
}

pub(super) fn validate_nfr_file(path: &Path, path_str: &str) -> Result<CallToolResult, ErrorData> {
    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let v = validate_core::validate_nfr(&source);
    single_result_response(
        nfr_validation_to_result(v, Some(path_str.to_string())),
        None,
    )
}

pub(super) fn validate_directory(dir: &Path, path_str: &str) -> Result<CallToolResult, ErrorData> {
    let files = match discover::discover_all_files(dir) {
        Ok(f) => f,
        Err(e) => return Ok(tool_error(e)),
    };

    if files.is_empty() {
        return Ok(tool_error(format!(
            "no .spec or .nfr files found in {}",
            path_str
        )));
    }

    let nfr_discovery = discover_and_parse_nfrs(dir);
    let nfr_specs_map = nfr_discovery.specs;

    let mut results = Vec::new();
    let mut graph_state = graph::GraphState::load_or_build();
    let mut all_changes: HashMap<String, response::SpecChanges> = HashMap::new();

    for file_path in &files {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_str = file_path.display().to_string();

        if ext == "nfr" {
            let source = match read_file_checked(file_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("Cannot read file: {}", e);
                    results.push(make_fail_result(
                        Some(file_str.clone()),
                        "nfr",
                        vec![response::ValidationError {
                            file: Some(file_str),
                            line: 0,
                            fix: suggest_fix(&msg),
                            message: msg,
                        }],
                    ));
                    continue;
                }
            };
            let v = validate_core::validate_nfr(&source);
            results.push(nfr_validation_to_result(v, Some(file_str)));
        } else {
            let source = match read_file_checked(file_path) {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("Cannot read file: {}", e);
                    results.push(make_fail_result(
                        Some(file_str.clone()),
                        "spec",
                        vec![response::ValidationError {
                            file: Some(file_str),
                            line: 0,
                            fix: suggest_fix(&msg),
                            message: msg,
                        }],
                    ));
                    continue;
                }
            };
            let siblings = discover::discover_specs(dir, Some(file_path));
            let v =
                validate_core::validate_spec(&source, None, Some(&siblings), Some(&nfr_specs_map));

            if let Some(spec) = &v.spec {
                if v.semantic_errors.is_empty() {
                    let (updated, changes) = upsert_spec_and_detect_changes(
                        &mut graph_state.cache,
                        spec,
                        &source,
                        &file_str,
                        v.is_valid,
                    );
                    if updated {
                        graph_state.dirty = true;
                    }
                    if let Some(c) = changes {
                        all_changes.insert(spec.name.clone(), c);
                    }
                }
            }

            results.push(spec_validation_to_result(v, Some(file_str), true));
        }
    }

    graph_state.save_if_dirty();

    let changes = if all_changes.is_empty() {
        None
    } else {
        Some(all_changes)
    };

    results_to_response(results, changes)
}

// ── Converters: validate_core result → MCP response ──

/// Convert a spec validation result into an MCP ValidateResult.
fn spec_validation_to_result(
    v: validate_core::SpecValidation,
    file: Option<String>,
    include_deps: bool,
) -> response::ValidateResult {
    if v.spec.is_none() {
        return make_fail_result(
            file.clone(),
            "spec",
            parse_errors_to_validation(&v.parse_errors, file),
        );
    }
    let spec = v.spec.as_ref().unwrap();

    let mut errors = Vec::new();
    let mut status = "pass";

    if !v.semantic_errors.is_empty() {
        status = "fail";
        errors.extend(semantic_errors_to_validation(
            &v.semantic_errors,
            file.clone(),
        ));
    }

    if !v.crossref_errors.is_empty() {
        status = "fail";
        errors.extend(errors_to_validation(&v.crossref_errors, file.clone()));
    }

    if !v.dep_errors.is_empty() {
        status = "fail";
        for err in &v.dep_errors {
            let fix = suggest_fix(err);
            errors.push(response::ValidationError {
                file: file.clone(),
                line: 0,
                message: err.clone(),
                fix,
            });
        }
    }

    let mut builder = ResultBuilder::new(file, spec.name.clone(), spec.version.clone(), "spec")
        .status(status)
        .behavior_count(spec.behaviors.len())
        .errors(errors);

    if include_deps {
        builder = builder.dependencies(
            spec.dependencies
                .iter()
                .map(format_dep_constraint)
                .collect(),
        );
    }

    builder.build()
}

/// Convert an NFR validation result into an MCP ValidateResult.
fn nfr_validation_to_result(
    v: validate_core::NfrValidation,
    file: Option<String>,
) -> response::ValidateResult {
    if v.nfr.is_none() {
        return make_fail_result(
            file.clone(),
            "nfr",
            parse_errors_to_validation(&v.parse_errors, file),
        );
    }
    let nfr = v.nfr.as_ref().unwrap();

    let (status, errors) = if !v.semantic_errors.is_empty() {
        (
            "fail",
            semantic_errors_to_validation(&v.semantic_errors, file.clone()),
        )
    } else {
        ("pass", vec![])
    };

    ResultBuilder::new(file, nfr.category.clone(), nfr.version.clone(), "nfr")
        .status(status)
        .constraint_count(nfr.constraints.len())
        .errors(errors)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::{BehaviorSnapshot, CachedEntry};

    fn make_cached_entry() -> CachedEntry {
        CachedEntry {
            content_hash: "abc".to_string(),
            version: "1.0.0".to_string(),
            behavior_count: 0,
            valid: true,
            dependencies: vec![],
            path: "test.spec".to_string(),
            nfr_categories: vec![],
            behaviors: HashMap::new(),
            baseline: None,
            baseline_version: None,
        }
    }

    fn snap(category: &str, hash: &str) -> BehaviorSnapshot {
        BehaviorSnapshot {
            category: category.to_string(),
            hash: hash.to_string(),
        }
    }

    #[test]
    /// validate-changes: no_baseline_returns_none
    fn no_baseline_returns_none() {
        let entry = make_cached_entry();
        assert!(entry.baseline.is_none());
        let result = compute_spec_changes(&entry);
        assert!(result.is_none());
    }

    #[test]
    /// validate-changes: identical_baseline_and_behaviors_returns_none
    fn identical_baseline_and_behaviors_returns_none() {
        let mut entry = make_cached_entry();
        let mut behaviors = HashMap::new();
        behaviors.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.behaviors = behaviors.clone();
        entry.baseline = Some(behaviors);

        let result = compute_spec_changes(&entry);
        assert!(result.is_none());
    }

    #[test]
    /// validate-changes: detects_removed_behavior
    fn detects_removed_behavior() {
        let mut entry = make_cached_entry();

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        baseline.insert("login-failure".to_string(), snap("error_case", "hash2"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert_eq!(changes.removed.len(), 1);
        assert_eq!(changes.removed[0].name, "login-failure");
        assert_eq!(changes.removed[0].category, "error_case");
        assert!(changes.added.is_empty());
        assert!(changes.modified.is_empty());
        assert_eq!(changes.unchanged, 1);
    }

    #[test]
    /// validate-changes: detects_added_behavior
    fn detects_added_behavior() {
        let mut entry = make_cached_entry();

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash1"));
        current.insert("login-failure".to_string(), snap("error_case", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.added[0].name, "login-failure");
        assert_eq!(changes.added[0].category, "error_case");
        assert!(changes.removed.is_empty());
        assert!(changes.modified.is_empty());
        assert_eq!(changes.unchanged, 1);
    }

    #[test]
    /// validate-changes: detects_modified_behavior_hash_changed
    fn detects_modified_behavior_hash_changed() {
        let mut entry = make_cached_entry();

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.modified[0].name, "login-success");
        assert!(
            changes.modified[0]
                .sections
                .contains(&"content".to_string())
        );
        assert!(changes.added.is_empty());
        assert!(changes.removed.is_empty());
        assert_eq!(changes.unchanged, 0);
    }

    #[test]
    /// validate-changes: detects_modified_behavior_category_changed
    fn detects_modified_behavior_category_changed() {
        let mut entry = make_cached_entry();

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("edge_case", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.modified[0].name, "login-success");
        // content is always reported when the hash differs
        assert!(
            changes.modified[0]
                .sections
                .contains(&"content".to_string())
        );
        // category is reported additionally when the category changed
        assert!(
            changes.modified[0]
                .sections
                .contains(&"category".to_string())
        );
    }

    #[test]
    /// validate-changes: mixed_changes_add_remove_modify
    fn mixed_changes_add_remove_modify() {
        let mut entry = make_cached_entry();

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        baseline.insert("login-failure".to_string(), snap("error_case", "hash2"));
        baseline.insert("empty-email".to_string(), snap("edge_case", "hash3"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        // login-success: unchanged
        current.insert("login-success".to_string(), snap("happy_path", "hash1"));
        // login-failure: removed (not in current)
        // empty-email: modified (different hash)
        current.insert(
            "empty-email".to_string(),
            snap("edge_case", "hash3-modified"),
        );
        // new-behavior: added
        current.insert("new-behavior".to_string(), snap("happy_path", "hash4"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.removed.len(), 1);
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.unchanged, 1);

        assert_eq!(changes.added[0].name, "new-behavior");
        assert_eq!(changes.removed[0].name, "login-failure");
        assert_eq!(changes.modified[0].name, "empty-email");
    }

    #[test]
    /// validate-changes: version_change_is_populated_when_version_changes
    fn version_change_is_populated_when_version_changes() {
        let mut entry = make_cached_entry();
        entry.version = "2.0.0".to_string();
        entry.baseline_version = Some("1.0.0".to_string());

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        let changes = result.unwrap();
        assert!(changes.version_change.is_some());
        let vc = changes.version_change.unwrap();
        assert_eq!(vc.from, "1.0.0");
        assert_eq!(vc.to, "2.0.0");
    }

    #[test]
    /// validate-changes: version_change_is_none_when_version_unchanged
    fn version_change_is_none_when_version_unchanged() {
        let mut entry = make_cached_entry();
        // version stays at "1.0.0", baseline_version also "1.0.0"
        entry.baseline_version = Some("1.0.0".to_string());

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        assert!(result.unwrap().version_change.is_none());
    }

    #[test]
    /// validate-changes: version_change_is_none_without_baseline_version
    fn version_change_is_none_without_baseline_version() {
        let mut entry = make_cached_entry();
        // baseline_version is None (no version tracking yet)

        let mut baseline = HashMap::new();
        baseline.insert("login-success".to_string(), snap("happy_path", "hash1"));
        entry.baseline = Some(baseline);

        let mut current = HashMap::new();
        current.insert("login-success".to_string(), snap("happy_path", "hash2"));
        entry.behaviors = current;

        let result = compute_spec_changes(&entry);
        assert!(result.is_some());
        assert!(result.unwrap().version_change.is_none());
    }
}
