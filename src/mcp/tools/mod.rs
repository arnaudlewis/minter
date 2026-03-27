mod graph;
mod inspect;
mod validate;

use std::path::{Path, PathBuf};

use std::future::Future;

use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::*;
use rmcp::schemars;
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, tool, tool_router};
use serde::Deserialize;

use crate::core::content;
use crate::core::discover;
use crate::core::parser;
use crate::mcp::{next_steps, response};
use crate::model::{BehaviorCategory, VALID_NFR_CATEGORIES};

#[derive(Debug, Clone)]
pub struct MinterServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl Default for MinterServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MinterServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

// ── Tool parameter structs ─────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    #[schemars(description = "File or directory path to validate")]
    pub path: Option<String>,
    #[schemars(description = "Inline spec content to validate (instead of path)")]
    pub content: Option<String>,
    #[schemars(description = "Content type for inline validation: spec or nfr")]
    pub content_type: Option<String>,
    #[schemars(
        description = "Enable deep mode: resolve dependencies and cross-validate NFR references"
    )]
    pub deep: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InspectParams {
    #[schemars(description = "File path to inspect")]
    pub path: Option<String>,
    #[schemars(description = "Inline spec content to inspect (instead of path)")]
    pub content: Option<String>,
    #[schemars(description = "Content type for inline inspection: spec or nfr")]
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScaffoldParams {
    #[schemars(description = "Spec type: spec or nfr")]
    #[serde(rename = "type")]
    pub spec_type: String,
    #[schemars(description = "NFR category (required when type is nfr)")]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatParams {
    #[schemars(description = "Spec type: spec or nfr")]
    #[serde(rename = "type")]
    pub spec_type: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GraphParams {
    #[schemars(description = "Directory path containing spec files")]
    pub path: String,
    #[schemars(description = "Show reverse dependencies of a named spec")]
    pub impacted: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSpecsParams {
    #[schemars(
        description = "Directory path containing spec files (defaults to working directory)"
    )]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListNfrsParams {
    #[schemars(
        description = "Directory path containing NFR files (defaults to working directory)"
    )]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    #[schemars(
        description = "Search query to match against spec names, behavior names, and NFR constraint names"
    )]
    pub query: String,
    #[schemars(description = "Directory path to search in (defaults to working directory)")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AssessParams {
    #[schemars(description = "File path to assess")]
    pub path: Option<String>,
    #[schemars(description = "Inline spec content to assess (instead of path)")]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GuideParams {
    #[schemars(
        description = "Topic: workflow, authoring, smells, nfr, context, methodology, coverage, config, lock, ci, or web"
    )]
    pub topic: String,
}

// ── Helper: error result ───────────────────────────────

pub(super) fn tool_error(msg: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(msg)],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

pub(super) fn mcp_error(msg: impl Into<String>) -> ErrorData {
    ErrorData::new(ErrorCode::INTERNAL_ERROR, msg.into(), None)
}

// ── File-read security helper ──────────────────────────

pub(super) use crate::core::io::MAX_FILE_SIZE;

pub(super) fn read_file_checked(path: &Path) -> Result<String, String> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown");

    // Extension validation
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "spec" && ext != "nfr" {
        return Err(format!(
            "{} does not have a .spec or .nfr extension",
            filename
        ));
    }

    // Symlink check: reject symlinks for individual files
    let sym_meta = std::fs::symlink_metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!("permission denied reading {}", filename)
        } else {
            format!("Cannot read {}: {}", filename, e)
        }
    })?;
    if sym_meta.file_type().is_symlink() {
        return Err(format!(
            "{} is a symlink — reading symlinks is not allowed",
            filename
        ));
    }

    // File size check
    let metadata = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!("permission denied reading {}", filename)
        } else {
            format!("Cannot read {}: {}", filename, e)
        }
    })?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!("{} exceeds maximum file size of 10MB", filename));
    }

    // Read with PermissionDenied handling
    std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!("permission denied reading {}", filename)
        } else {
            format!("Cannot read {}: {}", filename, e)
        }
    })
}

/// Canonicalize an MCP tool path argument, preventing path traversal.
///
/// Resolves `..` components and verifies the result exists. Symlinks on
/// individual files are rejected before canonicalization; directory
/// arguments are exempt because walkdir uses `follow_links(false)`.
pub(super) fn sanitize_path(path: &Path) -> Result<PathBuf, String> {
    // Check symlink status on the original path before canonicalization.
    // Reject symlinks pointing to files; allow symlinks pointing to directories
    // since walkdir handles directory traversal with follow_links(false).
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            // Resolve to check whether the target is a file or directory
            if let Ok(target_meta) = std::fs::metadata(path) {
                if !target_meta.is_dir() {
                    let filename = path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("unknown");
                    return Err(format!(
                        "{} is a symlink — reading symlinks is not allowed",
                        filename
                    ));
                }
            } else {
                // Dangling symlink -- reject
                return Err(format!("Path not found: {}", path.display()));
            }
        }
    }

    std::fs::canonicalize(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Path not found: {}", path.display())
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!("permission denied accessing {}", path.display())
        } else {
            format!("Cannot resolve path {}: {}", path.display(), e)
        }
    })
}

/// Resolve an optional directory path parameter: sanitize if provided, fall back to cwd.
fn resolve_dir(path: Option<&str>) -> Result<PathBuf, String> {
    match path {
        Some(p) => sanitize_path(Path::new(p)),
        None => std::env::current_dir().map_err(|e| e.to_string()),
    }
}

pub(super) fn format_dep_constraint(dep: &crate::model::Dependency) -> response::DependencyRef {
    response::DependencyRef {
        name: dep.spec_name.clone(),
        constraint: format!(">= {}", dep.version_constraint),
    }
}

// ── Tool implementations ───────────────────────────────

#[tool_router]
impl MinterServer {
    #[tool(
        description = "Onboarding tool for spec-driven development. MUST be called before using any other minter tool. Returns the complete methodology, workflow phases, and tool summary."
    )]
    fn initialize_minter(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            content::initialize_minter().to_string(),
        )]))
    }

    #[tool(
        description = "Condensed reference on spec-driven development practices. Topics: workflow, authoring, smells, nfr, context, methodology, coverage, config, lock, ci, web. Call this whenever you need to understand a concept or best practice."
    )]
    fn guide(
        &self,
        Parameters(params): Parameters<GuideParams>,
    ) -> Result<CallToolResult, ErrorData> {
        use crate::core::commands::guide::run_guide_topic;
        match run_guide_topic(&params.topic) {
            Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
            Err(msg) => Ok(tool_error(msg)),
        }
    }

    #[tool(
        description = "Generate a blank spec skeleton to start authoring. After scaffolding, fill in behaviors for each user-observable outcome. Validate after each significant addition."
    )]
    fn scaffold(
        &self,
        Parameters(params): Parameters<ScaffoldParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match params.spec_type.as_str() {
            "spec" => {
                let text = content::fr_scaffold();
                let steps = next_steps::after_scaffold_fr();
                let mut body = text.to_string();
                body.push_str(&format_next_steps(&steps));
                Ok(CallToolResult::success(vec![Content::text(body)]))
            }
            "nfr" => {
                let category = match &params.category {
                    Some(c) => c.as_str(),
                    None => {
                        return Ok(tool_error(
                            "NFR scaffold requires a 'category' parameter".to_string(),
                        ));
                    }
                };
                if !VALID_NFR_CATEGORIES.contains(&category) {
                    return Ok(tool_error(format!(
                        "Unknown category '{}'. Valid categories: {}",
                        category,
                        VALID_NFR_CATEGORIES.join(", ")
                    )));
                }
                let text = content::nfr_scaffold(category);
                let steps = next_steps::after_scaffold_nfr();
                let mut body = text;
                body.push_str(&format_next_steps(&steps));
                Ok(CallToolResult::success(vec![Content::text(body)]))
            }
            other => Ok(tool_error(format!(
                "Unknown type '{}'. Valid types: spec, nfr",
                other
            ))),
        }
    }

    #[tool(
        description = "Display the DSL grammar reference. Call before writing your first spec to learn the syntax. The grammar is the source of truth for spec format."
    )]
    fn format(
        &self,
        Parameters(params): Parameters<FormatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match params.spec_type.as_str() {
            "spec" => {
                let text = content::fr_grammar();
                let steps = next_steps::after_format();
                let mut body = text.to_string();
                body.push_str(&format_next_steps(&steps));
                Ok(CallToolResult::success(vec![Content::text(body)]))
            }
            "nfr" => {
                let text = content::nfr_grammar();
                let steps = next_steps::after_format();
                let mut body = text.to_string();
                body.push_str(&format_next_steps(&steps));
                Ok(CallToolResult::success(vec![Content::text(body)]))
            }
            other => Ok(tool_error(format!(
                "Unknown type '{}'. Valid types: spec, nfr",
                other
            ))),
        }
    }

    #[tool(
        description = "Validate specs after writing or editing — call in a loop until all errors pass. If all pass, proceed to writing red tests (use guide topic 'coverage' for tag format). If any fail, fix errors using the suggestions in the response, then re-validate. Never write implementation code before validation passes."
    )]
    fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // Inline content validation
        if let Some(ref content_str) = params.content {
            if content_str.len() > MAX_FILE_SIZE as usize {
                return Ok(tool_error(
                    "Inline content exceeds maximum size of 10MB".to_string(),
                ));
            }
            return validate::validate_inline(content_str, params.content_type.as_deref());
        }

        let path_str = match &params.path {
            Some(p) => p.as_str(),
            None => {
                return Ok(tool_error(
                    "Either 'path' or 'content' parameter is required".to_string(),
                ));
            }
        };

        let raw_path = Path::new(path_str);
        let path = match sanitize_path(raw_path) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };

        let deep = params.deep.unwrap_or(false);

        if path.is_dir() {
            validate::validate_directory(&path, path_str)
        } else {
            validate::validate_file(&path, path_str, deep)
        }
    }

    #[tool(
        description = "Examine a spec's structure after validation passes. Use to verify behavior count, category balance, dependency completeness, and assertion types before moving to tests."
    )]
    fn inspect(
        &self,
        Parameters(params): Parameters<InspectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // Inline content inspection
        if let Some(ref content_str) = params.content {
            if content_str.len() > MAX_FILE_SIZE as usize {
                return Ok(tool_error(
                    "Inline content exceeds maximum size of 10MB".to_string(),
                ));
            }
            return inspect::inspect_inline(content_str, params.content_type.as_deref());
        }

        let path_str = match &params.path {
            Some(p) => p.as_str(),
            None => {
                return Ok(tool_error(
                    "Either 'path' or 'content' parameter is required".to_string(),
                ));
            }
        };

        let raw_path = Path::new(path_str);
        let path = match sanitize_path(raw_path) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "nfr" {
            inspect::inspect_nfr_file(&path)
        } else {
            inspect::inspect_spec_file(&path)
        }
    }

    #[tool(
        description = "Query the dependency graph to understand how specs relate. Use before adding 'depends on' to verify the dependency exists and check for cycles."
    )]
    fn graph(
        &self,
        Parameters(params): Parameters<GraphParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let raw_path = Path::new(&params.path);
        let path = match sanitize_path(raw_path) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };

        let spec_files = match discover::discover_spec_files(&path) {
            Ok(files) => files,
            Err(e) => return Ok(tool_error(e)),
        };

        if spec_files.is_empty() {
            return Ok(tool_error(format!(
                "no spec files found in {}",
                params.path
            )));
        }

        // Parse all specs
        let mut spec_nodes: Vec<(String, String, String, Vec<crate::model::Dependency>)> =
            Vec::new();
        for spec_path in &spec_files {
            let source = match read_file_checked(spec_path) {
                Ok(s) => s,
                Err(msg) => return Ok(tool_error(msg)),
            };
            let spec = parser::parse(&source)
                .map_err(|errors| mcp_error(format!("{}: {}", spec_path.display(), errors[0])))?;
            spec_nodes.push((
                spec.name,
                spec_path.display().to_string(),
                spec.version,
                spec.dependencies,
            ));
        }

        match params.impacted {
            Some(target) => graph::graph_impacted(&spec_nodes, &target),
            None => graph::graph_full(&spec_nodes),
        }
    }

    #[tool(
        description = "Browse all specs in the project. Use at the start of authoring to understand existing specs, find dependencies, and avoid naming conflicts."
    )]
    fn list_specs(
        &self,
        Parameters(params): Parameters<ListSpecsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        use crate::core::validation::semantic;

        let dir = match resolve_dir(params.path.as_deref()) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };

        let spec_files = match discover::discover_spec_files(&dir) {
            Ok(files) => files,
            Err(e) => return Ok(tool_error(e)),
        };

        let mut entries: Vec<response::SpecEntry> = Vec::new();
        for spec_path in &spec_files {
            let source = match read_file_checked(spec_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let spec = match parser::parse(&source) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let validation_status = if semantic::validate(&spec).is_ok() {
                "valid"
            } else {
                "invalid"
            }
            .to_string();

            let nfr_refs: Vec<String> = spec.all_nfr_categories();
            let dependencies: Vec<response::DependencyRef> = spec
                .dependencies
                .iter()
                .map(format_dep_constraint)
                .collect();

            entries.push(response::SpecEntry {
                name: spec.name,
                version: spec.version,
                path: spec_path.display().to_string(),
                behavior_count: spec.behaviors.len(),
                validation_status,
                nfr_refs,
                dependencies,
            });
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));

        let resp = response::ListSpecsResponse {
            specs: entries,
            next_steps: next_steps::after_list_specs(),
        };
        let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Browse all NFR categories and their constraints. Use when writing the 'nfr' section of a spec to reference existing constraints correctly."
    )]
    fn list_nfrs(
        &self,
        Parameters(params): Parameters<ListNfrsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        use crate::model::ConstraintBody;

        let dir = match resolve_dir(params.path.as_deref()) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };

        let nfr_files = discover::discover_nfr_files(&dir);

        let mut entries: Vec<response::NfrEntry> = Vec::new();
        for nfr_path in &nfr_files {
            let source = match read_file_checked(nfr_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let nfr = match parser::parse_nfr(&source) {
                Ok(n) => n,
                Err(_) => continue,
            };

            let constraints: Vec<response::NfrConstraintEntry> = nfr
                .constraints
                .iter()
                .map(|c| {
                    let (type_str, threshold) = match &c.body {
                        ConstraintBody::Metric {
                            threshold_operator,
                            threshold_value,
                            ..
                        } => (
                            "metric",
                            Some(format!("{} {}", threshold_operator, threshold_value)),
                        ),
                        ConstraintBody::Rule { .. } => ("rule", None),
                    };
                    response::NfrConstraintEntry {
                        name: c.name.clone(),
                        constraint_type: type_str.to_string(),
                        description: c.description.clone(),
                        threshold,
                        overridable: c.overridable,
                    }
                })
                .collect();

            entries.push(response::NfrEntry {
                category: nfr.category.clone(),
                version: nfr.version.clone(),
                path: nfr_path.display().to_string(),
                constraint_count: nfr.constraints.len(),
                constraints,
            });
        }

        entries.sort_by(|a, b| a.category.cmp(&b.category));

        let resp = response::ListNfrsResponse {
            nfrs: entries,
            next_steps: next_steps::after_list_nfrs(),
        };
        let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Search for specs, behaviors, and NFR constraints by keyword. Use to find existing behaviors before creating new ones, and to discover dependencies."
    )]
    fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let dir = match resolve_dir(params.path.as_deref()) {
            Ok(p) => p,
            Err(msg) => return Ok(tool_error(msg)),
        };
        let query_lower = params.query.to_lowercase();

        let mut results: Vec<response::SearchResult> = Vec::new();

        // Search specs and behaviors
        if let Ok(spec_files) = discover::discover_spec_files(&dir) {
            for spec_path in &spec_files {
                let source = match read_file_checked(spec_path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let spec = match parser::parse(&source) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let path_str = spec_path.display().to_string();

                // Match spec name
                if spec.name.to_lowercase().contains(&query_lower) {
                    results.push(response::SearchResult {
                        result_type: "spec".to_string(),
                        name: spec.name.clone(),
                        spec_name: None,
                        category: None,
                        path: path_str.clone(),
                    });
                }

                // Match behavior names
                for behavior in &spec.behaviors {
                    if behavior.name.to_lowercase().contains(&query_lower) {
                        results.push(response::SearchResult {
                            result_type: "behavior".to_string(),
                            name: behavior.name.clone(),
                            spec_name: Some(spec.name.clone()),
                            category: None,
                            path: path_str.clone(),
                        });
                    }
                }
            }
        }

        // Search NFR constraints
        let nfr_files = discover::discover_nfr_files(&dir);
        for nfr_path in &nfr_files {
            let source = match read_file_checked(nfr_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let nfr = match parser::parse_nfr(&source) {
                Ok(n) => n,
                Err(_) => continue,
            };

            let path_str = nfr_path.display().to_string();

            for constraint in &nfr.constraints {
                if constraint.name.to_lowercase().contains(&query_lower) {
                    results.push(response::SearchResult {
                        result_type: "nfr_constraint".to_string(),
                        name: constraint.name.clone(),
                        spec_name: None,
                        category: Some(nfr.category.clone()),
                        path: path_str.clone(),
                    });
                }
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));

        let resp = response::SearchResponse {
            results,
            next_steps: next_steps::after_search(),
        };
        let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Analyze a spec for quality issues: coverage balance (happy/error/edge), requirement smells (implementation leaks, vague descriptions), missing error cases, and NFR gaps. Call after validate passes to improve spec quality before writing tests."
    )]
    fn assess(
        &self,
        Parameters(params): Parameters<AssessParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // Parse spec from inline content or file path
        let spec = if let Some(ref content_str) = params.content {
            if content_str.len() > MAX_FILE_SIZE as usize {
                return Ok(tool_error(
                    "Inline content exceeds maximum size of 10MB".to_string(),
                ));
            }
            match parser::parse(content_str) {
                Ok(s) => s,
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| format!("line {}: {}", e.line, e.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(tool_error(msg));
                }
            }
        } else if let Some(ref path_str) = params.path {
            let raw_path = Path::new(path_str);
            let path = match sanitize_path(raw_path) {
                Ok(p) => p,
                Err(msg) => return Ok(tool_error(msg)),
            };
            let source = match read_file_checked(&path) {
                Ok(s) => s,
                Err(msg) => return Ok(tool_error(msg)),
            };
            match parser::parse(&source) {
                Ok(s) => s,
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| format!("line {}: {}", e.line, e.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(tool_error(msg));
                }
            }
        } else {
            return Ok(tool_error(
                "Either 'path' or 'content' parameter is required".to_string(),
            ));
        };

        // Analyze coverage balance
        let mut happy = 0usize;
        let mut error = 0usize;
        let mut edge = 0usize;
        for b in &spec.behaviors {
            match b.category {
                BehaviorCategory::HappyPath => happy += 1,
                BehaviorCategory::ErrorCase => error += 1,
                BehaviorCategory::EdgeCase => edge += 1,
            }
        }

        let assessment = if error == 0 && happy > 0 {
            "missing_error_cases".to_string()
        } else if edge == 0 && happy > 2 {
            "missing_edge_cases".to_string()
        } else {
            "balanced".to_string()
        };

        let smells = detect_smells(&spec);
        let missing = suggest_missing_behaviors(&spec);
        let nfr_gaps = detect_nfr_gaps(&spec);

        let resp = response::AssessResponse {
            coverage_balance: response::CoverageBalance {
                happy_path: happy,
                error_case: error,
                edge_case: edge,
                assessment,
            },
            smells,
            missing,
            nfr_gaps,
            next_steps: next_steps::after_assess(),
        };
        let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl ServerHandler for MinterServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "minter".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Spec-driven development server. Specs are the source of truth. \
                 Workflow: spec → validate → red tests → implement → all green. \
                 Call initialize_minter before using any other tool."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        let tools = self.tool_router.list_all();
        std::future::ready(Ok(ListToolsResult {
            tools,
            ..Default::default()
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let ctx = ToolCallContext::new(self, request, context);
        async { self.tool_router.call(ctx).await }
    }
}

// ── Format next_steps as text suffix ───────────────────

fn format_next_steps(steps: &[response::NextStep]) -> String {
    if steps.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n\nNext steps:\n");
    for step in steps {
        out.push_str(&format!("- {}\n", step.action));
    }
    out
}

// ── Assess analysis helpers ────────────────────────────

const IMPL_KEYWORDS: &[&str] = &[
    "sqs",
    "lambda",
    "queue",
    "kafka",
    "redis",
    "postgres",
    "mongo",
    "dynamo",
    "s3",
    "api-gateway",
    "step-function",
    "docker",
    "kubernetes",
    "cron",
    "worker",
    "handler",
    "middleware",
    "controller",
    "repository",
    "dao",
    "database",
    "cache",
    "broker",
    "pipeline",
];

fn detect_smells(spec: &crate::model::Spec) -> Vec<response::SmellEntry> {
    let mut smells = Vec::new();

    for behavior in &spec.behaviors {
        let name_lower = behavior.name.to_lowercase();
        let desc_lower = behavior.description.to_lowercase();

        // Check for implementation keywords in name
        let mut name_matched = false;
        for keyword in IMPL_KEYWORDS {
            if name_lower.contains(keyword) {
                smells.push(response::SmellEntry {
                    behavior: behavior.name.clone(),
                    smell_type: "implementation_leak".to_string(),
                    detail: format!(
                        "Behavior name contains '{}' — an implementation detail",
                        keyword
                    ),
                    fix: "Describe the user-observable outcome, not the mechanism. Ask: what does the user/caller see?".to_string(),
                });
                name_matched = true;
                break;
            }
        }

        // Check for implementation keywords in description (only if name didn't match)
        if !name_matched {
            for keyword in IMPL_KEYWORDS {
                if desc_lower.contains(keyword) {
                    smells.push(response::SmellEntry {
                        behavior: behavior.name.clone(),
                        smell_type: "implementation_leak".to_string(),
                        detail: format!(
                            "Description mentions '{}' — an implementation detail",
                            keyword
                        ),
                        fix: "Rewrite to describe the observable outcome without naming specific technologies".to_string(),
                    });
                    break;
                }
            }
        }

        // Check for too many assertions (>8 total across postconditions)
        let assertion_count: usize = behavior
            .postconditions
            .iter()
            .map(|p| p.assertions.len())
            .sum();
        if assertion_count > 8 {
            smells.push(response::SmellEntry {
                behavior: behavior.name.clone(),
                smell_type: "too_coarse".to_string(),
                detail: format!(
                    "{} assertions — behavior may be too coarse",
                    assertion_count
                ),
                fix: "Split into multiple focused behaviors, each testing one outcome".to_string(),
            });
        }

        // Check for vague description (too short)
        if behavior.description.len() < 10 {
            smells.push(response::SmellEntry {
                behavior: behavior.name.clone(),
                smell_type: "vague_description".to_string(),
                detail:
                    "Description is very short — may not clearly explain what the behavior does"
                        .to_string(),
                fix: "Write a one-sentence description of the user-observable outcome".to_string(),
            });
        }
    }

    smells
}

fn suggest_missing_behaviors(spec: &crate::model::Spec) -> Vec<response::MissingEntry> {
    let mut missing = Vec::new();

    let has_error = spec
        .behaviors
        .iter()
        .any(|b| b.category == BehaviorCategory::ErrorCase);
    let has_edge = spec
        .behaviors
        .iter()
        .any(|b| b.category == BehaviorCategory::EdgeCase);
    let happy_count = spec
        .behaviors
        .iter()
        .filter(|b| b.category == BehaviorCategory::HappyPath)
        .count();

    if happy_count > 0 && !has_error {
        for b in spec
            .behaviors
            .iter()
            .filter(|b| b.category == BehaviorCategory::HappyPath)
        {
            missing.push(response::MissingEntry {
                missing_type: "error_case".to_string(),
                for_behavior: b.name.clone(),
                suggestion: format!(
                    "What happens when {} fails or receives invalid input?",
                    b.name
                ),
            });
        }
    }

    if happy_count > 2 && !has_edge {
        missing.push(response::MissingEntry {
            missing_type: "edge_case".to_string(),
            for_behavior: String::new(),
            suggestion: "Consider edge cases: boundary values, empty inputs, concurrent access"
                .to_string(),
        });
    }

    missing
}

fn detect_nfr_gaps(spec: &crate::model::Spec) -> Vec<response::NfrGapEntry> {
    let mut gaps = Vec::new();

    if spec.nfr_refs.is_empty() && spec.behaviors.len() >= 3 {
        gaps.push(response::NfrGapEntry {
            suggestion: format!(
                "Spec has {} behaviors but no NFR references. Consider adding performance, reliability, or operability constraints.",
                spec.behaviors.len()
            ),
        });
    }

    gaps
}
