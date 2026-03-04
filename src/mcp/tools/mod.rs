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
use crate::model::VALID_NFR_CATEGORIES;

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
pub struct GuideParams {
    #[schemars(
        description = "Topic: workflow, authoring, smells, nfr, context, methodology, or coverage"
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
        description = "Condensed reference on spec-driven development practices. Returns guidance on workflow phases, spec authoring, requirements smells, NFR design, context management, or coverage tagging."
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
        description = "Generate a blank spec skeleton. Returns a functional requirement (FR) or non-functional requirement (NFR) template."
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
        description = "Display the DSL grammar reference for writing specs. Returns the functional requirement (FR) or non-functional requirement (NFR) grammar."
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
        description = "Validate one or more .spec or .nfr files, a directory, or inline content. Returns structured results with pass/fail status and error details."
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
        description = "Display structured metadata for a spec or NFR file. Returns categories, dependencies, assertion types, and coverage analysis."
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
        description = "Query the dependency graph across specs in a directory. Returns all specs and edges, or reverse dependencies of a named spec."
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

fn format_next_steps(steps: &[&str]) -> String {
    if steps.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n\nNext steps:\n");
    for step in steps {
        out.push_str(&format!("- {}\n", step));
    }
    out
}
