use serde::Serialize;

// ── Validate response types ────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub results: Vec<ValidateResult>,
    pub summary: ValidateSummary,
    pub next_steps: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ValidateResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_count: Option<usize>,
    pub errors: Vec<ValidationError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<DependencyRef>>,
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DependencyRef {
    pub name: String,
    pub constraint: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

// ── Inspect response types ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct InspectSpecResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub version: String,
    pub title: String,
    pub behavior_count: usize,
    pub categories: std::collections::HashMap<String, usize>,
    pub dependencies: Vec<DependencyRef>,
    pub assertion_types: Vec<String>,
    pub next_steps: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct InspectNfrResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub version: String,
    pub title: String,
    pub category: String,
    pub constraint_count: usize,
    pub types: ConstraintTypeDist,
    pub next_steps: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ConstraintTypeDist {
    pub metric: usize,
    pub rule: usize,
}

// ── Graph response types ───────────────────────────────

#[derive(Debug, Serialize)]
pub struct GraphFullResponse {
    pub specs: Vec<GraphSpecEntry>,
    pub edges: Vec<GraphEdge>,
    pub next_steps: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct GraphSpecEntry {
    pub name: String,
    pub file: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub constraint: String,
}

#[derive(Debug, Serialize)]
pub struct GraphImpactedResponse {
    pub target: String,
    pub impacted: Vec<GraphSpecEntry>,
    pub next_steps: Vec<&'static str>,
}
