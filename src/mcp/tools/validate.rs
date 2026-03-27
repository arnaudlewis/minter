use std::path::Path;

use rmcp::model::*;

use crate::core::commands::validate_core;
use crate::core::graph::discover_and_parse_nfrs;
use crate::core::{discover, parser::fr::ParseError, validation::semantic::SemanticError};
use crate::mcp::{next_steps, response};

use super::{MAX_FILE_SIZE, format_dep_constraint, mcp_error, read_file_checked, tool_error};

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
) -> Result<CallToolResult, ErrorData> {
    let passed = results.iter().filter(|r| r.status == "pass").count();
    let failed = results.iter().filter(|r| r.status == "fail").count();
    let any_fail = failed > 0;
    let resp = response::ValidateResponse {
        results,
        summary: response::ValidateSummary {
            total: passed + failed,
            passed,
            failed,
        },
        next_steps: if any_fail {
            next_steps::after_validate_fail()
        } else {
            next_steps::after_validate_pass()
        },
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wrap a single result into a full `ValidateResponse` JSON string as a `CallToolResult`.
fn single_result_response(result: response::ValidateResult) -> Result<CallToolResult, ErrorData> {
    let is_pass = result.status == "pass";
    let resp = response::ValidateResponse {
        results: vec![result],
        summary: response::ValidateSummary {
            total: 1,
            passed: if is_pass { 1 } else { 0 },
            failed: if is_pass { 0 } else { 1 },
        },
        next_steps: if is_pass {
            next_steps::after_validate_pass()
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
            single_result_response(spec_validation_to_result(v, None, false))
        }
        "nfr" => {
            let v = validate_core::validate_nfr(content_str);
            single_result_response(nfr_validation_to_result(v, None))
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
        return single_result_response(spec_validation_to_result(
            v,
            Some(path_str.to_string()),
            false,
        ));
    }

    // Deep mode: build multi-result response (main spec + resolved deps)
    let file = Some(path_str.to_string());

    // Spec didn't parse or has semantic errors — no deep resolution happened
    if v.spec.is_none() || !v.semantic_errors.is_empty() {
        return single_result_response(spec_validation_to_result(v, file, true));
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

    results_to_response(results)
}

pub(super) fn validate_nfr_file(path: &Path, path_str: &str) -> Result<CallToolResult, ErrorData> {
    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let v = validate_core::validate_nfr(&source);
    single_result_response(nfr_validation_to_result(v, Some(path_str.to_string())))
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
            results.push(spec_validation_to_result(v, Some(file_str), true));
        }
    }

    results_to_response(results)
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
