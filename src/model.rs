/// The complete parsed spec.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub name: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub motivation: String,
    pub nfr_refs: Vec<NfrRef>,
    pub behaviors: Vec<Behavior>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorCategory {
    HappyPath,
    ErrorCase,
    EdgeCase,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Behavior {
    pub name: String,
    pub category: BehaviorCategory,
    pub description: String,
    pub nfr_refs: Vec<BehaviorNfrRef>,
    pub preconditions: Vec<Precondition>,
    pub action: Action,
    pub postconditions: Vec<Postcondition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Precondition {
    Prose(String),
    Alias {
        name: String,
        entity: String,
        properties: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub name: String,
    pub inputs: Vec<ActionInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionInput {
    Value {
        name: String,
        value: String,
    },
    AliasRef {
        name: String,
        alias: String,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostconditionKind {
    Returns(String),
    Emits(String),
    SideEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Postcondition {
    pub kind: PostconditionKind,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Assertion {
    Equals {
        field: String,
        value: String,
    },
    EqualsRef {
        field: String,
        alias: String,
        alias_field: String,
    },
    IsPresent {
        field: String,
    },
    Contains {
        field: String,
        value: String,
    },
    InRange {
        field: String,
        min: String,
        max: String,
    },
    MatchesPattern {
        field: String,
        pattern: String,
    },
    GreaterOrEqual {
        field: String,
        value: String,
    },
    Prose(String),
}

impl Spec {
    pub fn dep_names(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .map(|d| d.spec_name.clone())
            .collect()
    }

    pub fn nfr_categories(&self) -> std::collections::HashSet<String> {
        self.nfr_refs.iter().map(|r| r.category.clone()).collect()
    }

    pub fn all_nfr_categories(&self) -> Vec<String> {
        let mut cats: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for r in &self.nfr_refs {
            cats.insert(r.category.clone());
        }
        for b in &self.behaviors {
            for r in &b.nfr_refs {
                cats.insert(r.category.clone());
            }
        }
        cats.into_iter().collect()
    }

    /// Returns NFR references grouped by category as `NfrRefKind`.
    /// A whole-file ref (no anchor) anywhere produces `WholeFile`, absorbing any anchors.
    /// Otherwise, produces `Anchors(sorted_deduplicated_vec)`.
    pub fn all_nfr_refs_grouped(&self) -> std::collections::BTreeMap<String, NfrRefKind> {
        let mut map: std::collections::BTreeMap<
            String,
            (bool, std::collections::BTreeSet<String>),
        > = std::collections::BTreeMap::new();
        for r in &self.nfr_refs {
            let entry = map
                .entry(r.category.clone())
                .or_insert((false, std::collections::BTreeSet::new()));
            if let Some(anchor) = &r.anchor {
                entry.1.insert(anchor.clone());
            } else {
                entry.0 = true;
            }
        }
        for b in &self.behaviors {
            for r in &b.nfr_refs {
                let entry = map
                    .entry(r.category.clone())
                    .or_insert((false, std::collections::BTreeSet::new()));
                entry.1.insert(r.anchor.clone());
            }
        }
        map.into_iter()
            .map(|(cat, (has_whole_file, anchors))| {
                let kind = if has_whole_file {
                    NfrRefKind::WholeFile
                } else {
                    NfrRefKind::Anchors(anchors.into_iter().collect())
                };
                (cat, kind)
            })
            .collect()
    }
}

/// Describes how a spec references an NFR category in the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum NfrRefKind {
    WholeFile,
    Anchors(Vec<String>),
}

/// A spec-level NFR reference: whole-file or anchor.
#[derive(Debug, Clone, PartialEq)]
pub struct NfrRef {
    pub category: String,
    pub anchor: Option<String>,
}

/// A behavior-level NFR reference: always anchored, optional override.
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorNfrRef {
    pub category: String,
    pub anchor: String,
    pub override_operator: Option<String>,
    pub override_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub spec_name: String,
    pub version_constraint: String,
}

// ── NFR model types ─────────────────────────────────────

/// The complete parsed NFR spec.
#[derive(Debug, Clone, PartialEq)]
pub struct NfrSpec {
    pub category: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub motivation: String,
    pub constraints: Vec<NfrConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NfrConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub description: String,
    pub body: ConstraintBody,
    pub violation: String,
    pub overridable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    Metric,
    Rule,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintBody {
    Metric {
        metric: String,
        threshold_operator: String,
        threshold_value: String,
        verification: MetricVerification,
    },
    Rule {
        rule_text: String,
        verification: RuleVerification,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricVerification {
    pub environments: Vec<String>,
    pub benchmarks: Vec<String>,
    pub datasets: Vec<String>,
    pub passes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVerification {
    pub statics: Vec<String>,
    pub runtimes: Vec<String>,
}

// ── Shared constants ────────────────────────────────────

pub const VALID_NFR_CATEGORIES: &[&str] = &[
    "performance",
    "reliability",
    "security",
    "observability",
    "scalability",
    "cost",
    "operability",
];

pub const VALID_GUIDE_TOPICS: &[&str] = &[
    "workflow",
    "authoring",
    "smells",
    "nfr",
    "context",
    "methodology",
    "coverage",
    "config",
    "lock",
    "ci",
    "web",
];

/// Guide topics available in the CLI via `minter guide <topic>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum GuideTopic {
    Workflow,
    Authoring,
    Smells,
    Nfr,
    Context,
    Methodology,
    Coverage,
    Config,
    Lock,
    Ci,
    Web,
}

impl GuideTopic {
    pub fn as_str(&self) -> &'static str {
        match self {
            GuideTopic::Workflow => "workflow",
            GuideTopic::Authoring => "authoring",
            GuideTopic::Smells => "smells",
            GuideTopic::Nfr => "nfr",
            GuideTopic::Context => "context",
            GuideTopic::Methodology => "methodology",
            GuideTopic::Coverage => "coverage",
            GuideTopic::Config => "config",
            GuideTopic::Lock => "lock",
            GuideTopic::Ci => "ci",
            GuideTopic::Web => "web",
        }
    }
}

// ── Shared helpers ──────────────────────────────────────

/// Capitalize the first character of a string.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Check if a string is valid kebab-case: lowercase ASCII alphanumeric + hyphens,
/// no leading/trailing/double hyphens.
pub fn is_kebab_case(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
}

/// Check if a string is valid semver.
pub fn is_valid_semver(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_kebab_case ───────────────────────────────────

    #[test]
    fn kebab_case_simple() {
        assert!(is_kebab_case("hello"));
    }

    #[test]
    fn kebab_case_with_hyphens() {
        assert!(is_kebab_case("hello-world"));
    }

    #[test]
    fn kebab_case_with_digits() {
        assert!(is_kebab_case("v2-api"));
    }

    #[test]
    fn kebab_case_reject_empty() {
        assert!(!is_kebab_case(""));
    }

    #[test]
    fn kebab_case_reject_uppercase() {
        assert!(!is_kebab_case("Hello"));
    }

    #[test]
    fn kebab_case_reject_underscore() {
        assert!(!is_kebab_case("hello_world"));
    }

    #[test]
    fn kebab_case_reject_leading_hyphen() {
        assert!(!is_kebab_case("-hello"));
    }

    #[test]
    fn kebab_case_reject_trailing_hyphen() {
        assert!(!is_kebab_case("hello-"));
    }

    #[test]
    fn kebab_case_reject_double_hyphen() {
        assert!(!is_kebab_case("hello--world"));
    }

    #[test]
    fn kebab_case_reject_space() {
        assert!(!is_kebab_case("hello world"));
    }

    // ── is_valid_semver ─────────────────────────────────

    #[test]
    fn semver_valid_basic() {
        assert!(is_valid_semver("1.0.0"));
    }

    #[test]
    fn semver_valid_with_prerelease() {
        assert!(is_valid_semver("1.0.0-alpha.1"));
    }

    #[test]
    fn semver_valid_with_build() {
        assert!(is_valid_semver("1.0.0+build.123"));
    }

    #[test]
    fn semver_reject_missing_patch() {
        assert!(!is_valid_semver("1.0"));
    }

    #[test]
    fn semver_reject_text() {
        assert!(!is_valid_semver("NOPE"));
    }

    #[test]
    fn semver_reject_empty() {
        assert!(!is_valid_semver(""));
    }

    #[test]
    fn semver_reject_v_prefix() {
        assert!(!is_valid_semver("v1.0.0"));
    }

    // ── capitalize ─────────────────────────────────────

    #[test]
    fn capitalize_simple() {
        assert_eq!(capitalize("hello"), "Hello");
    }

    #[test]
    fn capitalize_empty() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn capitalize_already_upper() {
        assert_eq!(capitalize("Hello"), "Hello");
    }

    // ── VALID_GUIDE_TOPICS ────────────────────────────────

    #[test]
    fn guide_topics_contains_all_eleven() {
        assert!(VALID_GUIDE_TOPICS.contains(&"workflow"));
        assert!(VALID_GUIDE_TOPICS.contains(&"authoring"));
        assert!(VALID_GUIDE_TOPICS.contains(&"smells"));
        assert!(VALID_GUIDE_TOPICS.contains(&"nfr"));
        assert!(VALID_GUIDE_TOPICS.contains(&"context"));
        assert!(VALID_GUIDE_TOPICS.contains(&"methodology"));
        assert!(VALID_GUIDE_TOPICS.contains(&"coverage"));
        assert!(VALID_GUIDE_TOPICS.contains(&"config"));
        assert!(VALID_GUIDE_TOPICS.contains(&"lock"));
        assert!(VALID_GUIDE_TOPICS.contains(&"ci"));
        assert!(VALID_GUIDE_TOPICS.contains(&"web"));
        assert_eq!(VALID_GUIDE_TOPICS.len(), 11);
    }
}
