use minter::core::design::taste::{TasteError, resolve_taste, resolve_taste_with_spacing};

// ─── Helper: WCAG contrast math ────────────────────────────────────────────

fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(hex: &str) -> f64 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f64 / 255.0;
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

fn contrast_ratio(hex1: &str, hex2: &str) -> f64 {
    let l1 = luminance(hex1);
    let l2 = luminance(hex2);
    let (a, b) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (a + 0.05) / (b + 0.05)
}

// ─── Palette: default ──────────────────────────────────────────────────────

/// design-taste: apply-palette-color-roles (default palette)
#[test]
fn palette_default_has_correct_metadata() {
    let taste = resolve_taste(None).unwrap();
    assert_eq!(taste.palette.name, "default");
    assert_eq!(taste.palette.primary, "#2563eb");
    assert!(
        taste
            .palette
            .description
            .to_lowercase()
            .contains("professional"),
        "description should mention professional: {}",
        taste.palette.description
    );
    assert!(taste.palette.vibe.contains(&"professional".to_string()));
    assert!(taste.palette.vibe.contains(&"trustworthy".to_string()));
    assert!(taste.palette.vibe.contains(&"clean".to_string()));
    assert!(
        taste.palette.reference.to_lowercase().contains("github"),
        "reference should mention GitHub: {}",
        taste.palette.reference
    );
    assert_eq!(taste.palette.shades.len(), 10);
}

/// design-taste: apply-palette-color-roles (slate-blue palette)
#[test]
fn palette_slate_blue_has_correct_metadata() {
    let taste = resolve_taste(Some("slate-blue")).unwrap();
    assert_eq!(taste.palette.name, "slate-blue");
    assert_eq!(taste.palette.primary, "#1e40af");
    assert!(
        taste
            .palette
            .description
            .to_lowercase()
            .contains("data-dense")
            || taste.palette.description.to_lowercase().contains("dark"),
        "description should mention data-dense or dark: {}",
        taste.palette.description
    );
    assert!(taste.palette.vibe.contains(&"technical".to_string()));
    assert!(taste.palette.vibe.contains(&"data-focused".to_string()));
    assert!(
        taste.palette.reference.to_lowercase().contains("linear"),
        "reference should mention Linear: {}",
        taste.palette.reference
    );
    assert_eq!(taste.palette.shades.len(), 10);
}

/// design-taste: apply-palette-color-roles (midnight palette)
#[test]
fn palette_midnight_has_correct_metadata() {
    let taste = resolve_taste(Some("midnight")).unwrap();
    assert_eq!(taste.palette.name, "midnight");
    assert_eq!(taste.palette.primary, "#4f46e5");
    assert_eq!(taste.palette.accent, "#db2777");
    assert!(
        taste.palette.description.to_lowercase().contains("indigo")
            || taste
                .palette
                .description
                .to_lowercase()
                .contains("contemporary"),
        "description should mention indigo or contemporary: {}",
        taste.palette.description
    );
    assert!(taste.palette.vibe.contains(&"modern".to_string()));
    assert!(taste.palette.vibe.contains(&"creative".to_string()));
    assert!(
        taste.palette.reference.to_lowercase().contains("figma"),
        "reference should mention Figma: {}",
        taste.palette.reference
    );
    assert_eq!(taste.palette.shades.len(), 10);
}

// ─── Semantic colors ───────────────────────────────────────────────────────

/// design-taste: apply-palette-color-roles (semantic colors present for all palettes)
#[test]
fn all_palettes_have_semantic_colors() {
    for name in &[None, Some("slate-blue"), Some("midnight")] {
        let taste = resolve_taste(*name).unwrap();
        let sem = &taste.palette.semantic;
        // All semantic colors must be valid hex
        for (label, color) in [
            ("success", &sem.success),
            ("warning", &sem.warning),
            ("error", &sem.error),
            ("info", &sem.info),
        ] {
            assert!(
                color.starts_with('#') && color.len() == 7,
                "palette {:?} semantic {} should be valid hex: {}",
                name,
                label,
                color
            );
        }
    }
}

// ─── Dark mode ─────────────────────────────────────────────────────────────

/// design-taste: apply-palette-light-dark-variants (dark mode for all palettes)
#[test]
fn all_palettes_have_dark_mode() {
    for name in &[None, Some("slate-blue"), Some("midnight")] {
        let taste = resolve_taste(*name).unwrap();
        let dm = &taste.palette.dark_mode;
        // Dark mode background should be dark (starts with #, valid hex)
        assert!(
            dm.background.starts_with('#') && dm.background.len() == 7,
            "palette {:?} dark_mode.background should be valid hex: {}",
            name,
            dm.background
        );
        assert!(
            dm.foreground.starts_with('#') && dm.foreground.len() == 7,
            "palette {:?} dark_mode.foreground should be valid hex: {}",
            name,
            dm.foreground
        );
        assert!(
            dm.primary.starts_with('#') && dm.primary.len() == 7,
            "palette {:?} dark_mode.primary should be valid hex: {}",
            name,
            dm.primary
        );
        assert_eq!(
            dm.shades.len(),
            10,
            "palette {:?} dark_mode should have 10 shades",
            name
        );
        // Dark mode semantic colors should exist
        let sem = &dm.semantic;
        assert!(sem.success.starts_with('#'));
        assert!(sem.warning.starts_with('#'));
        assert!(sem.error.starts_with('#'));
        assert!(sem.info.starts_with('#'));
    }
}

// ─── WCAG AA contrast ──────────────────────────────────────────────────────

/// design-taste: enforce-wcag-contrast (primary on white >= 4.5:1)
#[test]
fn wcag_aa_primary_on_white() {
    for name in &[None, Some("slate-blue"), Some("midnight")] {
        let taste = resolve_taste(*name).unwrap();
        let ratio = contrast_ratio(&taste.palette.primary, "#ffffff");
        assert!(
            ratio >= 4.5,
            "palette {:?} primary {} on white contrast is {:.2}, need >= 4.5",
            name,
            taste.palette.primary,
            ratio
        );
    }
}

/// design-taste: enforce-wcag-contrast (semantic colors on white >= 4.5:1)
#[test]
fn wcag_aa_semantic_on_white() {
    for name in &[None, Some("slate-blue"), Some("midnight")] {
        let taste = resolve_taste(*name).unwrap();
        let sem = &taste.palette.semantic;
        for (label, color) in [
            ("success", &sem.success),
            ("warning", &sem.warning),
            ("error", &sem.error),
            ("info", &sem.info),
        ] {
            let ratio = contrast_ratio(color, "#ffffff");
            assert!(
                ratio >= 4.5,
                "palette {:?} semantic {} ({}) on white contrast is {:.2}, need >= 4.5",
                name,
                label,
                color,
                ratio
            );
        }
    }
}

/// design-taste: enforce-wcag-contrast (dark mode foreground on background >= 4.5:1)
#[test]
fn wcag_aa_dark_mode_contrast() {
    for name in &[None, Some("slate-blue"), Some("midnight")] {
        let taste = resolve_taste(*name).unwrap();
        let dm = &taste.palette.dark_mode;
        let ratio = contrast_ratio(&dm.foreground, &dm.background);
        assert!(
            ratio >= 4.5,
            "palette {:?} dark mode fg {} on bg {} contrast is {:.2}, need >= 4.5",
            name,
            dm.foreground,
            dm.background,
            ratio
        );
    }
}

// ─── Type scale ────────────────────────────────────────────────────────────

/// design-taste: apply-type-scale (Major Third 1.25 ratio, 7 levels, body 16px)
#[test]
fn type_scale_defaults() {
    let taste = resolve_taste(None).unwrap();
    let ts = &taste.type_scale;
    assert!(
        (ts.ratio - 1.25).abs() < f64::EPSILON,
        "type scale ratio should be 1.25, got {}",
        ts.ratio
    );
    assert_eq!(ts.levels, 7, "type scale should have 7 levels");
    assert!(
        (ts.body_size - 16.0).abs() < f64::EPSILON,
        "body size should be 16px, got {}",
        ts.body_size
    );
    assert_eq!(ts.sizes.len(), 7, "should have 7 size values");
}

/// design-taste: apply-type-scale (sizes are progressive — each larger than the next)
#[test]
fn type_scale_progressive() {
    let taste = resolve_taste(None).unwrap();
    let ts = &taste.type_scale;
    // sizes[0] is h1 (largest), sizes[last] is caption (smallest)
    for i in 0..ts.sizes.len() - 1 {
        assert!(
            ts.sizes[i] > ts.sizes[i + 1],
            "size[{}]={} should be > size[{}]={}",
            i,
            ts.sizes[i],
            i + 1,
            ts.sizes[i + 1]
        );
    }
    // Line heights should exist for each level
    assert_eq!(ts.line_heights.len(), ts.sizes.len());
    // Font weights should exist for each level
    assert_eq!(ts.font_weights.len(), ts.sizes.len());
}

// ─── Spacing ───────────────────────────────────────────────────────────────

/// design-taste: apply-spacing-grid (4px base, 8 steps, ratio 1.5)
#[test]
fn spacing_defaults() {
    let taste = resolve_taste(None).unwrap();
    let sp = &taste.spacing;
    assert_eq!(sp.base, 4, "spacing base should be 4px");
    assert!(
        (sp.ratio - 1.5).abs() < f64::EPSILON,
        "spacing ratio should be 1.5, got {}",
        sp.ratio
    );
    assert_eq!(sp.steps.len(), 8, "should have 8 spacing steps");
    // All steps should be multiples of 4
    for (i, step) in sp.steps.iter().enumerate() {
        assert!(
            step % 4 == 0,
            "spacing step[{}]={} should be multiple of 4",
            i,
            step
        );
    }
}

/// design-taste: apply-spacing-grid (steps are progressive)
#[test]
fn spacing_progressive() {
    let taste = resolve_taste(None).unwrap();
    let sp = &taste.spacing;
    for i in 0..sp.steps.len() - 1 {
        assert!(
            sp.steps[i] < sp.steps[i + 1],
            "spacing step[{}]={} should be < step[{}]={}",
            i,
            sp.steps[i],
            i + 1,
            sp.steps[i + 1]
        );
    }
}

// ─── Animation ─────────────────────────────────────────────────────────────

/// design-taste: apply-animation-defaults (transition duration + easing)
#[test]
fn animation_defaults() {
    let taste = resolve_taste(None).unwrap();
    let anim = &taste.animation;
    assert!(
        (150..=300).contains(&anim.transition_duration_ms),
        "transition duration {}ms should be 150..=300",
        anim.transition_duration_ms
    );
    assert_eq!(anim.easing, "ease-out");
}

// ─── Component styles ──────────────────────────────────────────────────────

/// design-taste: enforce-composite-taste-constraints (border radius defaults)
#[test]
fn component_style_defaults() {
    let taste = resolve_taste(None).unwrap();
    let cs = &taste.component_styles;
    assert!(
        cs.button_border_radius > 0,
        "button border radius should be > 0"
    );
    assert!(
        cs.card_border_radius > 0,
        "card border radius should be > 0"
    );
}

// ─── Error: unknown palette ────────────────────────────────────────────────

/// design-taste: palette-fails-wcag-contrast (unknown palette returns error)
#[test]
fn error_unknown_palette() {
    let result = resolve_taste(Some("nonexistent-palette"));
    assert!(result.is_err(), "unknown palette should return error");
    let err = result.unwrap_err();
    match err {
        TasteError::UnknownPalette(name) => {
            assert_eq!(name, "nonexistent-palette");
        }
        other => panic!("expected UnknownPalette, got {:?}", other),
    }
}

/// design-taste: apply-spacing-grid (invalid spacing base returns error)
#[test]
fn error_invalid_spacing_base() {
    let result = resolve_taste_with_spacing(None, Some(0));
    assert!(result.is_err(), "spacing base 0 should return error");
    let err = result.unwrap_err();
    match err {
        TasteError::InvalidSpacingBase(val) => {
            assert_eq!(val, 0);
        }
        other => panic!("expected InvalidSpacingBase, got {:?}", other),
    }
}
