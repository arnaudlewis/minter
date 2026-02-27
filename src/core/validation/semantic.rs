use crate::model::*;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub rule: &'static str,
    pub message: String,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.rule, self.message)
    }
}

/// Run all semantic validation rules on a parsed Spec.
/// Returns Ok(()) if valid, Err with all violations if invalid.
pub fn validate(spec: &Spec) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();

    check_kebab_case_name(spec, &mut errors);
    check_valid_semver(spec, &mut errors);
    check_unique_behavior_names(spec, &mut errors);
    check_at_least_one_happy_path(spec, &mut errors);

    check_nfr_containment(spec, &mut errors);

    for behavior in &spec.behaviors {
        check_unique_aliases(behavior, &mut errors);
        check_alias_refs_resolve(behavior, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_kebab_case_name(spec: &Spec, errors: &mut Vec<SemanticError>) {
    if !crate::model::is_kebab_case(&spec.name) {
        errors.push(SemanticError {
            rule: "kebab-case-name",
            message: format!("Spec name '{}' is not valid kebab-case", spec.name),
        });
    }
}

fn check_valid_semver(spec: &Spec, errors: &mut Vec<SemanticError>) {
    if !crate::model::is_valid_semver(&spec.version) {
        errors.push(SemanticError {
            rule: "valid-semver",
            message: format!("Version '{}' is not valid semver", spec.version),
        });
    }
}

fn check_unique_behavior_names(spec: &Spec, errors: &mut Vec<SemanticError>) {
    let mut seen = HashSet::new();
    for b in &spec.behaviors {
        if !seen.insert(&b.name) {
            errors.push(SemanticError {
                rule: "unique-behavior-names",
                message: format!("Duplicate behavior name '{}'", b.name),
            });
        }
    }
}

fn check_at_least_one_happy_path(spec: &Spec, errors: &mut Vec<SemanticError>) {
    let has_happy = spec
        .behaviors
        .iter()
        .any(|b| b.category == BehaviorCategory::HappyPath);
    if !has_happy {
        errors.push(SemanticError {
            rule: "at-least-one-happy-path",
            message: "Spec must have at least one happy_path behavior".to_string(),
        });
    }
}

fn check_nfr_containment(spec: &Spec, errors: &mut Vec<SemanticError>) {
    if spec.nfr_refs.is_empty() {
        return;
    }
    let spec_categories = spec.nfr_categories();
    for behavior in &spec.behaviors {
        for nfr_ref in &behavior.nfr_refs {
            if !spec_categories.contains(&nfr_ref.category) {
                errors.push(SemanticError {
                    rule: "nfr-containment",
                    message: format!(
                        "Behavior '{}' references NFR category '{}' which is not declared in the spec-level nfr section",
                        behavior.name, nfr_ref.category
                    ),
                });
            }
        }
    }
}

fn check_unique_aliases(behavior: &Behavior, errors: &mut Vec<SemanticError>) {
    let mut seen = HashSet::new();
    for pre in &behavior.preconditions {
        if let Precondition::Alias { name, .. } = pre
            && !seen.insert(name)
        {
            errors.push(SemanticError {
                rule: "unique-aliases",
                message: format!("Duplicate alias '{}' in behavior '{}'", name, behavior.name),
            });
        }
    }
}

fn check_alias_refs_resolve(behavior: &Behavior, errors: &mut Vec<SemanticError>) {
    let declared: HashSet<&str> = behavior
        .preconditions
        .iter()
        .filter_map(|p| match p {
            Precondition::Alias { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    // Check when-section alias refs
    for input in &behavior.action.inputs {
        if let ActionInput::AliasRef { alias, .. } = input
            && !declared.contains(alias.as_str())
        {
            errors.push(SemanticError {
                rule: "alias-refs-resolve",
                message: format!(
                    "Alias '{}' referenced in when of '{}' is not declared in given",
                    alias, behavior.name
                ),
            });
        }
    }

    // Check then-section alias refs
    for post in &behavior.postconditions {
        for assertion in &post.assertions {
            if let Assertion::EqualsRef { alias, .. } = assertion
                && !declared.contains(alias.as_str())
            {
                errors.push(SemanticError {
                    rule: "alias-refs-resolve",
                    message: format!(
                        "Alias '{}' referenced in then of '{}' is not declared in given",
                        alias, behavior.name
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "semantic.test.rs"]
mod tests;
