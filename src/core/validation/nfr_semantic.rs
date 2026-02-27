use super::semantic::SemanticError;
use crate::model::NfrSpec;
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
    if !crate::model::is_valid_semver(&nfr.version) {
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
        if !crate::model::is_kebab_case(&c.name) {
            errors.push(SemanticError {
                rule: "kebab-case-constraint-name",
                message: format!("Constraint name '{}' is not valid kebab-case", c.name),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn valid_nfr() -> NfrSpec {
        NfrSpec {
            category: "performance".to_string(),
            version: "1.0.0".to_string(),
            title: "Performance NFR".to_string(),
            description: "Performance requirements".to_string(),
            motivation: "Ensure fast responses".to_string(),
            constraints: vec![NfrConstraint {
                name: "api-response-time".to_string(),
                constraint_type: ConstraintType::Metric,
                description: "Response time limit".to_string(),
                body: ConstraintBody::Metric {
                    metric: "p95-latency".to_string(),
                    threshold_operator: "<".to_string(),
                    threshold_value: "500ms".to_string(),
                    verification: MetricVerification {
                        environments: vec!["staging".to_string()],
                        benchmarks: vec!["load-test".to_string()],
                        datasets: vec!["standard".to_string()],
                        passes: vec!["3-of-5".to_string()],
                    },
                },
                violation: "Response too slow".to_string(),
                overridable: true,
            }],
        }
    }

    fn make_constraint(name: &str) -> NfrConstraint {
        NfrConstraint {
            name: name.to_string(),
            constraint_type: ConstraintType::Metric,
            description: "test".to_string(),
            body: ConstraintBody::Metric {
                metric: "test-metric".to_string(),
                threshold_operator: "<".to_string(),
                threshold_value: "100ms".to_string(),
                verification: MetricVerification {
                    environments: vec![],
                    benchmarks: vec![],
                    datasets: vec![],
                    passes: vec![],
                },
            },
            violation: "test violation".to_string(),
            overridable: false,
        }
    }

    #[test]
    /// nfr_semantic: accept_valid_nfr
    fn accept_valid_nfr() {
        let result = validate(&valid_nfr());
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    }

    #[test]
    /// nfr_semantic: reject_invalid_semver
    fn reject_invalid_semver() {
        let mut nfr = valid_nfr();
        nfr.version = "NOPE".to_string();
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("semver")));
    }

    #[test]
    /// nfr_semantic: reject_missing_patch_version
    fn reject_missing_patch_version() {
        let mut nfr = valid_nfr();
        nfr.version = "1.0".to_string();
        let result = validate(&nfr);
        assert!(result.is_err());
    }

    #[test]
    /// nfr_semantic: accept_unique_constraint_names
    fn accept_unique_constraint_names() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![
            make_constraint("api-response-time"),
            make_constraint("db-query-time"),
        ];
        let result = validate(&nfr);
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    }

    #[test]
    /// nfr_semantic: reject_duplicate_constraint_names
    fn reject_duplicate_constraint_names() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![
            make_constraint("api-response-time"),
            make_constraint("api-response-time"),
        ];
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("Duplicate")));
    }

    #[test]
    /// nfr_semantic: accept_kebab_case_constraint
    fn accept_kebab_case_constraint() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![make_constraint("api-response-time")];
        let result = validate(&nfr);
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    }

    #[test]
    /// nfr_semantic: reject_uppercase_constraint
    fn reject_uppercase_constraint() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![make_constraint("ApiResponseTime")];
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("kebab")));
    }

    #[test]
    /// nfr_semantic: reject_underscore_constraint
    fn reject_underscore_constraint() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![make_constraint("api_response_time")];
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("kebab")));
    }

    #[test]
    /// nfr_semantic: reject_leading_hyphen_constraint
    fn reject_leading_hyphen_constraint() {
        let mut nfr = valid_nfr();
        nfr.constraints = vec![make_constraint("-api")];
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("kebab")));
    }

    #[test]
    /// nfr_semantic: report_all_errors
    fn report_all_errors() {
        let mut nfr = valid_nfr();
        nfr.version = "NOPE".to_string();
        nfr.constraints = vec![
            make_constraint("api-response-time"),
            make_constraint("api-response-time"),
        ];
        let result = validate(&nfr);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.len() >= 2,
            "Expected at least 2 errors but got {}: {:?}",
            errors.len(),
            errors
        );
    }
}
