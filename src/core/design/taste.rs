use std::fmt;

// ─── Error type ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum TasteError {
    UnknownPalette(String),
    InvalidSpacingBase(usize),
}

impl fmt::Display for TasteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TasteError::UnknownPalette(name) => write!(f, "unknown palette: {name}"),
            TasteError::InvalidSpacingBase(val) => write!(f, "invalid spacing base: {val}"),
        }
    }
}

impl std::error::Error for TasteError {}

// ─── Public types ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PaletteDefinition {
    pub name: String,
    pub description: String,
    pub vibe: Vec<String>,
    pub reference: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub neutral: String,
    pub semantic: SemanticColors,
    pub shades: Vec<String>,
    pub dark_mode: DarkModeColors,
}

#[derive(Debug)]
pub struct SemanticColors {
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

#[derive(Debug)]
pub struct DarkModeColors {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub shades: Vec<String>,
    pub semantic: SemanticColors,
}

#[derive(Debug)]
pub struct TasteConfig {
    pub palette: PaletteDefinition,
    pub type_scale: TypeScaleConfig,
    pub spacing: SpacingConfig,
    pub animation: AnimationConfig,
    pub component_styles: ComponentStyleConfig,
}

#[derive(Debug)]
pub struct TypeScaleConfig {
    pub ratio: f64,
    pub levels: usize,
    pub body_size: f64,
    pub sizes: Vec<f64>,
    pub line_heights: Vec<f64>,
    pub font_weights: Vec<u16>,
    pub font_family: String,
}

#[derive(Debug)]
pub struct SpacingConfig {
    pub base: usize,
    pub ratio: f64,
    pub steps: Vec<usize>,
}

#[derive(Debug)]
pub struct AnimationConfig {
    pub transition_duration_ms: usize,
    pub easing: String,
}

#[derive(Debug)]
pub struct ComponentStyleConfig {
    pub button_border_radius: usize,
    pub card_border_radius: usize,
}

// ─── Public API ────────────────────────────────────────────────────────────

/// Resolve a complete taste configuration for the given palette name.
/// Passing `None` selects the "default" palette.
pub fn resolve_taste(palette_name: Option<&str>) -> Result<TasteConfig, TasteError> {
    resolve_taste_with_spacing(palette_name, None)
}

/// Resolve taste with an optional custom spacing base (must be > 0).
pub fn resolve_taste_with_spacing(
    palette_name: Option<&str>,
    spacing_base: Option<usize>,
) -> Result<TasteConfig, TasteError> {
    let name = palette_name.unwrap_or("default");

    let palette = build_palette(name)?;

    let base = spacing_base.unwrap_or(4);
    if base == 0 {
        return Err(TasteError::InvalidSpacingBase(0));
    }

    Ok(TasteConfig {
        palette,
        type_scale: build_type_scale(),
        spacing: build_spacing(base),
        animation: build_animation(),
        component_styles: build_component_styles(),
    })
}

// ─── Palette construction ──────────────────────────────────────────────────

fn build_palette(name: &str) -> Result<PaletteDefinition, TasteError> {
    match name {
        "default" => Ok(palette_default()),
        "slate-blue" => Ok(palette_slate_blue()),
        "midnight" => Ok(palette_midnight()),
        other => Err(TasteError::UnknownPalette(other.to_string())),
    }
}

fn palette_default() -> PaletteDefinition {
    let primary = "#2563eb";
    PaletteDefinition {
        name: "default".into(),
        description: "Professional blue \u{2014} trust, precision. Think GitHub.".into(),
        vibe: vec!["professional".into(), "trustworthy".into(), "clean".into()],
        reference: "GitHub".into(),
        primary: primary.into(),
        secondary: "#475569".into(),
        accent: "#7c3aed".into(),
        neutral: "#64748b".into(),
        semantic: light_semantic_colors(),
        shades: generate_shades(primary),
        dark_mode: build_dark_mode(primary),
    }
}

fn palette_slate_blue() -> PaletteDefinition {
    let primary = "#1e40af";
    PaletteDefinition {
        name: "slate-blue".into(),
        description: "Dark, data-dense. Think Linear.".into(),
        vibe: vec!["technical".into(), "data-focused".into()],
        reference: "Linear".into(),
        primary: primary.into(),
        secondary: "#334155".into(),
        accent: "#6d28d9".into(),
        neutral: "#475569".into(),
        semantic: light_semantic_colors(),
        shades: generate_shades(primary),
        dark_mode: build_dark_mode(primary),
    }
}

fn palette_midnight() -> PaletteDefinition {
    let primary = "#4f46e5";
    let accent = "#db2777";
    PaletteDefinition {
        name: "midnight".into(),
        description: "Contemporary indigo. Think Figma.".into(),
        vibe: vec!["modern".into(), "creative".into()],
        reference: "Figma".into(),
        primary: primary.into(),
        secondary: "#6366f1".into(),
        accent: accent.into(),
        neutral: "#475569".into(),
        semantic: light_semantic_colors(),
        shades: generate_shades(primary),
        dark_mode: build_dark_mode(primary),
    }
}

// ─── Semantic colors ───────────────────────────────────────────────────────

/// WCAG AA compliant semantic colors for light backgrounds.
fn light_semantic_colors() -> SemanticColors {
    SemanticColors {
        success: "#15803d".into(), // green-700
        warning: "#a16207".into(), // yellow-700
        error: "#b91c1c".into(),   // red-700
        info: "#1d4ed8".into(),    // blue-700
    }
}

/// WCAG AA compliant semantic colors for dark backgrounds.
fn dark_semantic_colors() -> SemanticColors {
    SemanticColors {
        success: "#4ade80".into(), // green-400
        warning: "#facc15".into(), // yellow-400
        error: "#f87171".into(),   // red-400
        info: "#60a5fa".into(),    // blue-400
    }
}

// ─── Dark mode ─────────────────────────────────────────────────────────────

fn build_dark_mode(primary_hex: &str) -> DarkModeColors {
    DarkModeColors {
        background: "#0f172a".into(), // slate-900
        foreground: "#f1f5f9".into(), // slate-100
        primary: lighten_hex(primary_hex),
        shades: generate_dark_shades(primary_hex),
        semantic: dark_semantic_colors(),
    }
}

// ─── Shade generation ──────────────────────────────────────────────────────

/// Generate 10 shades by varying HSL lightness from 0.95 (lightest) to 0.15 (darkest).
fn generate_shades(hex: &str) -> Vec<String> {
    let (h, s, _) = hex_to_hsl(hex);
    let lightness_values = [0.95, 0.90, 0.82, 0.73, 0.62, 0.50, 0.40, 0.32, 0.23, 0.15];
    lightness_values
        .iter()
        .map(|&l| hsl_to_hex(h, s, l))
        .collect()
}

/// Generate 10 dark-mode shades (inverted: lightest at index 9, darkest at 0).
fn generate_dark_shades(hex: &str) -> Vec<String> {
    let (h, s, _) = hex_to_hsl(hex);
    let lightness_values = [0.15, 0.23, 0.32, 0.40, 0.50, 0.62, 0.73, 0.82, 0.90, 0.95];
    lightness_values
        .iter()
        .map(|&l| hsl_to_hex(h, s, l))
        .collect()
}

/// Lighten a hex color for dark mode primary (bump lightness by ~0.2, cap at 0.85).
fn lighten_hex(hex: &str) -> String {
    let (h, s, l) = hex_to_hsl(hex);
    let new_l = (l + 0.2).min(0.85);
    hsl_to_hex(h, s, new_l)
}

// ─── Color math ────────────────────────────────────────────────────────────

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn hex_to_hsl(hex: &str) -> (f64, f64, f64) {
    let (r, g, b) = hex_to_rgb(hex);
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - rf).abs() < f64::EPSILON {
        let mut h = (gf - bf) / d;
        if gf < bf {
            h += 6.0;
        }
        h
    } else if (max - gf).abs() < f64::EPSILON {
        (bf - rf) / d + 2.0
    } else {
        (rf - gf) / d + 4.0
    };

    (h * 60.0, s, l)
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let (r, g, b) = hsl_to_rgb(h, s, l);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

// ─── Type scale ────────────────────────────────────────────────────────────

/// Major Third (1.25) type scale, 7 levels, body at 16px.
/// Levels: h1, h2, h3, h4, body, small, caption (largest to smallest).
fn build_type_scale() -> TypeScaleConfig {
    let ratio = 1.25_f64;
    let body_size = 16.0_f64;
    let levels = 7_usize;

    // body is at index 4 (0-indexed). Indices 0..3 are headings (scale up),
    // index 5 is small (scale down once), index 6 is caption (scale down twice).
    let mut sizes = Vec::with_capacity(levels);
    for i in 0..levels {
        let exponent = 4i32 - i as i32; // h1=+4, h2=+3, h3=+2, h4=+1, body=0, small=-1, caption=-2
        let size = body_size * ratio.powi(exponent);
        // Round to 2 decimal places
        sizes.push((size * 100.0).round() / 100.0);
    }

    // Line heights: tighter for headings, looser for body/small/caption
    let line_heights = vec![1.1, 1.15, 1.2, 1.25, 1.5, 1.5, 1.5];

    // Font weights: heavier for headings, normal for body, lighter for small/caption
    let font_weights = vec![700, 700, 600, 600, 400, 400, 400];

    TypeScaleConfig {
        ratio,
        levels,
        body_size,
        sizes,
        line_heights,
        font_weights,
        font_family: "Inter, system-ui, sans-serif".into(),
    }
}

// ─── Spacing ───────────────────────────────────────────────────────────────

/// Generate 8 spacing steps from a base, using ratio 1.5, all rounded to
/// nearest multiple of the base.
fn build_spacing(base: usize) -> SpacingConfig {
    let ratio = 1.5_f64;
    let mut steps = Vec::with_capacity(8);
    let mut current = base as f64;
    for _ in 0..8 {
        // Round to nearest multiple of base
        let rounded = ((current / base as f64).round() as usize) * base;
        // Ensure minimum of base and monotonically increasing
        let value = if steps.is_empty() {
            rounded.max(base)
        } else {
            let prev = *steps.last().unwrap();
            rounded.max(prev + base)
        };
        steps.push(value);
        current *= ratio;
    }

    SpacingConfig { base, ratio, steps }
}

// ─── Animation ─────────────────────────────────────────────────────────────

fn build_animation() -> AnimationConfig {
    AnimationConfig {
        transition_duration_ms: 200,
        easing: "ease-out".into(),
    }
}

// ─── Component styles ──────────────────────────────────────────────────────

fn build_component_styles() -> ComponentStyleConfig {
    ComponentStyleConfig {
        button_border_radius: 6,
        card_border_radius: 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_hsl_round_trip() {
        // Blue #2563eb → HSL → hex should be close to original
        let (h, s, l) = hex_to_hsl("#2563eb");
        let result = hsl_to_hex(h, s, l);
        assert_eq!(result, "#2563eb");
    }

    #[test]
    fn hsl_white_and_black() {
        // White
        let (_, _, l) = hex_to_hsl("#ffffff");
        assert!((l - 1.0).abs() < 0.01);
        // Black
        let (_, _, l) = hex_to_hsl("#000000");
        assert!(l.abs() < 0.01);
    }

    #[test]
    fn shade_count() {
        let shades = generate_shades("#2563eb");
        assert_eq!(shades.len(), 10);
        // First shade should be lightest
        let (_, _, l_first) = hex_to_hsl(&shades[0]);
        let (_, _, l_last) = hex_to_hsl(&shades[9]);
        assert!(l_first > l_last, "first shade should be lighter than last");
    }

    #[test]
    fn spacing_all_multiples_of_base() {
        let sp = build_spacing(4);
        for step in &sp.steps {
            assert!(step % 4 == 0, "step {} not a multiple of 4", step);
        }
    }

    #[test]
    fn spacing_monotonically_increasing() {
        let sp = build_spacing(4);
        for i in 0..sp.steps.len() - 1 {
            assert!(sp.steps[i] < sp.steps[i + 1]);
        }
    }

    #[test]
    fn type_scale_body_at_16() {
        let ts = build_type_scale();
        // Body is at index 4
        assert!((ts.sizes[4] - 16.0).abs() < 0.01);
    }
}
