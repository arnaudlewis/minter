pub fn after_validate_pass() -> Vec<&'static str> {
    vec![
        "write one e2e test per behavior",
        "tests must fail (red) before implementation",
    ]
}

pub fn after_validate_fail() -> Vec<&'static str> {
    vec!["fix the errors listed above", "re-validate"]
}

pub fn after_scaffold_fr() -> Vec<&'static str> {
    vec![
        "fill in behaviors for each user-observable outcome",
        "validate the spec with the validate tool",
    ]
}

pub fn after_scaffold_nfr() -> Vec<&'static str> {
    vec![
        "define metric or rule constraints",
        "reference from functional specs using nfr section",
    ]
}

pub fn after_format() -> Vec<&'static str> {
    vec!["use this grammar to write your spec"]
}

pub fn after_inspect(has_error_case: bool) -> Vec<&'static str> {
    if has_error_case {
        vec!["review behavior coverage and add missing edge cases"]
    } else {
        vec!["add error_case behaviors for each happy path"]
    }
}

pub fn after_graph() -> Vec<&'static str> {
    vec!["review impacted specs when changing a dependency"]
}

pub fn after_list_specs() -> Vec<&'static str> {
    vec![
        "inspect a specific spec for full details",
        "validate specs to check for errors",
        "use graph to visualize dependencies",
    ]
}

pub fn after_list_nfrs() -> Vec<&'static str> {
    vec![
        "reference NFR constraints in your spec's nfr section",
        "use guide topic 'nfr' for NFR design patterns",
    ]
}

pub fn after_search() -> Vec<&'static str> {
    vec![
        "inspect matching specs for full details",
        "use the results to find dependencies for your spec",
    ]
}
