use rmcp::model::*;

use crate::core::commands::graph::find_spec_dependents;
use crate::mcp::{next_steps, response};

use super::{mcp_error, tool_error};

// ── Graph helpers ──────────────────────────────────────

pub(super) fn graph_full(
    spec_nodes: &[(String, String, String, Vec<crate::model::Dependency>)],
) -> Result<CallToolResult, ErrorData> {
    let mut specs = Vec::new();
    let mut edges = Vec::new();

    for (name, file, version, deps) in spec_nodes {
        specs.push(response::GraphSpecEntry {
            name: name.clone(),
            file: file.clone(),
            version: version.clone(),
        });
        for dep in deps {
            edges.push(response::GraphEdge {
                from: name.clone(),
                to: dep.spec_name.clone(),
                constraint: format!(">= {}", dep.version_constraint),
            });
        }
    }

    let resp = response::GraphFullResponse {
        specs,
        edges,
        next_steps: next_steps::after_graph(),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub(super) fn graph_impacted(
    spec_nodes: &[(String, String, String, Vec<crate::model::Dependency>)],
    target: &str,
) -> Result<CallToolResult, ErrorData> {
    let by_name: std::collections::HashMap<&str, usize> = spec_nodes
        .iter()
        .enumerate()
        .map(|(i, (name, ..))| (name.as_str(), i))
        .collect();

    if !by_name.contains_key(target) {
        return Ok(tool_error(format!(
            "spec '{}' not found in directory",
            target
        )));
    }

    let bfs_input: Vec<(String, Vec<String>)> = spec_nodes
        .iter()
        .map(|(name, _, _, deps)| {
            (
                name.clone(),
                deps.iter().map(|d| d.spec_name.clone()).collect(),
            )
        })
        .collect();
    let mut impacted = find_spec_dependents(&bfs_input, target);
    impacted.sort();

    let impacted_entries: Vec<response::GraphSpecEntry> = impacted
        .iter()
        .filter_map(|name| {
            by_name.get(name.as_str()).map(|&i| {
                let (n, f, v, _) = &spec_nodes[i];
                response::GraphSpecEntry {
                    name: n.clone(),
                    file: f.clone(),
                    version: v.clone(),
                }
            })
        })
        .collect();

    let resp = response::GraphImpactedResponse {
        target: target.to_string(),
        impacted: impacted_entries,
        next_steps: next_steps::after_graph(),
    };
    let json = serde_json::to_string(&resp).map_err(|e| mcp_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}
