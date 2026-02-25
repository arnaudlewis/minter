use std::collections::HashMap;

use crate::model::{
    BehaviorNfrRef, ConstraintBody, ConstraintType, NfrSpec, Spec,
};

#[derive(Debug)]
pub struct CrossRefError {
    pub message: String,
}

impl std::fmt::Display for CrossRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Cross-validate a spec's NFR references against the resolved NFR specs.
/// `nfr_specs` maps category name -> parsed NfrSpec.
pub fn cross_validate(
    spec: &Spec,
    nfr_specs: &HashMap<String, NfrSpec>,
) -> Result<(), Vec<CrossRefError>> {
    let mut errors = Vec::new();

    // Collect all NFR refs: spec-level + behavior-level
    for nfr_ref in &spec.nfr_refs {
        check_category_exists(&nfr_ref.category, nfr_specs, &mut errors);
        if let Some(anchor) = &nfr_ref.anchor {
            check_anchor_exists(&nfr_ref.category, anchor, nfr_specs, &mut errors);
        }
    }

    for behavior in &spec.behaviors {
        for nfr_ref in &behavior.nfr_refs {
            check_category_exists(&nfr_ref.category, nfr_specs, &mut errors);
            check_anchor_exists(&nfr_ref.category, &nfr_ref.anchor, nfr_specs, &mut errors);

            if nfr_ref.override_operator.is_some() {
                check_override(nfr_ref, nfr_specs, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_category_exists(
    category: &str,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    if !nfr_specs.contains_key(category) {
        errors.push(CrossRefError {
            message: format!(
                "NFR category '{}' not found — no .nfr file declares this category",
                category
            ),
        });
    }
}

fn check_anchor_exists(
    category: &str,
    anchor: &str,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    if let Some(nfr) = nfr_specs.get(category) {
        let constraint_exists = nfr.constraints.iter().any(|c| c.name == anchor);
        if !constraint_exists {
            errors.push(CrossRefError {
                message: format!(
                    "Constraint '{}' not found in NFR category '{}'",
                    anchor, category
                ),
            });
        }
    }
    // If category doesn't exist, that error is already reported by check_category_exists
}

fn check_override(
    nfr_ref: &BehaviorNfrRef,
    nfr_specs: &HashMap<String, NfrSpec>,
    errors: &mut Vec<CrossRefError>,
) {
    let nfr = match nfr_specs.get(&nfr_ref.category) {
        Some(n) => n,
        None => return, // category missing error already reported
    };

    let constraint = match nfr.constraints.iter().find(|c| c.name == nfr_ref.anchor) {
        Some(c) => c,
        None => return, // anchor missing error already reported
    };

    let override_op = match &nfr_ref.override_operator {
        Some(op) => op.as_str(),
        None => return,
    };
    let override_val = match &nfr_ref.override_value {
        Some(v) => v.as_str(),
        None => return,
    };

    // Rule 4: overridable must be yes
    if !constraint.overridable {
        errors.push(CrossRefError {
            message: format!(
                "Constraint '{}' in '{}' is not overridable",
                nfr_ref.anchor, nfr_ref.category
            ),
        });
        return;
    }

    // Rule 5: must be a metric constraint (not rule)
    if constraint.constraint_type == ConstraintType::Rule {
        errors.push(CrossRefError {
            message: format!(
                "Cannot override rule constraint '{}' in '{}' — only metric constraints support overrides",
                nfr_ref.anchor, nfr_ref.category
            ),
        });
        return;
    }

    // Rule 6: operator must match the original threshold
    let (original_op, original_val) = match &constraint.body {
        ConstraintBody::Metric {
            threshold_operator,
            threshold_value,
            ..
        } => (threshold_operator.as_str(), threshold_value.as_str()),
        ConstraintBody::Rule { .. } => return,
    };

    if override_op != original_op {
        errors.push(CrossRefError {
            message: format!(
                "Override operator '{}' for '{}' does not match original threshold operator '{}'",
                override_op, nfr_ref.anchor, original_op
            ),
        });
        return;
    }

    // Rule 7: value must be stricter than the default
    let original_norm = normalize_value(original_val);
    let override_norm = normalize_value(override_val);

    if let (Some(orig), Some(ovr)) = (original_norm, override_norm) {
        let is_stricter = match override_op {
            "<" | "<=" => ovr < orig,
            ">" | ">=" => ovr > orig,
            "==" => (ovr - orig).abs() < f64::EPSILON,
            _ => true,
        };

        if !is_stricter {
            errors.push(CrossRefError {
                message: format!(
                    "Override value '{}' for '{}' is not stricter than the default '{}'",
                    override_val, nfr_ref.anchor, original_val
                ),
            });
        }
    }
}

/// Normalize a threshold value with optional unit to a canonical f64.
/// Supports: ms, s, %, KB, MB, GB, or bare numbers.
fn normalize_value(val: &str) -> Option<f64> {
    let val = val.trim();

    if let Some(num) = val.strip_suffix("ms") {
        num.trim().parse::<f64>().ok()
    } else if let Some(num) = val.strip_suffix('s') {
        num.trim().parse::<f64>().ok().map(|v| v * 1000.0) // convert to ms
    } else if let Some(num) = val.strip_suffix('%') {
        num.trim().parse::<f64>().ok()
    } else if let Some(num) = val.strip_suffix("GB") {
        num.trim().parse::<f64>().ok().map(|v| v * 1_000_000.0) // convert to KB
    } else if let Some(num) = val.strip_suffix("MB") {
        num.trim().parse::<f64>().ok().map(|v| v * 1_000.0) // convert to KB
    } else if let Some(num) = val.strip_suffix("KB") {
        num.trim().parse::<f64>().ok()
    } else {
        val.parse::<f64>().ok()
    }
}
