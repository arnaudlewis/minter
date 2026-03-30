use std::fmt;

use serde::{Deserialize, Serialize};

use super::analysis::{AnalysisResult, Domain, SpecMetadata};
use super::taste;

// ─── Error type ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum GenerationError {
    MissingAnalysis,
    TasteError(taste::TasteError),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::MissingAnalysis => {
                write!(f, "missing analysis result: cannot generate design system")
            }
            GenerationError::TasteError(e) => write!(f, "taste error: {e}"),
        }
    }
}

impl std::error::Error for GenerationError {}

impl From<taste::TasteError> for GenerationError {
    fn from(e: taste::TasteError) -> Self {
        GenerationError::TasteError(e)
    }
}

// ─── Public types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSystem {
    pub archetype: ArchetypeInfo,
    pub palette: PaletteInfo,
    pub dark_mode: DarkModeInfo,
    pub type_scale: TypeScaleInfo,
    pub spacing: SpacingInfo,
    pub component_styles: ComponentStyleInfo,
    pub layout: LayoutConfig,
    pub spec_metadata: Option<SpecMetadataInfo>,
    pub decisions: Vec<DesignDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeInfo {
    pub name: String,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteInfo {
    pub name: String,
    pub description: String,
    pub vibe: Vec<String>,
    pub reference: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub neutral: String,
    pub semantic: SemanticColorsInfo,
    pub shades: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticColorsInfo {
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkModeInfo {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub shades: Vec<String>,
    pub semantic: SemanticColorsInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScaleInfo {
    pub ratio: f64,
    pub level_count: usize,
    pub body_size: f64,
    pub sizes: Vec<f64>,
    pub line_heights: Vec<f64>,
    pub font_weights: Vec<u16>,
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingInfo {
    pub base: usize,
    pub ratio: f64,
    pub step_count: usize,
    pub steps: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStyleInfo {
    pub button_border_radius: usize,
    pub card_border_radius: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub sidebar: Option<SidebarConfig>,
    pub header: Option<HeaderConfig>,
    pub content: ContentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarConfig {
    pub position: String,
    pub width: String,
    pub nav_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    pub title: String,
    pub has_search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    pub sections: Vec<ContentSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    pub kind: String,
    pub title: String,
    pub width: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadataInfo {
    pub project_name: String,
    pub domains: Vec<DomainInfo>,
    pub spec_metrics: Vec<SpecMetricInfo>,
    pub total_spec_count: usize,
    pub total_behavior_count: usize,
    pub total_entity_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub name: String,
    pub spec_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetricInfo {
    pub name: String,
    pub version: String,
    pub behavior_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignDecision {
    pub property: String,
    pub value: String,
    pub previous_value: Option<String>,
}

// ─── Public API ───────────────────────────────────────────────────────────

/// Generate a complete `DesignSystem` from an analysis result.
///
/// `palette_override` selects a named palette from the taste layer.
/// Passing `None` uses the default palette.
pub fn generate_design_system(
    analysis: &AnalysisResult,
    palette_override: Option<&str>,
) -> Result<DesignSystem, GenerationError> {
    // 1. Resolve taste config
    let taste_config = taste::resolve_taste(palette_override)?;

    let archetype_name = &analysis.archetype.name;

    // 2. Build layout from archetype
    let layout = build_layout(archetype_name, &analysis.spec_metadata);

    // 3. Map taste into DesignSystem types
    let palette = map_palette(&taste_config.palette);
    let dark_mode = map_dark_mode(&taste_config.palette.dark_mode);
    let type_scale = map_type_scale(&taste_config.type_scale);
    let spacing = map_spacing(&taste_config.spacing);
    let component_styles = map_component_styles(&taste_config.component_styles);

    // 4. Carry spec_metadata
    let spec_metadata = map_spec_metadata(&analysis.spec_metadata);

    // 5. Build decisions log
    let decisions = build_decisions(archetype_name, &palette.name);

    Ok(DesignSystem {
        archetype: ArchetypeInfo {
            name: analysis.archetype.name.clone(),
            confidence: analysis.archetype.confidence,
            reasoning: analysis.archetype.reasoning.clone(),
        },
        palette,
        dark_mode,
        type_scale,
        spacing,
        component_styles,
        layout,
        spec_metadata,
        decisions,
    })
}

// ─── Layout builders ──────────────────────────────────────────────────────

fn build_layout(archetype: &str, metadata: &SpecMetadata) -> LayoutConfig {
    match archetype {
        "dashboard" => build_dashboard_layout(metadata),
        "e-commerce" => build_ecommerce_layout(metadata),
        "content" => build_content_layout(metadata),
        "form-heavy" => build_form_heavy_layout(metadata),
        _ => build_generic_layout(metadata),
    }
}

fn build_dashboard_layout(metadata: &SpecMetadata) -> LayoutConfig {
    let nav_items = derive_nav_items(metadata);

    LayoutConfig {
        sidebar: Some(SidebarConfig {
            position: "left".into(),
            width: "240px".into(),
            nav_items,
        }),
        header: Some(HeaderConfig {
            title: project_name_or_default(metadata),
            has_search: true,
        }),
        content: ContentConfig {
            sections: vec![
                ContentSection {
                    kind: "metric-cards".into(),
                    title: "Overview".into(),
                    width: "full".into(),
                },
                ContentSection {
                    kind: "data-table".into(),
                    title: "Details".into(),
                    width: "full".into(),
                },
            ],
        },
    }
}

fn build_ecommerce_layout(metadata: &SpecMetadata) -> LayoutConfig {
    LayoutConfig {
        sidebar: None,
        header: Some(HeaderConfig {
            title: project_name_or_default(metadata),
            has_search: true,
        }),
        content: ContentConfig {
            sections: vec![
                ContentSection {
                    kind: "hero".into(),
                    title: "Featured".into(),
                    width: "full".into(),
                },
                ContentSection {
                    kind: "product-grid".into(),
                    title: "Products".into(),
                    width: "full".into(),
                },
            ],
        },
    }
}

fn build_content_layout(metadata: &SpecMetadata) -> LayoutConfig {
    LayoutConfig {
        sidebar: None,
        header: Some(HeaderConfig {
            title: project_name_or_default(metadata),
            has_search: false,
        }),
        content: ContentConfig {
            sections: vec![ContentSection {
                kind: "article-body".into(),
                title: "Content".into(),
                width: "720px".into(),
            }],
        },
    }
}

fn build_form_heavy_layout(metadata: &SpecMetadata) -> LayoutConfig {
    LayoutConfig {
        sidebar: None,
        header: Some(HeaderConfig {
            title: project_name_or_default(metadata),
            has_search: false,
        }),
        content: ContentConfig {
            sections: vec![
                ContentSection {
                    kind: "progress-indicator".into(),
                    title: "Progress".into(),
                    width: "full".into(),
                },
                ContentSection {
                    kind: "wizard-form".into(),
                    title: "Form".into(),
                    width: "640px".into(),
                },
            ],
        },
    }
}

fn build_generic_layout(metadata: &SpecMetadata) -> LayoutConfig {
    let sections = if metadata.domains.is_empty() {
        // Fallback: single overview section
        vec![ContentSection {
            kind: "overview".into(),
            title: "Overview".into(),
            width: "full".into(),
        }]
    } else {
        // One content-section per domain
        metadata
            .domains
            .iter()
            .map(|d| ContentSection {
                kind: "content-section".into(),
                title: capitalize(&d.name),
                width: "full".into(),
            })
            .collect()
    };

    LayoutConfig {
        sidebar: None,
        header: Some(HeaderConfig {
            title: project_name_or_default(metadata),
            has_search: false,
        }),
        content: ContentConfig { sections },
    }
}

// ─── Nav item derivation ──────────────────────────────────────────────────

const MAX_NAV_ITEMS: usize = 10;

fn derive_nav_items(metadata: &SpecMetadata) -> Vec<String> {
    if metadata.spec_metrics.is_empty() && metadata.domains.is_empty() {
        return vec!["Dashboard".into(), "Settings".into()];
    }

    // Collect spec names, preferring specs from the largest domain
    let mut spec_names: Vec<String> = if metadata.domains.is_empty() {
        metadata
            .spec_metrics
            .iter()
            .map(|m| m.name.clone())
            .collect()
    } else {
        let mut sorted_domains: Vec<&Domain> = metadata.domains.iter().collect();
        sorted_domains.sort_by(|a, b| b.spec_count.cmp(&a.spec_count));

        sorted_domains
            .iter()
            .flat_map(|d| d.spec_names.iter().cloned())
            .collect()
    };

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    spec_names.retain(|n| seen.insert(n.clone()));

    // Transform: remove "-command" suffix, capitalize
    spec_names
        .into_iter()
        .take(MAX_NAV_ITEMS)
        .map(|name| {
            let stripped = name.strip_suffix("-command").unwrap_or(&name);
            capitalize(stripped)
        })
        .collect()
}

// ─── Type mapping ─────────────────────────────────────────────────────────

fn map_palette(p: &taste::PaletteDefinition) -> PaletteInfo {
    PaletteInfo {
        name: p.name.clone(),
        description: p.description.clone(),
        vibe: p.vibe.clone(),
        reference: p.reference.clone(),
        primary: p.primary.clone(),
        secondary: p.secondary.clone(),
        accent: p.accent.clone(),
        neutral: p.neutral.clone(),
        semantic: SemanticColorsInfo {
            success: p.semantic.success.clone(),
            warning: p.semantic.warning.clone(),
            error: p.semantic.error.clone(),
            info: p.semantic.info.clone(),
        },
        shades: p.shades.clone(),
    }
}

fn map_dark_mode(dm: &taste::DarkModeColors) -> DarkModeInfo {
    DarkModeInfo {
        background: dm.background.clone(),
        foreground: dm.foreground.clone(),
        primary: dm.primary.clone(),
        shades: dm.shades.clone(),
        semantic: SemanticColorsInfo {
            success: dm.semantic.success.clone(),
            warning: dm.semantic.warning.clone(),
            error: dm.semantic.error.clone(),
            info: dm.semantic.info.clone(),
        },
    }
}

fn map_type_scale(ts: &taste::TypeScaleConfig) -> TypeScaleInfo {
    TypeScaleInfo {
        ratio: ts.ratio,
        level_count: ts.levels,
        body_size: ts.body_size,
        sizes: ts.sizes.clone(),
        line_heights: ts.line_heights.clone(),
        font_weights: ts.font_weights.clone(),
        font_family: ts.font_family.clone(),
    }
}

fn map_spacing(sp: &taste::SpacingConfig) -> SpacingInfo {
    SpacingInfo {
        base: sp.base,
        ratio: sp.ratio,
        step_count: sp.steps.len(),
        steps: sp.steps.clone(),
    }
}

fn map_component_styles(cs: &taste::ComponentStyleConfig) -> ComponentStyleInfo {
    ComponentStyleInfo {
        button_border_radius: cs.button_border_radius,
        card_border_radius: cs.card_border_radius,
    }
}

fn map_spec_metadata(meta: &SpecMetadata) -> Option<SpecMetadataInfo> {
    Some(SpecMetadataInfo {
        project_name: meta.project_name.clone(),
        domains: meta
            .domains
            .iter()
            .map(|d| DomainInfo {
                name: d.name.clone(),
                spec_count: d.spec_count,
            })
            .collect(),
        spec_metrics: meta
            .spec_metrics
            .iter()
            .map(|m| SpecMetricInfo {
                name: m.name.clone(),
                version: m.version.clone(),
                behavior_count: m.behavior_count,
            })
            .collect(),
        total_spec_count: meta.total_spec_count,
        total_behavior_count: meta.total_behavior_count,
        total_entity_count: meta.total_entity_count,
    })
}

// ─── Decisions ────────────────────────────────────────────────────────────

fn build_decisions(archetype: &str, palette_name: &str) -> Vec<DesignDecision> {
    vec![
        DesignDecision {
            property: "archetype".into(),
            value: archetype.into(),
            previous_value: None,
        },
        DesignDecision {
            property: "palette".into(),
            value: palette_name.into(),
            previous_value: None,
        },
    ]
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn project_name_or_default(metadata: &SpecMetadata) -> String {
    if metadata.project_name.is_empty() {
        "Project".into()
    } else {
        metadata.project_name.clone()
    }
}
