use std::collections::HashMap;

use rmcp::model::*;

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
    let by_name: HashMap<&str, usize> = spec_nodes
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

    // Build reverse dependency map
    let mut reverse_deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, _, _, deps) in spec_nodes {
        for dep in deps {
            reverse_deps
                .entry(dep.spec_name.as_str())
                .or_default()
                .push(name.as_str());
        }
    }

    // BFS to find all transitive reverse dependencies
    let mut impacted: Vec<String> = Vec::new();
    let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut queue: Vec<&str> = vec![target];
    visited.insert(target);

    while let Some(current) = queue.pop() {
        if let Some(dependents) = reverse_deps.get(current) {
            for &dep in dependents {
                if visited.insert(dep) {
                    impacted.push(dep.to_string());
                    queue.push(dep);
                }
            }
        }
    }

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
