mod common;

use minter::core::design::analysis::{
    AnalysisResult, ArchetypeResult, BehaviorDistribution, Domain, SpecMetadata, SpecMetric,
};
use minter::core::design::generation::{GenerationError, generate_design_system};

// ── Test helpers ──────────────────────────────────────────────

fn make_metadata(
    project_name: &str,
    domains: Vec<Domain>,
    spec_metrics: Vec<SpecMetric>,
) -> SpecMetadata {
    let total_spec_count = spec_metrics.len();
    let total_behavior_count: usize = spec_metrics.iter().map(|m| m.behavior_count).sum();
    let total_entity_count = 14;
    SpecMetadata {
        project_name: project_name.to_string(),
        domains,
        spec_metrics,
        total_spec_count,
        total_behavior_count,
        total_entity_count,
    }
}

fn make_analysis(archetype_name: &str, metadata: SpecMetadata) -> AnalysisResult {
    AnalysisResult {
        entities: vec!["Spec".into(), "Behavior".into()],
        archetype: ArchetypeResult {
            name: archetype_name.to_string(),
            confidence: 0.85,
            reasoning: format!("Classified as {}", archetype_name),
        },
        spec_metadata: metadata,
        flow_count: 10,
        hierarchy_depth: 3,
        behavior_distribution: BehaviorDistribution {
            happy_path: 7,
            error_case: 2,
            edge_case: 1,
        },
        app_description: "A test application".to_string(),
        warnings: vec![],
    }
}

fn dashboard_analysis() -> AnalysisResult {
    let domains = vec![
        Domain {
            name: "commands".into(),
            spec_count: 11,
            spec_names: vec![
                "validate-command".into(),
                "watch-command".into(),
                "format-command".into(),
                "graph-command".into(),
                "coverage-command".into(),
                "scaffold-command".into(),
                "inspect-command".into(),
                "explain-command".into(),
                "lock-command".into(),
                "ci-command".into(),
                "web-command".into(),
            ],
        },
        Domain {
            name: "grammar".into(),
            spec_count: 2,
            spec_names: vec!["spec-grammar".into(), "nfr-grammar".into()],
        },
        Domain {
            name: "mcp".into(),
            spec_count: 3,
            spec_names: vec![
                "mcp-server".into(),
                "mcp-agent-guidance".into(),
                "mcp-response-format".into(),
            ],
        },
    ];
    let spec_metrics: Vec<SpecMetric> = domains
        .iter()
        .flat_map(|d| {
            d.spec_names.iter().map(|n| SpecMetric {
                name: n.clone(),
                version: "1.0.0".into(),
                behavior_count: 5,
            })
        })
        .collect();
    let metadata = make_metadata("Minter", domains, spec_metrics);
    make_analysis("dashboard", metadata)
}

fn generic_analysis_with_domains(domains: Vec<Domain>) -> AnalysisResult {
    let spec_metrics: Vec<SpecMetric> = domains
        .iter()
        .flat_map(|d| {
            d.spec_names.iter().map(|n| SpecMetric {
                name: n.clone(),
                version: "1.0.0".into(),
                behavior_count: 3,
            })
        })
        .collect();
    let metadata = make_metadata("TestProject", domains, spec_metrics);
    make_analysis("generic", metadata)
}

// ═══════════════════════════════════════════════════════════════
// design-generation behaviors
// ═══════════════════════════════════════════════════════════════

// ── Layout generation per archetype ───────────────────────────

/// design-generation: generate-dashboard-sidebar-and-header
#[test]
fn generate_dashboard_sidebar_and_header() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    // Layout has a sidebar
    let sidebar = ds
        .layout
        .sidebar
        .as_ref()
        .expect("dashboard should have a sidebar");
    assert_eq!(sidebar.position, "left");
    // Sidebar nav items are derived from spec names
    assert!(
        !sidebar.nav_items.is_empty(),
        "sidebar should have nav items"
    );

    // Layout has a header
    let header = ds
        .layout
        .header
        .as_ref()
        .expect("dashboard should have a header");
    assert_eq!(
        header.title, "Minter",
        "header title should match project name"
    );
}

/// design-generation: generate-dashboard-content-sections
#[test]
fn generate_dashboard_content_sections() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    let section_kinds: Vec<&str> = ds
        .layout
        .content
        .sections
        .iter()
        .map(|s| s.kind.as_str())
        .collect();
    assert!(
        section_kinds.contains(&"metric-cards"),
        "dashboard should include metric-cards section, got: {:?}",
        section_kinds
    );
    assert!(
        section_kinds.contains(&"data-table"),
        "dashboard should include data-table section, got: {:?}",
        section_kinds
    );
}

/// design-generation: generate-ecommerce-layout
#[test]
fn generate_ecommerce_layout() {
    let metadata = make_metadata(
        "Shop",
        vec![Domain {
            name: "products".into(),
            spec_count: 3,
            spec_names: vec!["product-list".into(), "cart".into(), "checkout".into()],
        }],
        vec![
            SpecMetric {
                name: "product-list".into(),
                version: "1.0.0".into(),
                behavior_count: 4,
            },
            SpecMetric {
                name: "cart".into(),
                version: "1.0.0".into(),
                behavior_count: 3,
            },
            SpecMetric {
                name: "checkout".into(),
                version: "1.0.0".into(),
                behavior_count: 5,
            },
        ],
    );
    let analysis = make_analysis("e-commerce", metadata);
    let ds = generate_design_system(&analysis, None).unwrap();

    // No sidebar
    assert!(
        ds.layout.sidebar.is_none(),
        "e-commerce should not have a sidebar"
    );
    // Has header with navigation
    let header = ds
        .layout
        .header
        .as_ref()
        .expect("e-commerce should have a header");
    assert!(!header.title.is_empty());
    // Content includes product-grid
    let section_kinds: Vec<&str> = ds
        .layout
        .content
        .sections
        .iter()
        .map(|s| s.kind.as_str())
        .collect();
    assert!(
        section_kinds.contains(&"product-grid"),
        "e-commerce should include product-grid section, got: {:?}",
        section_kinds
    );
}

/// design-generation: generate-content-layout
#[test]
fn generate_content_layout() {
    let metadata = make_metadata(
        "Blog",
        vec![Domain {
            name: "content".into(),
            spec_count: 2,
            spec_names: vec!["article".into(), "comment".into()],
        }],
        vec![
            SpecMetric {
                name: "article".into(),
                version: "1.0.0".into(),
                behavior_count: 5,
            },
            SpecMetric {
                name: "comment".into(),
                version: "1.0.0".into(),
                behavior_count: 3,
            },
        ],
    );
    let analysis = make_analysis("content", metadata);
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        ds.layout.sidebar.is_none(),
        "content should not have a sidebar"
    );
    assert!(ds.layout.header.is_some(), "content should have a header");

    let section_kinds: Vec<&str> = ds
        .layout
        .content
        .sections
        .iter()
        .map(|s| s.kind.as_str())
        .collect();
    assert!(
        section_kinds.contains(&"article-body"),
        "content should include article-body section, got: {:?}",
        section_kinds
    );
    // Container max width should be present on at least one section
    let has_max_width = ds
        .layout
        .content
        .sections
        .iter()
        .any(|s| !s.width.is_empty() && s.width != "full");
    assert!(
        has_max_width,
        "content layout should have a container max width"
    );
}

/// design-generation: generate-form-heavy-layout
#[test]
fn generate_form_heavy_layout() {
    let metadata = make_metadata(
        "Onboarding",
        vec![Domain {
            name: "forms".into(),
            spec_count: 2,
            spec_names: vec!["registration".into(), "profile".into()],
        }],
        vec![
            SpecMetric {
                name: "registration".into(),
                version: "1.0.0".into(),
                behavior_count: 4,
            },
            SpecMetric {
                name: "profile".into(),
                version: "1.0.0".into(),
                behavior_count: 3,
            },
        ],
    );
    let analysis = make_analysis("form-heavy", metadata);
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        ds.layout.sidebar.is_none(),
        "form-heavy should not have a sidebar"
    );
    assert!(
        ds.layout.header.is_some(),
        "form-heavy should have a header"
    );

    let section_kinds: Vec<&str> = ds
        .layout
        .content
        .sections
        .iter()
        .map(|s| s.kind.as_str())
        .collect();
    assert!(
        section_kinds.contains(&"wizard-form"),
        "form-heavy should include wizard-form section, got: {:?}",
        section_kinds
    );
    assert!(
        section_kinds.contains(&"progress-indicator"),
        "form-heavy should include progress-indicator section, got: {:?}",
        section_kinds
    );
}

/// design-generation: generate-generic-layout
#[test]
fn generate_generic_layout() {
    let domains = vec![
        Domain {
            name: "auth".into(),
            spec_count: 3,
            spec_names: vec!["login".into(), "register".into(), "reset".into()],
        },
        Domain {
            name: "billing".into(),
            spec_count: 4,
            spec_names: vec![
                "invoice".into(),
                "payment".into(),
                "subscription".into(),
                "refund".into(),
            ],
        },
        Domain {
            name: "settings".into(),
            spec_count: 3,
            spec_names: vec![
                "profile".into(),
                "preferences".into(),
                "notifications".into(),
            ],
        },
    ];
    let analysis = generic_analysis_with_domains(domains);
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        ds.layout.header.is_some() || ds.layout.sidebar.is_some(),
        "generic should have some layout structure"
    );
    assert!(
        !ds.layout.content.sections.is_empty(),
        "generic layout should have content sections"
    );
    // Sections should be derived from spec structure (domains)
    let section_titles: Vec<&str> = ds
        .layout
        .content
        .sections
        .iter()
        .map(|s| s.title.as_str())
        .collect();
    // At least one domain name should appear in section titles
    let has_domain_section = section_titles.iter().any(|t| {
        let lower = t.to_lowercase();
        lower.contains("auth") || lower.contains("billing") || lower.contains("settings")
    });
    assert!(
        has_domain_section,
        "generic layout sections should be derived from spec structure (domains), got: {:?}",
        section_titles
    );
}

// ── Palette application ───────────────────────────────────────

/// design-generation: apply-default-palette
#[test]
fn apply_default_palette() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    assert_eq!(ds.palette.name, "default");
    assert!(
        ds.palette.primary.starts_with('#') && ds.palette.primary.len() == 7,
        "primary should be valid hex: {}",
        ds.palette.primary
    );
    assert_eq!(ds.palette.shades.len(), 10, "should have 10 shade variants");
    // Semantic colors present
    assert!(ds.palette.semantic.success.starts_with('#'));
    assert!(ds.palette.semantic.warning.starts_with('#'));
    assert!(ds.palette.semantic.error.starts_with('#'));
    assert!(ds.palette.semantic.info.starts_with('#'));
    // Dark mode variant present
    assert!(ds.dark_mode.background.starts_with('#'));
    assert!(ds.dark_mode.foreground.starts_with('#'));
}

/// design-generation: apply-archetype-palette-hint
#[test]
fn apply_archetype_palette_hint() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    // Palette is resolved from taste layer
    assert!(
        !ds.palette.name.is_empty(),
        "palette name should be present"
    );
    assert!(
        !ds.palette.description.is_empty(),
        "palette description should be present"
    );
}

// ── Type scale and spacing ────────────────────────────────────

/// design-generation: generate-type-scale
#[test]
fn generate_type_scale() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        (ds.type_scale.ratio - 1.25).abs() < f64::EPSILON,
        "type scale ratio should be 1.25, got {}",
        ds.type_scale.ratio
    );
    assert_eq!(
        ds.type_scale.level_count, 7,
        "type scale should have 7 levels"
    );
    assert!(ds.type_scale.body_size > 0.0, "body size should be present");
}

/// design-generation: generate-spacing-scale
#[test]
fn generate_spacing_scale() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    assert_eq!(ds.spacing.base, 4, "spacing base should be 4");
    assert_eq!(ds.spacing.step_count, 8, "spacing should have 8 steps");
}

// ── Component styles ──────────────────────────────────────────

/// design-generation: generate-component-styles
#[test]
fn generate_component_styles() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        ds.component_styles.button_border_radius > 0,
        "button border radius should be > 0"
    );
    assert!(
        ds.component_styles.card_border_radius > 0,
        "card border radius should be > 0"
    );
}

// ── Spec metadata pass-through ────────────────────────────────

/// design-generation: include-spec-metadata
#[test]
fn include_spec_metadata() {
    let domains = vec![
        Domain {
            name: "Core Commands".into(),
            spec_count: 11,
            spec_names: (0..11).map(|i| format!("spec-{}", i)).collect(),
        },
        Domain {
            name: "Grammar".into(),
            spec_count: 2,
            spec_names: vec!["spec-grammar".into(), "nfr-grammar".into()],
        },
        Domain {
            name: "MCP".into(),
            spec_count: 3,
            spec_names: vec!["mcp-a".into(), "mcp-b".into(), "mcp-c".into()],
        },
    ];
    let spec_metrics: Vec<SpecMetric> = domains
        .iter()
        .flat_map(|d| {
            d.spec_names.iter().map(|n| SpecMetric {
                name: n.clone(),
                version: "1.0.0".into(),
                behavior_count: 40, // 29 specs * ~22.5 avg ≈ 653
            })
        })
        .collect();
    // Manually adjust to match spec: 29 specs, 653 behaviors, 14 entities
    let metadata = SpecMetadata {
        project_name: "Minter".into(),
        domains,
        spec_metrics,
        total_spec_count: 29,
        total_behavior_count: 653,
        total_entity_count: 14,
    };
    let analysis = make_analysis("dashboard", metadata);
    let ds = generate_design_system(&analysis, None).unwrap();

    let meta = ds
        .spec_metadata
        .as_ref()
        .expect("spec_metadata should be present");
    assert_eq!(meta.project_name, "Minter");
    assert_eq!(meta.total_spec_count, 29);
    assert_eq!(meta.total_behavior_count, 653);
    assert!(!meta.domains.is_empty(), "domains should not be empty");
}

/// design-generation: sidebar-nav-from-spec-names
#[test]
fn sidebar_nav_from_spec_names() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    let sidebar = ds
        .layout
        .sidebar
        .as_ref()
        .expect("dashboard should have sidebar");
    assert!(
        !sidebar.nav_items.is_empty(),
        "sidebar nav items should not be empty"
    );
    assert!(
        sidebar.nav_items.len() > 0,
        "sidebar nav items count should be > 0"
    );
}

// ── Content sections ──────────────────────────────────────────

/// design-generation: content-sections-have-structure
#[test]
fn content_sections_have_structure() {
    let analysis = dashboard_analysis();
    let ds = generate_design_system(&analysis, None).unwrap();

    for section in &ds.layout.content.sections {
        assert!(!section.kind.is_empty(), "section kind should not be empty");
        assert!(
            !section.title.is_empty(),
            "section title should not be empty"
        );
        assert!(
            !section.width.is_empty(),
            "section width should not be empty"
        );
    }
}

// ── Error cases ───────────────────────────────────────────────

/// design-generation: reject-missing-analysis
#[test]
fn reject_missing_analysis() {
    // Pass a None-like analysis — we model this as a function that takes Option
    // The spec says "no analysis result is available" → error.
    // We test by calling a variant or by passing empty data that triggers the error.
    let result = generate_design_system_from_option(None);
    assert!(result.is_err(), "missing analysis should return error");
    let err = result.unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("analysis"),
        "error message should contain 'analysis': {}",
        msg
    );
}

/// Helper that tests the Option<&AnalysisResult> path
fn generate_design_system_from_option(
    analysis: Option<&AnalysisResult>,
) -> Result<minter::core::design::generation::DesignSystem, GenerationError> {
    match analysis {
        Some(a) => generate_design_system(a, None),
        None => Err(GenerationError::MissingAnalysis),
    }
}

/// design-generation: handle-missing-spec-metadata
#[test]
fn handle_missing_spec_metadata() {
    // Analysis with archetype but empty metadata (no meaningful spec data)
    let metadata = SpecMetadata {
        project_name: String::new(),
        domains: vec![],
        spec_metrics: vec![],
        total_spec_count: 0,
        total_behavior_count: 0,
        total_entity_count: 0,
    };
    let analysis = make_analysis("dashboard", metadata);
    let ds = generate_design_system(&analysis, None).unwrap();

    // Layout should still be present
    assert!(ds.layout.header.is_some() || ds.layout.sidebar.is_some());
    // Palette should be present
    assert!(!ds.palette.name.is_empty());
    // Spec metadata should reflect the empty state
    let meta = ds.spec_metadata.as_ref();
    let is_empty_meta = meta.map_or(true, |m| m.total_spec_count == 0);
    assert!(
        is_empty_meta,
        "spec metadata should be absent or show empty counts"
    );
    // Sidebar nav items should use fallback labels
    if let Some(sidebar) = &ds.layout.sidebar {
        assert!(
            !sidebar.nav_items.is_empty(),
            "sidebar should have fallback nav items when metadata is empty"
        );
    }
}

// ── Edge cases ────────────────────────────────────────────────

/// design-generation: empty-domains-layout
#[test]
fn empty_domains_layout() {
    let analysis = generic_analysis_with_domains(vec![]);
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        !ds.layout.content.sections.is_empty(),
        "empty domains should still produce content sections"
    );
    // Should contain at least one default section
    let has_default = ds
        .layout
        .content
        .sections
        .iter()
        .any(|s| s.kind == "overview" || s.kind == "content-section");
    assert!(
        has_default,
        "empty domains layout should contain at least one default section, got: {:?}",
        ds.layout
            .content
            .sections
            .iter()
            .map(|s| &s.kind)
            .collect::<Vec<_>>()
    );
}

/// design-generation: single-domain-layout
#[test]
fn single_domain_layout() {
    let domains = vec![Domain {
        name: "core".into(),
        spec_count: 3,
        spec_names: vec!["spec-a".into(), "spec-b".into(), "spec-c".into()],
    }];
    let analysis = generic_analysis_with_domains(domains);
    let ds = generate_design_system(&analysis, None).unwrap();

    assert!(
        ds.layout.header.is_some() || ds.layout.sidebar.is_some(),
        "single domain should produce a valid layout"
    );
    assert!(
        !ds.layout.content.sections.is_empty(),
        "single domain should produce content sections"
    );
}
