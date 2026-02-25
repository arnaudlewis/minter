use super::*;

// ── Helpers ─────────────────────────────────────────────

fn valid_spec() -> Spec {
    Spec {
        name: "test-spec".to_string(),
        version: "1.0.0".to_string(),
        title: "Test".to_string(),
        description: "A test.".to_string(),
        motivation: "Testing.".to_string(),
        nfr_refs: vec![],
        behaviors: vec![happy_behavior("do-thing")],
        dependencies: vec![],
    }
}

fn happy_behavior(name: &str) -> Behavior {
    Behavior {
        name: name.to_string(),
        category: BehaviorCategory::HappyPath,
        description: "Do it".to_string(),
        nfr_refs: vec![],
        preconditions: vec![Precondition::Prose("Ready".to_string())],
        action: Action { name: "act".to_string(), inputs: vec![] },
        postconditions: vec![Postcondition {
            kind: PostconditionKind::Emits("stdout".to_string()),
            assertions: vec![Assertion::Contains {
                field: "output".to_string(),
                value: "done".to_string(),
            }],
        }],
    }
}

fn error_behavior(name: &str) -> Behavior {
    Behavior {
        name: name.to_string(),
        category: BehaviorCategory::ErrorCase,
        description: "Fail".to_string(),
        nfr_refs: vec![],
        preconditions: vec![Precondition::Prose("Ready".to_string())],
        action: Action { name: "act".to_string(), inputs: vec![] },
        postconditions: vec![Postcondition {
            kind: PostconditionKind::Emits("stderr".to_string()),
            assertions: vec![Assertion::Contains {
                field: "output".to_string(),
                value: "error".to_string(),
            }],
        }],
    }
}

// ═══════════════════════════════════════════════════════════════
// Rule: Behavior names are unique
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_unique_behavior_names() {
    let mut spec = valid_spec();
    spec.behaviors.push(error_behavior("other-thing"));
    assert!(validate(&spec).is_ok());
}

#[test]
fn reject_duplicate_behavior_names() {
    let mut spec = valid_spec();
    spec.behaviors.push(happy_behavior("do-thing"));
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("do-thing")));
}

// ═══════════════════════════════════════════════════════════════
// Rule: Aliases unique within each behavior
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_unique_aliases() {
    let mut spec = valid_spec();
    spec.behaviors[0].preconditions = vec![
        Precondition::Alias { name: "user".into(), entity: "User".into(), properties: vec![] },
        Precondition::Alias { name: "note".into(), entity: "Note".into(), properties: vec![] },
    ];
    assert!(validate(&spec).is_ok());
}

#[test]
fn reject_duplicate_aliases() {
    let mut spec = valid_spec();
    spec.behaviors[0].preconditions = vec![
        Precondition::Alias { name: "user".into(), entity: "User".into(), properties: vec![] },
        Precondition::Alias { name: "user".into(), entity: "User".into(), properties: vec![] },
    ];
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("user")));
}

// ═══════════════════════════════════════════════════════════════
// Rule: Alias references resolve
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_resolved_alias_in_when() {
    let mut spec = valid_spec();
    spec.behaviors[0].preconditions = vec![Precondition::Alias {
        name: "the_user".into(),
        entity: "User".into(),
        properties: vec![("id".into(), "123".into())],
    }];
    spec.behaviors[0].action.inputs = vec![ActionInput::AliasRef {
        name: "user_id".into(),
        alias: "the_user".into(),
        field: "id".into(),
    }];
    assert!(validate(&spec).is_ok());
}

#[test]
fn reject_unresolved_alias_in_when() {
    let mut spec = valid_spec();
    spec.behaviors[0].action.inputs = vec![ActionInput::AliasRef {
        name: "user_id".into(),
        alias: "nonexistent".into(),
        field: "id".into(),
    }];
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("nonexistent")));
}

#[test]
fn reject_unresolved_alias_in_then() {
    let mut spec = valid_spec();
    spec.behaviors[0].postconditions[0].assertions = vec![Assertion::EqualsRef {
        field: "created_by".into(),
        alias: "nonexistent".into(),
        alias_field: "id".into(),
    }];
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("nonexistent")));
}

// ═══════════════════════════════════════════════════════════════
// Rule: Valid semver
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_valid_semver() {
    assert!(validate(&valid_spec()).is_ok());
}

#[test]
fn reject_invalid_semver() {
    let mut spec = valid_spec();
    spec.version = "NOPE".into();
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("semver") || e.message.contains("NOPE")));
}

// ═══════════════════════════════════════════════════════════════
// Rule: Kebab-case name
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_kebab_case_name() {
    assert!(validate(&valid_spec()).is_ok());
}

#[test]
fn reject_non_kebab_case_name() {
    let mut spec = valid_spec();
    spec.name = "InvalidName".into();
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("InvalidName") || e.message.contains("kebab")));
}

#[test]
fn reject_name_with_underscore() {
    let mut spec = valid_spec();
    spec.name = "test_spec".into();
    let errors = validate(&spec).unwrap_err();
    assert!(!errors.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Rule: At least one happy_path
// ═══════════════════════════════════════════════════════════════

#[test]
fn accept_spec_with_happy_path() {
    assert!(validate(&valid_spec()).is_ok());
}

#[test]
fn reject_no_happy_path() {
    let mut spec = valid_spec();
    spec.behaviors = vec![error_behavior("fail-thing")];
    let errors = validate(&spec).unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("happy_path")));
}

// ═══════════════════════════════════════════════════════════════
// Multiple errors reported together
// ═══════════════════════════════════════════════════════════════

#[test]
fn report_all_errors() {
    let mut spec = valid_spec();
    spec.name = "InvalidName".into();
    spec.version = "NOPE".into();
    spec.behaviors = vec![error_behavior("fail-thing")];
    let errors = validate(&spec).unwrap_err();
    assert!(
        errors.len() >= 3,
        "Expected at least 3 errors, got {}: {:?}",
        errors.len(),
        errors
    );
}
