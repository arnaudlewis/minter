use std::collections::HashMap;
use std::path::Path;

use rmcp::model::*;

use crate::core::parser;
use crate::core::parser::nfr as nfr_parser;
use crate::core::validation::{nfr_semantic, semantic};
use crate::mcp::{next_steps, response};
use crate::model::{Assertion, BehaviorCategory, ConstraintType};

use super::{format_dep_constraint, mcp_error, read_file_checked, tool_error};

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

    let mut categories: HashMap<String, usize> = HashMap::new();
    let mut has_error_case = false;
    for b in &spec.behaviors {
        let label = match b.category {
            BehaviorCategory::HappyPath => "happy_path",
            BehaviorCategory::ErrorCase => {
                has_error_case = true;
                "error_case"
            }
            BehaviorCategory::EdgeCase => "edge_case",
        };
        *categories.entry(label.to_string()).or_insert(0) += 1;
    }

    let mut assertion_types: HashMap<String, usize> = HashMap::new();
    for b in &spec.behaviors {
        for post in &b.postconditions {
            for assertion in &post.assertions {
                let label = match assertion {
                    Assertion::Equals { .. } | Assertion::EqualsRef { .. } => "equals",
                    Assertion::IsPresent { .. } => "is_present",
                    Assertion::Contains { .. } => "contains",
                    Assertion::InRange { .. } => "in_range",
                    Assertion::MatchesPattern { .. } => "matches_pattern",
                    Assertion::GreaterOrEqual { .. } => "greater_or_equal",
                    Assertion::Prose(_) => "prose",
                };
                *assertion_types.entry(label.to_string()).or_insert(0) += 1;
            }
        }
    }

    let deps_list: Vec<response::DependencyRef> = spec
        .dependencies
        .iter()
        .map(format_dep_constraint)
        .collect();

    let assertion_type_list: Vec<String> = {
        let mut types: Vec<_> = assertion_types.keys().cloned().collect();
        types.sort();
        types
    };

    let resp = response::InspectSpecResponse {
        name: spec.name,
        result_type: "spec".to_string(),
        version: spec.version,
        title: spec.title,
        behavior_count: spec.behaviors.len(),
        categories,
        dependencies: deps_list,
        assertion_types: assertion_type_list,
        next_steps: next_steps::after_inspect(has_error_case),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
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

    let mut metric = 0;
    let mut rule = 0;
    for c in &nfr.constraints {
        match c.constraint_type {
            ConstraintType::Metric => metric += 1,
            ConstraintType::Rule => rule += 1,
        }
    }

    let resp = response::InspectNfrResponse {
        name: nfr.category.clone(),
        result_type: "nfr".to_string(),
        version: nfr.version,
        title: nfr.title,
        category: nfr.category,
        constraint_count: nfr.constraints.len(),
        types: response::ConstraintTypeDist { metric, rule },
        next_steps: next_steps::after_inspect(false),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
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

            let mut categories: HashMap<String, usize> = HashMap::new();
            let mut has_error_case = false;
            for b in &spec.behaviors {
                let label = match b.category {
                    BehaviorCategory::HappyPath => "happy_path",
                    BehaviorCategory::ErrorCase => {
                        has_error_case = true;
                        "error_case"
                    }
                    BehaviorCategory::EdgeCase => "edge_case",
                };
                *categories.entry(label.to_string()).or_insert(0) += 1;
            }

            let mut assertion_types: HashMap<String, usize> = HashMap::new();
            for b in &spec.behaviors {
                for post in &b.postconditions {
                    for assertion in &post.assertions {
                        let label = match assertion {
                            Assertion::Equals { .. } | Assertion::EqualsRef { .. } => "equals",
                            Assertion::IsPresent { .. } => "is_present",
                            Assertion::Contains { .. } => "contains",
                            Assertion::InRange { .. } => "in_range",
                            Assertion::MatchesPattern { .. } => "matches_pattern",
                            Assertion::GreaterOrEqual { .. } => "greater_or_equal",
                            Assertion::Prose(_) => "prose",
                        };
                        *assertion_types.entry(label.to_string()).or_insert(0) += 1;
                    }
                }
            }

            let assertion_type_list: Vec<String> = {
                let mut types: Vec<_> = assertion_types.keys().cloned().collect();
                types.sort();
                types
            };

            let deps_list: Vec<response::DependencyRef> = spec
                .dependencies
                .iter()
                .map(format_dep_constraint)
                .collect();

            let resp = response::InspectSpecResponse {
                name: spec.name,
                result_type: "spec".to_string(),
                version: spec.version,
                title: spec.title,
                behavior_count: spec.behaviors.len(),
                categories,
                dependencies: deps_list,
                assertion_types: assertion_type_list,
                next_steps: next_steps::after_inspect(has_error_case),
            };
            let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
            Ok(CallToolResult::success(vec![Content::text(json)]))
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

            let mut metric = 0;
            let mut rule = 0;
            for c in &nfr.constraints {
                match c.constraint_type {
                    ConstraintType::Metric => metric += 1,
                    ConstraintType::Rule => rule += 1,
                }
            }

            let resp = response::InspectNfrResponse {
                name: nfr.category.clone(),
                result_type: "nfr".to_string(),
                version: nfr.version,
                title: nfr.title,
                category: nfr.category,
                constraint_count: nfr.constraints.len(),
                types: response::ConstraintTypeDist { metric, rule },
                next_steps: next_steps::after_inspect(false),
            };
            let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
        other => Ok(tool_error(format!(
            "Unknown content_type '{}'. Valid types: spec, nfr",
            other
        ))),
    }
}
