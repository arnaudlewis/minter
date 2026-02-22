/// The complete parsed spec.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub name: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub motivation: String,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub spec_name: String,
    pub version_constraint: String,
}
