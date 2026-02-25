use crate::model::NfrSpec;
use crate::semantic::SemanticError;
use std::collections::HashSet;

/// Run semantic validation rules on a parsed NfrSpec.
pub fn validate(nfr: &NfrSpec) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();

    check_valid_semver(nfr, &mut errors);
    check_unique_constraint_names(nfr, &mut errors);
    check_kebab_case_constraint_names(nfr, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_valid_semver(nfr: &NfrSpec, errors: &mut Vec<SemanticError>) {
    if semver::Version::parse(&nfr.version).is_err() {
        errors.push(SemanticError {
            rule: "valid-semver",
            message: format!("Version '{}' is not valid semver", nfr.version),
        });
    }
}

fn check_unique_constraint_names(nfr: &NfrSpec, errors: &mut Vec<SemanticError>) {
    let mut seen = HashSet::new();
    for c in &nfr.constraints {
        if !seen.insert(&c.name) {
            errors.push(SemanticError {
                rule: "unique-constraint-names",
                message: format!("Duplicate constraint name '{}'", c.name),
            });
        }
    }
}

fn check_kebab_case_constraint_names(nfr: &NfrSpec, errors: &mut Vec<SemanticError>) {
    for c in &nfr.constraints {
        let name = &c.name;
        let is_kebab = !name.is_empty()
            && name
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--");

        if !is_kebab {
            errors.push(SemanticError {
                rule: "kebab-case-constraint-name",
                message: format!(
                    "Constraint name '{}' is not valid kebab-case",
                    name
                ),
            });
        }
    }
}
