use super::response::NextStep;

pub fn after_validate_pass() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "Assess spec quality before writing tests".to_string(),
            tool: Some("assess".to_string()),
            params: None,
        },
        NextStep {
            action: "Review coverage tag format".to_string(),
            tool: Some("guide".to_string()),
            params: Some(serde_json::json!({"topic": "coverage"})),
        },
        NextStep {
            action: "Write one e2e test per behavior — tests must fail before implementation"
                .to_string(),
            tool: None,
            params: None,
        },
    ]
}

pub fn after_validate_fail() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "Fix the errors listed above using the fix suggestions".to_string(),
            tool: None,
            params: None,
        },
        NextStep {
            action: "Re-validate after fixing".to_string(),
            tool: Some("validate".to_string()),
            params: None,
        },
    ]
}

pub fn after_scaffold_fr() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "fill in behaviors for each user-observable outcome".to_string(),
            tool: None,
            params: None,
        },
        NextStep {
            action: "validate the spec after adding behaviors".to_string(),
            tool: Some("validate".to_string()),
            params: None,
        },
    ]
}

pub fn after_scaffold_nfr() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "define metric or rule constraints".to_string(),
            tool: None,
            params: None,
        },
        NextStep {
            action: "reference from functional specs using nfr section".to_string(),
            tool: None,
            params: None,
        },
    ]
}

pub fn after_format() -> Vec<NextStep> {
    vec![NextStep {
        action: "use this grammar to write your spec".to_string(),
        tool: Some("scaffold".to_string()),
        params: None,
    }]
}

pub fn after_inspect(has_error_case: bool) -> Vec<NextStep> {
    if has_error_case {
        vec![NextStep {
            action: "review behavior coverage and add missing edge cases".to_string(),
            tool: None,
            params: None,
        }]
    } else {
        vec![NextStep {
            action: "add error_case behaviors for each happy path".to_string(),
            tool: None,
            params: None,
        }]
    }
}

pub fn after_graph() -> Vec<NextStep> {
    vec![NextStep {
        action: "review impacted specs when changing a dependency".to_string(),
        tool: None,
        params: None,
    }]
}

pub fn after_list_specs() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "inspect a specific spec for full details".to_string(),
            tool: Some("inspect".to_string()),
            params: None,
        },
        NextStep {
            action: "validate specs to check for errors".to_string(),
            tool: Some("validate".to_string()),
            params: None,
        },
        NextStep {
            action: "use graph to visualize dependencies".to_string(),
            tool: Some("graph".to_string()),
            params: None,
        },
    ]
}

pub fn after_list_nfrs() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "reference NFR constraints in your spec's nfr section".to_string(),
            tool: None,
            params: None,
        },
        NextStep {
            action: "learn NFR design patterns".to_string(),
            tool: Some("guide".to_string()),
            params: Some(serde_json::json!({"topic": "nfr"})),
        },
    ]
}

pub fn after_search() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "inspect matching specs for full details".to_string(),
            tool: Some("inspect".to_string()),
            params: None,
        },
        NextStep {
            action: "use the results to find dependencies for your spec".to_string(),
            tool: None,
            params: None,
        },
    ]
}

pub fn after_assess() -> Vec<NextStep> {
    vec![
        NextStep {
            action: "fix any smells or coverage gaps identified above".to_string(),
            tool: None,
            params: None,
        },
        NextStep {
            action: "re-validate after making changes".to_string(),
            tool: Some("validate".to_string()),
            params: None,
        },
        NextStep {
            action: "write one e2e test per behavior — tests must fail before implementation"
                .to_string(),
            tool: None,
            params: None,
        },
    ]
}
