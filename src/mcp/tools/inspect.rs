use std::path::Path;

use rmcp::model::*;

use crate::core::commands::inspect_core;
use crate::core::parser;
use crate::core::parser::nfr as nfr_parser;
use crate::core::validation::{nfr_semantic, semantic};
use crate::mcp::{next_steps, response};

use super::{mcp_error, read_file_checked, tool_error};

// ── Inspect helpers ────────────────────────────────────

pub(super) fn inspect_spec_file(path: &Path) -> Result<CallToolResult, ErrorData> {
    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            let msg = errors
                .iter()
                .map(|e| format!("line {}: {}", e.line, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Ok(tool_error(msg));
        }
    };

    if let Err(errors) = semantic::validate(&spec) {
        let msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Ok(tool_error(msg));
    }

    spec_metadata_to_response(&spec)
}

pub(super) fn inspect_nfr_file(path: &Path) -> Result<CallToolResult, ErrorData> {
    let source = match read_file_checked(path) {
        Ok(s) => s,
        Err(msg) => return Ok(tool_error(msg)),
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(n) => n,
        Err(errors) => {
            let msg = errors
                .iter()
                .map(|e| format!("line {}: {}", e.line, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Ok(tool_error(msg));
        }
    };

    if let Err(errors) = nfr_semantic::validate(&nfr) {
        let msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Ok(tool_error(msg));
    }

    nfr_metadata_to_response(&nfr)
}

pub(super) fn inspect_inline(
    content_str: &str,
    content_type: Option<&str>,
) -> Result<CallToolResult, ErrorData> {
    let ct = content_type.unwrap_or("spec");
    match ct {
        "spec" => {
            let spec = match parser::parse(content_str) {
                Ok(s) => s,
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| format!("line {}: {}", e.line, e.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(tool_error(msg));
                }
            };

            if let Err(errors) = semantic::validate(&spec) {
                let msg = errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(tool_error(msg));
            }

            spec_metadata_to_response(&spec)
        }
        "nfr" => {
            let nfr = match nfr_parser::parse_nfr(content_str) {
                Ok(n) => n,
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| format!("line {}: {}", e.line, e.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(tool_error(msg));
                }
            };

            // No semantic validation for inline NFR (preserving existing behavior)
            nfr_metadata_to_response(&nfr)
        }
        other => Ok(tool_error(format!(
            "Unknown content_type '{}'. Valid types: spec, nfr",
            other
        ))),
    }
}

/// Convert shared spec metadata to MCP JSON response.
fn spec_metadata_to_response(spec: &crate::model::Spec) -> Result<CallToolResult, ErrorData> {
    let metadata = inspect_core::inspect_spec(spec);

    let categories = metadata.categories.into_iter().collect();
    let assertion_type_list = metadata
        .assertion_types
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let deps_list = metadata
        .dependencies
        .iter()
        .map(|(name, vc)| response::DependencyRef {
            name: name.clone(),
            constraint: format!(">= {}", vc),
        })
        .collect();

    let resp = response::InspectSpecResponse {
        name: metadata.name,
        result_type: "spec".to_string(),
        version: metadata.version,
        title: metadata.title,
        behavior_count: metadata.behavior_count,
        categories,
        dependencies: deps_list,
        assertion_types: assertion_type_list,
        next_steps: next_steps::after_inspect(metadata.has_error_case),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Convert shared NFR metadata to MCP JSON response.
fn nfr_metadata_to_response(nfr: &crate::model::NfrSpec) -> Result<CallToolResult, ErrorData> {
    let metadata = inspect_core::inspect_nfr(nfr);

    let resp = response::InspectNfrResponse {
        name: metadata.category.clone(),
        result_type: "nfr".to_string(),
        version: metadata.version,
        title: metadata.title,
        category: metadata.category,
        constraint_count: metadata.constraint_count,
        types: response::ConstraintTypeDist {
            metric: metadata.metric_count,
            rule: metadata.rule_count,
        },
        next_steps: next_steps::after_inspect(false),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}
