use std::collections::HashMap;
use std::path::Path;

use rmcp::model::*;

use crate::core::parser;
use crate::core::parser::nfr as nfr_parser;
use crate::core::validation::{crossref as nfr_crossref, nfr_semantic, semantic};
use crate::core::{deps, discover};
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

/// Convert parse errors into `ValidationError` vec.
fn parse_errors_to_validation(
    errors: &[crate::core::parser::fr::ParseError],
    file: Option<String>,
) -> Vec<response::ValidationError> {
    errors
        .iter()
        .map(|e| response::ValidationError {
            file: file.clone(),
            line: e.line,
            message: e.message.clone(),
        })
        .collect()
}

/// Convert semantic errors into `ValidationError` vec.
fn semantic_errors_to_validation(
    errors: &[impl ToString],
    file: Option<String>,
) -> Vec<response::ValidationError> {
    errors
        .iter()
        .map(|e| response::ValidationError {
            file: file.clone(),
            line: 0,
            message: e.to_string(),
        })
        .collect()
}

/// Discover and parse all NFR files in a directory into a category→spec map.
fn discover_nfr_map(dir: &Path) -> HashMap<String, crate::model::NfrSpec> {
    let nfr_files = discover::discover_nfr_files(dir);
    let mut map = HashMap::new();
    for nfr_path in &nfr_files {
        if let Ok(source) = read_file_checked(nfr_path)
            && let Ok(nfr) = nfr_parser::parse_nfr(&source)
        {
            map.insert(nfr.category.clone(), nfr);
        }
    }
    map
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
            let spec = match parser::parse(content_str) {
                Ok(s) => s,
                Err(errors) => {
                    let result =
                        make_fail_result(None, "spec", parse_errors_to_validation(&errors, None));
                    return single_result_response(result);
                }
            };

            let (status, errors) = match semantic::validate(&spec) {
                Ok(()) => ("pass", vec![]),
                Err(sem_errors) => ("fail", semantic_errors_to_validation(&sem_errors, None)),
            };

            let result = ResultBuilder::new(None, spec.name.clone(), spec.version.clone(), "spec")
                .status(status)
                .behavior_count(spec.behaviors.len())
                .errors(errors)
                .build();
            single_result_response(result)
        }
        "nfr" => {
            let nfr = match nfr_parser::parse_nfr(content_str) {
                Ok(n) => n,
                Err(errors) => {
                    let result =
                        make_fail_result(None, "nfr", parse_errors_to_validation(&errors, None));
                    return single_result_response(result);
                }
            };

            let (status, errors) = match nfr_semantic::validate(&nfr) {
                Ok(()) => ("pass", vec![]),
                Err(sem_errors) => ("fail", semantic_errors_to_validation(&sem_errors, None)),
            };

            let result = ResultBuilder::new(None, nfr.category.clone(), nfr.version.clone(), "nfr")
                .status(status)
                .constraint_count(nfr.constraints.len())
                .errors(errors)
                .build();
            single_result_response(result)
        }
        other => Ok(tool_error(format!(
            "Unknown content_type '{}'. Valid types: spec, nfr",
            other
        ))),
    }
}

/// Deep-mode resolution: resolve deps, cross-validate NFRs, append results.
fn resolve_deep_mode(
    path: &Path,
    path_str: &str,
    spec: &crate::model::Spec,
    results: &mut Vec<response::ValidateResult>,
) {
    let tree_root = path.parent().unwrap_or(Path::new("."));
    let siblings = discover::discover_specs(tree_root, Some(path));
    let mut res_ctx = deps::ResolutionContext {
        siblings,
        resolved: HashMap::new(),
        stack: vec![spec.name.clone()],
        errors: Vec::new(),
    };
    deps::resolve_and_collect(&spec.dependencies, &mut res_ctx, 0);

    // NFR cross-validation
    let nfr_specs_map = discover_nfr_map(tree_root);
    let has_nfr_refs =
        !spec.nfr_refs.is_empty() || spec.behaviors.iter().any(|b| !b.nfr_refs.is_empty());
    if has_nfr_refs
        && let Err(crossref_errors) = nfr_crossref::cross_validate(spec, &nfr_specs_map)
        && let Some(first) = results.first_mut()
    {
        first.status = "fail".to_string();
        for ce in &crossref_errors {
            first.errors.push(response::ValidationError {
                file: Some(path_str.to_string()),
                line: 0,
                message: ce.to_string(),
            });
        }
    }

    // Append resolved deps
    for (dep_name, rd) in &res_ctx.resolved {
        let dep_path = res_ctx
            .siblings
            .get(dep_name)
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

    // Append resolution errors
    if !res_ctx.errors.is_empty()
        && let Some(first) = results.first_mut()
    {
        first.status = "fail".to_string();
        for err_msg in &res_ctx.errors {
            first.errors.push(response::ValidationError {
                file: Some(path_str.to_string()),
                line: 0,
                message: err_msg.clone(),
            });
        }
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

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            let result = make_fail_result(
                Some(path_str.to_string()),
                "spec",
                parse_errors_to_validation(&errors, Some(path_str.to_string())),
            );
            return single_result_response(result);
        }
    };

    let (status, errors) = match semantic::validate(&spec) {
        Ok(()) => ("pass", vec![]),
        Err(sem_errors) => (
            "fail",
            semantic_errors_to_validation(&sem_errors, Some(path_str.to_string())),
        ),
    };

    let mut results = vec![];

    let is_pass = status == "pass";
    let mut builder = ResultBuilder::new(
        Some(path_str.to_string()),
        spec.name.clone(),
        spec.version.clone(),
        "spec",
    )
    .status(status)
    .behavior_count(spec.behaviors.len())
    .errors(errors);
    if deep {
        builder = builder.dependencies(
            spec.dependencies
                .iter()
                .map(format_dep_constraint)
                .collect(),
        );
    }
    results.push(builder.build());

    // Deep mode: resolve dependencies + cross-validate NFRs
    if deep && is_pass {
        resolve_deep_mode(path, path_str, &spec, &mut results);
    }

    results_to_response(results)
}

pub(super) fn validate_nfr_file(path: &Path, path_str: &str) -> Result<CallToolResult, ErrorData> {
    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(n) => n,
        Err(errors) => {
            let result = make_fail_result(
                Some(path_str.to_string()),
                "nfr",
                parse_errors_to_validation(&errors, Some(path_str.to_string())),
            );
            return single_result_response(result);
        }
    };

    let (status, errors) = match nfr_semantic::validate(&nfr) {
        Ok(()) => ("pass", vec![]),
        Err(sem_errors) => (
            "fail",
            semantic_errors_to_validation(&sem_errors, Some(path_str.to_string())),
        ),
    };

    let result = ResultBuilder::new(
        Some(path_str.to_string()),
        nfr.category.clone(),
        nfr.version.clone(),
        "nfr",
    )
    .status(status)
    .constraint_count(nfr.constraints.len())
    .errors(errors)
    .build();
    single_result_response(result)
}

/// Validate a single NFR file in directory context.
fn validate_nfr_entry(file_path: &Path, file_str: String) -> response::ValidateResult {
    let source = match read_file_checked(file_path) {
        Ok(s) => s,
        Err(e) => {
            return make_fail_result(
                Some(file_str.clone()),
                "nfr",
                vec![response::ValidationError {
                    file: Some(file_str),
                    line: 0,
                    message: format!("Cannot read file: {}", e),
                }],
            );
        }
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(n) => n,
        Err(errors) => {
            return make_fail_result(
                Some(file_str.clone()),
                "nfr",
                parse_errors_to_validation(&errors, Some(file_str)),
            );
        }
    };

    let (status, errors) = match nfr_semantic::validate(&nfr) {
        Ok(()) => ("pass", vec![]),
        Err(sem_errors) => (
            "fail",
            semantic_errors_to_validation(&sem_errors, Some(file_str.clone())),
        ),
    };

    ResultBuilder::new(Some(file_str), nfr.category, nfr.version, "nfr")
        .status(status)
        .constraint_count(nfr.constraints.len())
        .errors(errors)
        .build()
}

/// Validate a single spec file in directory context (always deep).
fn validate_spec_entry(
    file_path: &Path,
    file_str: String,
    dir: &Path,
    nfr_specs_map: &HashMap<String, crate::model::NfrSpec>,
) -> response::ValidateResult {
    let source = match read_file_checked(file_path) {
        Ok(s) => s,
        Err(e) => {
            return make_fail_result(
                Some(file_str.clone()),
                "spec",
                vec![response::ValidationError {
                    file: Some(file_str),
                    line: 0,
                    message: format!("Cannot read file: {}", e),
                }],
            );
        }
    };

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            return make_fail_result(
                Some(file_str.clone()),
                "spec",
                parse_errors_to_validation(&errors, Some(file_str)),
            );
        }
    };

    let (status, mut errors) = match semantic::validate(&spec) {
        Ok(()) => ("pass", vec![]),
        Err(sem_errors) => (
            "fail",
            semantic_errors_to_validation(&sem_errors, Some(file_str.clone())),
        ),
    };

    // Directory validation is always deep — cross-validate NFRs
    let mut final_status = status.to_string();
    let has_nfr_refs =
        !spec.nfr_refs.is_empty() || spec.behaviors.iter().any(|b| !b.nfr_refs.is_empty());
    if has_nfr_refs && let Err(crossref_errors) = nfr_crossref::cross_validate(&spec, nfr_specs_map)
    {
        final_status = "fail".to_string();
        for ce in &crossref_errors {
            errors.push(response::ValidationError {
                file: Some(file_str.clone()),
                line: 0,
                message: ce.to_string(),
            });
        }
    }

    // Resolve deps (always deep for directory)
    let siblings = discover::discover_specs(dir, Some(file_path));
    let mut res_ctx = deps::ResolutionContext {
        siblings,
        resolved: HashMap::new(),
        stack: vec![spec.name.clone()],
        errors: Vec::new(),
    };
    deps::resolve_and_collect(&spec.dependencies, &mut res_ctx, 0);
    if !res_ctx.errors.is_empty() {
        final_status = "fail".to_string();
        for err_msg in &res_ctx.errors {
            errors.push(response::ValidationError {
                file: Some(file_str.clone()),
                line: 0,
                message: err_msg.clone(),
            });
        }
    }

    let deps_list: Vec<response::DependencyRef> = spec
        .dependencies
        .iter()
        .map(format_dep_constraint)
        .collect();

    ResultBuilder::new(Some(file_str), spec.name, spec.version, "spec")
        .status(&final_status)
        .behavior_count(spec.behaviors.len())
        .errors(errors)
        .dependencies(deps_list)
        .build()
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

    let nfr_specs_map = discover_nfr_map(dir);

    let mut results = Vec::new();
    for file_path in &files {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_str = file_path.display().to_string();

        if ext == "nfr" {
            results.push(validate_nfr_entry(file_path, file_str));
        } else {
            results.push(validate_spec_entry(
                file_path,
                file_str,
                dir,
                &nfr_specs_map,
            ));
        }
    }

    results_to_response(results)
}
