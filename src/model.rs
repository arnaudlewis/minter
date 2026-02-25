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
    Value { name: String, value: String },
    AliasRef { name: String, alias: String, field: String },
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
    Equals { field: String, value: String },
    EqualsRef { field: String, alias: String, alias_field: String },
    IsPresent { field: String },
    Contains { field: String, value: String },
    InRange { field: String, min: String, max: String },
    MatchesPattern { field: String, pattern: String },
    GreaterOrEqual { field: String, value: String },
    Prose(String),
}

impl Spec {
    pub fn dep_names(&self) -> Vec<String> {
        self.dependencies.iter().map(|d| d.spec_name.clone()).collect()
    }

    pub fn nfr_categories(&self) -> std::collections::HashSet<String> {
        self.nfr_refs.iter().map(|r| r.category.clone()).collect()
    }
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
