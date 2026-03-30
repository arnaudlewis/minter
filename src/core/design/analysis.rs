use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::core::discover::discover_spec_files;
use crate::core::io::read_file_safe;
use crate::core::parser::fr;
use crate::model::{BehaviorCategory, PostconditionKind, Precondition, Spec};

// ── Public types ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub entities: Vec<String>,
    pub archetype: ArchetypeResult,
    pub spec_metadata: SpecMetadata,
    pub flow_count: usize,
    pub hierarchy_depth: usize,
    pub behavior_distribution: BehaviorDistribution,
    pub app_description: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArchetypeResult {
    pub name: String,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct SpecMetadata {
    pub project_name: String,
    pub domains: Vec<Domain>,
    pub spec_metrics: Vec<SpecMetric>,
    pub total_spec_count: usize,
    pub total_behavior_count: usize,
    pub total_entity_count: usize,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub name: String,
    pub spec_count: usize,
    pub spec_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SpecMetric {
    pub name: String,
    pub version: String,
    pub behavior_count: usize,
}

#[derive(Debug, Clone)]
pub struct BehaviorDistribution {
    pub happy_path: usize,
    pub error_case: usize,
    pub edge_case: usize,
}

// ── Main entry point ───────────────────────────────────────────

/// Analyze all `.spec` files in a directory tree.
///
/// Returns an [`AnalysisResult`] with extracted entities, archetype classification,
/// metadata, structural metrics, and warnings for unparseable files.
pub fn analyze_specs(dir: &Path) -> Result<AnalysisResult, String> {
    if !dir.exists() {
        return Err(format!("directory does not exist: {}", dir.display()));
    }

    let spec_files = discover_spec_files(dir)?;

    if spec_files.is_empty() {
        return Err("no spec files found in directory".to_string());
    }

    // Parse all specs, collecting warnings for failures
    let mut parsed_specs: Vec<(Spec, std::path::PathBuf)> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for path in &spec_files {
        let content = match read_file_safe(path) {
            Ok(c) => c,
            Err(e) => {
                warnings.push(format!("could not read {}: {}", path.display(), e));
                continue;
            }
        };
        match fr::parse(&content) {
            Ok(spec) => parsed_specs.push((spec, path.clone())),
            Err(errs) => {
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let first_err = errs.first().map(|e| e.to_string()).unwrap_or_default();
                warnings.push(format!("failed to parse {}: {}", file_name, first_err));
            }
        }
    }

    if parsed_specs.is_empty() {
        return Err("no spec files could be parsed".to_string());
    }

    // Extract entities
    let entities = extract_entities(&parsed_specs);

    // Compute metadata
    let spec_metadata = build_metadata(dir, &parsed_specs, entities.len());

    // Compute structural metrics
    let hierarchy_depth = compute_hierarchy_depth(dir, &spec_files);
    let flow_count = parsed_specs.iter().map(|(s, _)| s.behaviors.len()).sum();
    let behavior_distribution = compute_behavior_distribution(&parsed_specs);

    // Classify archetype
    let archetype = classify_archetype(&entities, hierarchy_depth, &parsed_specs);

    // Generate app description
    let app_description = generate_description(&entities, &archetype, &spec_metadata);

    Ok(AnalysisResult {
        entities,
        archetype,
        spec_metadata,
        flow_count,
        hierarchy_depth,
        behavior_distribution,
        app_description,
        warnings,
    })
}

// ── Entity extraction ──────────────────────────────────────────

fn extract_entities(specs: &[(Spec, std::path::PathBuf)]) -> Vec<String> {
    let mut entities = BTreeSet::new();

    for (spec, _) in specs {
        for behavior in &spec.behaviors {
            // Extract from alias preconditions
            for pre in &behavior.preconditions {
                if let Precondition::Alias { entity, .. } = pre {
                    entities.insert(entity.clone());
                }
            }
            // Extract from return postconditions
            for post in &behavior.postconditions {
                if let PostconditionKind::Returns(ref return_type) = post.kind {
                    entities.insert(return_type.clone());
                }
            }
        }
    }

    entities.into_iter().collect()
}

// ── Archetype classification ───────────────────────────────────

const ECOMMERCE_KEYWORDS: &[&str] = &[
    "product", "cart", "order", "payment", "checkout", "invoice", "receipt", "shipping",
];
const CONTENT_KEYWORDS: &[&str] = &[
    "article", "post", "author", "comment", "blog", "publish", "content", "tag",
];
const FORM_KEYWORDS: &[&str] = &[
    "form",
    "wizard",
    "step",
    "validation",
    "field",
    "input",
    "submit",
];
const DASHBOARD_KEYWORDS: &[&str] = &[
    "dashboard",
    "widget",
    "chart",
    "metric",
    "panel",
    "analytics",
    "report",
];

fn keyword_score(entities: &[String], keywords: &[&str]) -> f64 {
    if entities.is_empty() {
        return 0.0;
    }
    let lower: Vec<String> = entities.iter().map(|e| e.to_lowercase()).collect();
    let matches = keywords
        .iter()
        .filter(|kw| lower.iter().any(|e| e.contains(*kw)))
        .count();
    (matches as f64) / (keywords.len() as f64)
}

fn classify_archetype(
    entities: &[String],
    hierarchy_depth: usize,
    parsed_specs: &[(Spec, std::path::PathBuf)],
) -> ArchetypeResult {
    let spec_count = parsed_specs.len();

    // Score each archetype by keyword matching
    let ecommerce_score = keyword_score(entities, ECOMMERCE_KEYWORDS);
    let content_score = keyword_score(entities, CONTENT_KEYWORDS);
    let form_score = keyword_score(entities, FORM_KEYWORDS);
    let dashboard_kw_score = keyword_score(entities, DASHBOARD_KEYWORDS);

    // Structural signals for dashboard
    let mut structural_dashboard: f64 = 0.0;
    if spec_count > 15 {
        structural_dashboard += 0.3;
    }
    if hierarchy_depth > 4 {
        structural_dashboard += 0.3;
    }
    if parsed_specs.iter().any(|(s, _)| s.name.contains("web")) {
        structural_dashboard += 0.2;
    }

    let dashboard_score = dashboard_kw_score.max(structural_dashboard);

    let mut scores: Vec<(&str, f64)> = vec![
        ("e-commerce", ecommerce_score),
        ("content", content_score),
        ("form-heavy", form_score),
        ("dashboard", dashboard_score),
    ];
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let (best_name, best_score) = scores[0];

    let (name, reasoning) = if best_score >= 0.7 {
        (best_name, format!(
            "Strong {} signal: {:.0}% keyword/structural match across {} entities from {} specs",
            best_name, best_score * 100.0, entities.len(), spec_count
        ))
    } else if best_score >= 0.5 {
        (best_name, format!(
            "Moderate {} signal: {:.0}% keyword match across {} entities from {} specs",
            best_name, best_score * 100.0, entities.len(), spec_count
        ))
    } else {
        ("generic", format!(
            "No strong archetype signal detected (best: {} at {:.0}%); defaulting to generic for {} specs with {} entities",
            best_name, best_score * 100.0, spec_count, entities.len()
        ))
    };

    ArchetypeResult {
        name: name.to_string(),
        confidence: best_score,
        reasoning,
    }
}

// ── Metadata ───────────────────────────────────────────────────

fn build_metadata(
    dir: &Path,
    parsed_specs: &[(Spec, std::path::PathBuf)],
    entity_count: usize,
) -> SpecMetadata {
    let project_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Group specs by parent directory (domain)
    let mut domain_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (spec, path) in parsed_specs {
        let domain = path
            .parent()
            .and_then(|p| {
                if p == dir {
                    None
                } else {
                    p.file_name().and_then(|n| n.to_str())
                }
            })
            .unwrap_or("root")
            .to_string();
        domain_map
            .entry(domain)
            .or_default()
            .push(spec.name.clone());
    }

    let domains: Vec<Domain> = domain_map
        .into_iter()
        .map(|(name, spec_names)| Domain {
            spec_count: spec_names.len(),
            name,
            spec_names,
        })
        .collect();

    // Per-spec metrics
    let spec_metrics: Vec<SpecMetric> = parsed_specs
        .iter()
        .map(|(spec, _)| SpecMetric {
            name: spec.name.clone(),
            version: spec.version.clone(),
            behavior_count: spec.behaviors.len(),
        })
        .collect();

    let total_spec_count = parsed_specs.len();
    let total_behavior_count: usize = parsed_specs.iter().map(|(s, _)| s.behaviors.len()).sum();

    SpecMetadata {
        project_name,
        domains,
        spec_metrics,
        total_spec_count,
        total_behavior_count,
        total_entity_count: entity_count,
    }
}

// ── Structural metrics ─────────────────────────────────────────

fn compute_hierarchy_depth(dir: &Path, spec_files: &[std::path::PathBuf]) -> usize {
    spec_files
        .iter()
        .filter_map(|p| p.strip_prefix(dir).ok())
        .map(|rel| rel.components().count())
        .max()
        .unwrap_or(1)
}

fn compute_behavior_distribution(specs: &[(Spec, std::path::PathBuf)]) -> BehaviorDistribution {
    let mut happy = 0usize;
    let mut error = 0usize;
    let mut edge = 0usize;
    for (spec, _) in specs {
        for b in &spec.behaviors {
            match b.category {
                BehaviorCategory::HappyPath => happy += 1,
                BehaviorCategory::ErrorCase => error += 1,
                BehaviorCategory::EdgeCase => edge += 1,
            }
        }
    }
    BehaviorDistribution {
        happy_path: happy,
        error_case: error,
        edge_case: edge,
    }
}

// ── Description generation ─────────────────────────────────────

fn generate_description(
    entities: &[String],
    archetype: &ArchetypeResult,
    metadata: &SpecMetadata,
) -> String {
    let entity_list = if entities.len() <= 5 {
        entities.join(", ")
    } else {
        let first_five = &entities[..5];
        format!("{} and {} more", first_five.join(", "), entities.len() - 5)
    };

    format!(
        "A {} application with {} specs across {} domains, working with entities: {}",
        archetype.name,
        metadata.total_spec_count,
        metadata.domains.len(),
        entity_list,
    )
}
