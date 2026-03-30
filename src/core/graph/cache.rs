use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphCache {
    pub schema_version: u32,
    pub specs: HashMap<String, CachedEntry>,
    #[serde(default)]
    pub nfrs: HashMap<String, NfrCachedEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BehaviorSnapshot {
    pub category: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedEntry {
    pub content_hash: String,
    pub version: String,
    pub behavior_count: usize,
    pub valid: bool,
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub nfr_categories: Vec<String>,
    #[serde(default)]
    pub behaviors: HashMap<String, BehaviorSnapshot>,
    #[serde(default)]
    pub baseline: Option<HashMap<String, BehaviorSnapshot>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NfrCachedEntry {
    pub content_hash: String,
    pub version: String,
    pub constraint_count: usize,
}

#[derive(Debug)]
pub enum GraphError {
    Corrupted(String),
    SchemaMismatch,
    Io(io::Error),
}

impl Default for GraphCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphCache {
    pub fn new() -> Self {
        GraphCache {
            schema_version: SCHEMA_VERSION,
            specs: HashMap::new(),
            nfrs: HashMap::new(),
        }
    }

    /// Maximum graph cache file size (50 MB).
    const MAX_CACHE_SIZE: u64 = 50 * 1024 * 1024;

    /// Load a graph cache from a JSON file.
    /// Returns GraphError if the file is corrupted or has wrong schema.
    pub fn load(path: &Path) -> Result<GraphCache, GraphError> {
        let meta = fs::metadata(path).map_err(GraphError::Io)?;
        if meta.len() > Self::MAX_CACHE_SIZE {
            return Err(GraphError::Corrupted(
                "graph cache exceeds 50MB size limit".to_string(),
            ));
        }
        let content = fs::read_to_string(path).map_err(GraphError::Io)?;
        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| GraphError::Corrupted(e.to_string()))?;

        // Check schema_version
        match value.get("schema_version").and_then(|v| v.as_u64()) {
            Some(v) if v as u32 == SCHEMA_VERSION => {}
            Some(_) => return Err(GraphError::SchemaMismatch),
            None => return Err(GraphError::SchemaMismatch),
        }

        // Check that "specs" key exists
        if value.get("specs").is_none() {
            return Err(GraphError::SchemaMismatch);
        }

        let graph: GraphCache =
            serde_json::from_value(value).map_err(|e| GraphError::Corrupted(e.to_string()))?;
        Ok(graph)
    }

    /// Save the graph cache to a JSON file (atomic write via temp file + rename).
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, path)
    }

    /// Insert or update a spec entry in the graph.
    pub fn upsert(&mut self, name: String, entry: CachedEntry) {
        self.specs.insert(name, entry);
    }

    /// Check if a spec's content has changed compared to the cached hash.
    pub fn is_changed(&self, name: &str, current_hash: &str) -> bool {
        match self.specs.get(name) {
            Some(entry) => entry.content_hash != current_hash,
            None => true, // new spec, counts as changed
        }
    }

    /// Snapshot current behaviors as baseline for a named spec.
    pub fn acknowledge_baseline(&mut self, name: &str) {
        if let Some(entry) = self.specs.get_mut(name) {
            entry.baseline = Some(entry.behaviors.clone());
        }
    }

    /// Return the baseline to preserve when updating a cache entry.
    /// Prefers the explicit baseline, falls back to current behaviors.
    pub fn resolve_baseline(&self, name: &str) -> Option<HashMap<String, BehaviorSnapshot>> {
        self.specs
            .get(name)
            .map(|e| e.baseline.clone().unwrap_or_else(|| e.behaviors.clone()))
    }

    /// Check if an NFR file's content has changed compared to the cached hash.
    pub fn is_nfr_changed(&self, category: &str, current_hash: &str) -> bool {
        match self.nfrs.get(category) {
            Some(entry) => entry.content_hash != current_hash,
            None => true,
        }
    }

    /// Insert or update an NFR entry in the graph.
    pub fn upsert_nfr(&mut self, category: String, entry: NfrCachedEntry) {
        self.nfrs.insert(category, entry);
    }
}

fn category_str(cat: crate::model::BehaviorCategory) -> &'static str {
    match cat {
        crate::model::BehaviorCategory::HappyPath => "happy_path",
        crate::model::BehaviorCategory::ErrorCase => "error_case",
        crate::model::BehaviorCategory::EdgeCase => "edge_case",
    }
}

/// Compute a deterministic SHA-256 hash of a behavior's structural content.
pub fn behavior_hash(behavior: &crate::model::Behavior) -> String {
    use std::fmt::Write;

    let mut buf = String::new();

    // Category
    let _ = writeln!(buf, "cat:{}", category_str(behavior.category));

    // Description
    let _ = writeln!(buf, "desc:{}", behavior.description);

    // Preconditions
    for pre in &behavior.preconditions {
        match pre {
            crate::model::Precondition::Prose(text) => {
                let _ = writeln!(buf, "pre:prose:{}", text);
            }
            crate::model::Precondition::Alias {
                name,
                entity,
                properties,
            } => {
                let _ = write!(buf, "pre:alias:{}:{}:", name, entity);
                for (k, v) in properties {
                    let _ = write!(buf, "{}={},", k, v);
                }
                buf.push('\n');
            }
        }
    }

    // Action
    let _ = writeln!(buf, "action:{}", behavior.action.name);
    for input in &behavior.action.inputs {
        match input {
            crate::model::ActionInput::Value { name, value } => {
                let _ = writeln!(buf, "input:val:{}:{}", name, value);
            }
            crate::model::ActionInput::AliasRef { name, alias, field } => {
                let _ = writeln!(buf, "input:ref:{}:{}:{}", name, alias, field);
            }
        }
    }

    // Postconditions
    for post in &behavior.postconditions {
        match &post.kind {
            crate::model::PostconditionKind::Returns(channel) => {
                let _ = writeln!(buf, "post:returns:{}", channel);
            }
            crate::model::PostconditionKind::Emits(channel) => {
                let _ = writeln!(buf, "post:emits:{}", channel);
            }
            crate::model::PostconditionKind::SideEffect => {
                buf.push_str("post:side_effect\n");
            }
        }
        for assertion in &post.assertions {
            match assertion {
                crate::model::Assertion::Equals { field, value } => {
                    let _ = writeln!(buf, "assert:eq:{}:{}", field, value);
                }
                crate::model::Assertion::EqualsRef {
                    field,
                    alias,
                    alias_field,
                } => {
                    let _ = writeln!(buf, "assert:eqref:{}:{}:{}", field, alias, alias_field);
                }
                crate::model::Assertion::IsPresent { field } => {
                    let _ = writeln!(buf, "assert:present:{}", field);
                }
                crate::model::Assertion::Contains { field, value } => {
                    let _ = writeln!(buf, "assert:contains:{}:{}", field, value);
                }
                crate::model::Assertion::InRange { field, min, max } => {
                    let _ = writeln!(buf, "assert:range:{}:{}:{}", field, min, max);
                }
                crate::model::Assertion::MatchesPattern { field, pattern } => {
                    let _ = writeln!(buf, "assert:pattern:{}:{}", field, pattern);
                }
                crate::model::Assertion::GreaterOrEqual { field, value } => {
                    let _ = writeln!(buf, "assert:gte:{}:{}", field, value);
                }
                crate::model::Assertion::Prose(text) => {
                    let _ = writeln!(buf, "assert:prose:{}", text);
                }
            }
        }
    }

    // NFR refs
    for nfr in &behavior.nfr_refs {
        let _ = write!(buf, "nfr:{}:{}", nfr.category, nfr.anchor);
        if let Some(op) = &nfr.override_operator {
            let _ = write!(buf, ":op:{}", op);
        }
        if let Some(val) = &nfr.override_value {
            let _ = write!(buf, ":val:{}", val);
        }
        buf.push('\n');
    }

    let mut hasher = Sha256::new();
    hasher.update(buf.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Build a behaviors map from a parsed Spec.
pub fn compute_behaviors(spec: &crate::model::Spec) -> HashMap<String, BehaviorSnapshot> {
    spec.behaviors
        .iter()
        .map(|b| {
            (
                b.name.clone(),
                BehaviorSnapshot {
                    category: category_str(b.category).to_string(),
                    hash: behavior_hash(b),
                },
            )
        })
        .collect()
}

/// Compute SHA-256 hex digest of source content.
pub fn content_hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the path to graph.json at the current working directory.
pub fn graph_json_path_cwd() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".minter")
        .join("graph.json")
}

pub struct GraphState {
    pub cache: GraphCache,
    pub dirty: bool,
}

impl GraphState {
    pub fn load_or_build() -> Self {
        let graph_path = graph_json_path_cwd();
        if graph_path.exists() {
            match GraphCache::load(&graph_path) {
                Ok(cache) => {
                    return GraphState {
                        cache,
                        dirty: false,
                    };
                }
                Err(GraphError::Corrupted(msg)) => {
                    eprintln!(
                        "warning: cached graph is corrupt ({}), rebuilding from scratch",
                        msg
                    );
                }
                Err(GraphError::SchemaMismatch) => {
                    eprintln!(
                        "warning: cached graph has incompatible format, rebuilding from scratch"
                    );
                }
                Err(GraphError::Io(_)) => {}
            }
        }
        GraphState {
            cache: GraphCache::new(),
            dirty: true,
        }
    }

    pub fn save_if_dirty(&self) {
        if self.dirty {
            let graph_path = graph_json_path_cwd();
            if let Err(e) = self.cache.save(&graph_path) {
                eprintln!("warning: failed to save graph cache: {}", e);
            }
        }
    }

    pub fn prune_stale(&mut self, on_disk: &HashSet<String>) {
        let stale: Vec<String> = self
            .cache
            .specs
            .keys()
            .filter(|name| !on_disk.contains(name.as_str()))
            .cloned()
            .collect();
        for name in stale {
            self.cache.specs.remove(&name);
            self.dirty = true;
        }
    }

    pub fn prune_stale_nfrs(&mut self, nfr_on_disk: &HashSet<String>) {
        let stale: Vec<String> = self
            .cache
            .nfrs
            .keys()
            .filter(|cat| !nfr_on_disk.contains(cat.as_str()))
            .cloned()
            .collect();
        for cat in stale {
            self.cache.nfrs.remove(&cat);
            self.dirty = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_cached_entry(hash: &str) -> CachedEntry {
        CachedEntry {
            content_hash: hash.to_string(),
            version: "1.0.0".to_string(),
            behavior_count: 2,
            valid: true,
            dependencies: vec![],
            path: "test.spec".to_string(),
            nfr_categories: vec![],
            behaviors: HashMap::new(),
            baseline: None,
        }
    }

    fn make_nfr_cached_entry(hash: &str) -> NfrCachedEntry {
        NfrCachedEntry {
            content_hash: hash.to_string(),
            version: "1.0.0".to_string(),
            constraint_count: 1,
        }
    }

    // ── GraphCache basic tests ───────────────────────────

    #[test]
    /// cache: new_cache_has_correct_schema
    fn new_cache_has_correct_schema() {
        let cache = GraphCache::new();
        assert_eq!(cache.schema_version, 4);
    }

    #[test]
    /// cache: new_cache_is_empty
    fn new_cache_is_empty() {
        let cache = GraphCache::new();
        assert!(cache.specs.is_empty());
        assert!(cache.nfrs.is_empty());
    }

    #[test]
    /// cache: upsert_adds_spec
    fn upsert_adds_spec() {
        let mut cache = GraphCache::new();
        cache.upsert("auth".to_string(), make_cached_entry("abc123"));
        assert!(cache.specs.contains_key("auth"));
        assert_eq!(cache.specs["auth"].content_hash, "abc123");
    }

    #[test]
    /// cache: upsert_replaces_spec
    fn upsert_replaces_spec() {
        let mut cache = GraphCache::new();
        cache.upsert("auth".to_string(), make_cached_entry("first"));
        cache.upsert("auth".to_string(), make_cached_entry("second"));
        assert_eq!(cache.specs["auth"].content_hash, "second");
    }

    #[test]
    /// cache: is_changed_new_spec
    fn is_changed_new_spec() {
        let cache = GraphCache::new();
        assert!(cache.is_changed("unknown", "anyhash"));
    }

    #[test]
    /// cache: is_changed_same_hash
    fn is_changed_same_hash() {
        let mut cache = GraphCache::new();
        cache.upsert("auth".to_string(), make_cached_entry("abc123"));
        assert!(!cache.is_changed("auth", "abc123"));
    }

    #[test]
    /// cache: is_changed_different_hash
    fn is_changed_different_hash() {
        let mut cache = GraphCache::new();
        cache.upsert("auth".to_string(), make_cached_entry("abc123"));
        assert!(cache.is_changed("auth", "def456"));
    }

    #[test]
    /// cache: upsert_nfr_adds_entry
    fn upsert_nfr_adds_entry() {
        let mut cache = GraphCache::new();
        cache.upsert_nfr("performance".to_string(), make_nfr_cached_entry("nfrhash"));
        assert!(cache.nfrs.contains_key("performance"));
        assert_eq!(cache.nfrs["performance"].content_hash, "nfrhash");
    }

    #[test]
    /// cache: is_nfr_changed_new
    fn is_nfr_changed_new() {
        let cache = GraphCache::new();
        assert!(cache.is_nfr_changed("unknown", "anyhash"));
    }

    #[test]
    /// cache: is_nfr_changed_same_hash
    fn is_nfr_changed_same_hash() {
        let mut cache = GraphCache::new();
        cache.upsert_nfr("performance".to_string(), make_nfr_cached_entry("nfrhash"));
        assert!(!cache.is_nfr_changed("performance", "nfrhash"));
    }

    #[test]
    /// cache: is_nfr_changed_different_hash
    fn is_nfr_changed_different_hash() {
        let mut cache = GraphCache::new();
        cache.upsert_nfr("performance".to_string(), make_nfr_cached_entry("nfrhash"));
        assert!(cache.is_nfr_changed("performance", "otherhash"));
    }

    // ── save/load round-trip tests ───────────────────────

    #[test]
    /// cache: save_load_round_trip
    fn save_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.json");

        let mut cache = GraphCache::new();
        cache.upsert("auth".to_string(), make_cached_entry("abc123"));
        cache.upsert_nfr("performance".to_string(), make_nfr_cached_entry("nfr999"));
        cache.save(&path).unwrap();

        let loaded = GraphCache::load(&path).unwrap();
        assert_eq!(loaded.schema_version, 4);
        assert!(loaded.specs.contains_key("auth"));
        assert_eq!(loaded.specs["auth"].content_hash, "abc123");
        assert_eq!(loaded.specs["auth"].version, "1.0.0");
        assert_eq!(loaded.specs["auth"].behavior_count, 2);
        assert!(loaded.nfrs.contains_key("performance"));
        assert_eq!(loaded.nfrs["performance"].content_hash, "nfr999");
    }

    #[test]
    /// cache: load_wrong_schema_version
    fn load_wrong_schema_version() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let json = r#"{"schema_version": 99, "specs": {}}"#;
        fs::write(path, json).unwrap();

        let result = GraphCache::load(path);
        assert!(matches!(result, Err(GraphError::SchemaMismatch)));
    }

    #[test]
    /// cache: load_corrupted_json
    fn load_corrupted_json() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        fs::write(path, "not json").unwrap();

        let result = GraphCache::load(path);
        assert!(matches!(result, Err(GraphError::Corrupted(_))));
    }

    #[test]
    /// cache: content_hash_deterministic
    fn content_hash_deterministic() {
        let input = "spec auth\nversion 1.0.0\n";
        let h1 = content_hash(input);
        let h2 = content_hash(input);
        assert_eq!(h1, h2);
    }

    #[test]
    /// cache: content_hash_changes
    fn content_hash_changes() {
        let h1 = content_hash("spec auth\nversion 1.0.0\n");
        let h2 = content_hash("spec auth\nversion 2.0.0\n");
        assert_ne!(h1, h2);
    }

    // ── save/load round-trip with behaviors ─────────────

    #[test]
    /// cache: save_load_round_trip_with_behaviors
    fn save_load_round_trip_with_behaviors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.json");

        let mut cache = GraphCache::new();
        let mut behaviors = HashMap::new();
        behaviors.insert(
            "login-success".to_string(),
            BehaviorSnapshot {
                category: "happy_path".to_string(),
                hash: "abc123".to_string(),
            },
        );
        let mut entry = make_cached_entry("hash1");
        entry.behaviors = behaviors.clone();
        entry.baseline = Some(behaviors.clone());
        cache.upsert("auth".to_string(), entry);
        cache.save(&path).unwrap();

        let loaded = GraphCache::load(&path).unwrap();
        let loaded_entry = &loaded.specs["auth"];
        assert_eq!(loaded_entry.behaviors.len(), 1);
        assert_eq!(
            loaded_entry.behaviors["login-success"],
            BehaviorSnapshot {
                category: "happy_path".to_string(),
                hash: "abc123".to_string(),
            }
        );
        assert!(loaded_entry.baseline.is_some());
        assert_eq!(loaded_entry.baseline.as_ref().unwrap().len(), 1);
    }

    // ── behavior_hash tests ─────────────────────────────

    fn make_test_behavior() -> crate::model::Behavior {
        crate::model::Behavior {
            name: "login-success".to_string(),
            category: crate::model::BehaviorCategory::HappyPath,
            description: "User logs in successfully".to_string(),
            nfr_refs: vec![],
            preconditions: vec![crate::model::Precondition::Prose("user exists".to_string())],
            action: crate::model::Action {
                name: "login".to_string(),
                inputs: vec![crate::model::ActionInput::Value {
                    name: "email".to_string(),
                    value: "test@example.com".to_string(),
                }],
            },
            postconditions: vec![crate::model::Postcondition {
                kind: crate::model::PostconditionKind::Returns("session".to_string()),
                assertions: vec![crate::model::Assertion::IsPresent {
                    field: "token".to_string(),
                }],
            }],
        }
    }

    #[test]
    /// cache: behavior_hash_deterministic
    fn behavior_hash_deterministic() {
        let b = make_test_behavior();
        let h1 = behavior_hash(&b);
        let h2 = behavior_hash(&b);
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }

    #[test]
    /// cache: behavior_hash_changes_on_given
    fn behavior_hash_changes_on_given() {
        let b1 = make_test_behavior();
        let mut b2 = make_test_behavior();
        b2.preconditions = vec![crate::model::Precondition::Prose(
            "user is admin".to_string(),
        )];
        let h1 = behavior_hash(&b1);
        let h2 = behavior_hash(&b2);
        assert_ne!(h1, h2);
    }

    #[test]
    /// cache: behavior_hash_changes_on_description
    fn behavior_hash_changes_on_description() {
        let b1 = make_test_behavior();
        let mut b2 = make_test_behavior();
        b2.description = "Different description".to_string();
        assert_ne!(behavior_hash(&b1), behavior_hash(&b2));
    }

    #[test]
    /// cache: behavior_hash_changes_on_action
    fn behavior_hash_changes_on_action() {
        let b1 = make_test_behavior();
        let mut b2 = make_test_behavior();
        b2.action.name = "logout".to_string();
        assert_ne!(behavior_hash(&b1), behavior_hash(&b2));
    }

    #[test]
    /// cache: behavior_hash_changes_on_postcondition
    fn behavior_hash_changes_on_postcondition() {
        let b1 = make_test_behavior();
        let mut b2 = make_test_behavior();
        b2.postconditions = vec![crate::model::Postcondition {
            kind: crate::model::PostconditionKind::Emits("event".to_string()),
            assertions: vec![],
        }];
        assert_ne!(behavior_hash(&b1), behavior_hash(&b2));
    }

    #[test]
    /// cache: behavior_hash_changes_on_nfr_refs
    fn behavior_hash_changes_on_nfr_refs() {
        let b1 = make_test_behavior();
        let mut b2 = make_test_behavior();
        b2.nfr_refs = vec![crate::model::BehaviorNfrRef {
            category: "performance".to_string(),
            anchor: "api-response-time".to_string(),
            override_operator: None,
            override_value: None,
        }];
        assert_ne!(behavior_hash(&b1), behavior_hash(&b2));
    }

    // ── compute_behaviors tests ─────────────────────────

    #[test]
    /// cache: compute_behaviors_maps_all
    fn compute_behaviors_maps_all() {
        let spec = crate::model::Spec {
            name: "auth".to_string(),
            version: "1.0.0".to_string(),
            title: "Auth".to_string(),
            description: "".to_string(),
            motivation: "".to_string(),
            nfr_refs: vec![],
            behaviors: vec![
                {
                    let mut b = make_test_behavior();
                    b.name = "login-success".to_string();
                    b.category = crate::model::BehaviorCategory::HappyPath;
                    b
                },
                {
                    let mut b = make_test_behavior();
                    b.name = "login-failure".to_string();
                    b.category = crate::model::BehaviorCategory::ErrorCase;
                    b
                },
                {
                    let mut b = make_test_behavior();
                    b.name = "empty-email".to_string();
                    b.category = crate::model::BehaviorCategory::EdgeCase;
                    b
                },
            ],
            dependencies: vec![],
        };

        let behaviors = compute_behaviors(&spec);
        assert_eq!(behaviors.len(), 3);
        assert!(behaviors.contains_key("login-success"));
        assert!(behaviors.contains_key("login-failure"));
        assert!(behaviors.contains_key("empty-email"));
        assert_eq!(behaviors["login-success"].category, "happy_path");
        assert_eq!(behaviors["login-failure"].category, "error_case");
        assert_eq!(behaviors["empty-email"].category, "edge_case");
        // Each hash should be non-empty
        assert!(!behaviors["login-success"].hash.is_empty());
    }

    // ── acknowledge_baseline tests ──────────────────────

    #[test]
    /// cache: acknowledge_baseline_sets_baseline
    fn acknowledge_baseline_sets_baseline() {
        let mut cache = GraphCache::new();
        let mut behaviors = HashMap::new();
        behaviors.insert(
            "login-success".to_string(),
            BehaviorSnapshot {
                category: "happy_path".to_string(),
                hash: "abc123".to_string(),
            },
        );
        let mut entry = make_cached_entry("hash1");
        entry.behaviors = behaviors.clone();
        cache.upsert("auth".to_string(), entry);

        assert!(cache.specs["auth"].baseline.is_none());
        cache.acknowledge_baseline("auth");
        assert_eq!(cache.specs["auth"].baseline, Some(behaviors));
    }

    #[test]
    /// cache: acknowledge_baseline_noop_on_unknown
    fn acknowledge_baseline_noop_on_unknown() {
        let mut cache = GraphCache::new();
        cache.acknowledge_baseline("nonexistent");
        assert!(!cache.specs.contains_key("nonexistent"));
    }

    // ── baseline preserved on upsert ────────────────────

    #[test]
    /// cache: baseline_preserved_on_upsert
    fn baseline_preserved_on_upsert() {
        let mut cache = GraphCache::new();
        let mut behaviors = HashMap::new();
        behaviors.insert(
            "login-success".to_string(),
            BehaviorSnapshot {
                category: "happy_path".to_string(),
                hash: "abc123".to_string(),
            },
        );
        let baseline = behaviors.clone();
        let mut entry = make_cached_entry("hash1");
        entry.behaviors = behaviors;
        entry.baseline = Some(baseline.clone());
        cache.upsert("auth".to_string(), entry);

        // Upsert with new behaviors but same baseline
        let mut new_behaviors = HashMap::new();
        new_behaviors.insert(
            "login-success".to_string(),
            BehaviorSnapshot {
                category: "happy_path".to_string(),
                hash: "def456".to_string(),
            },
        );
        let mut new_entry = make_cached_entry("hash2");
        new_entry.behaviors = new_behaviors.clone();
        new_entry.baseline = Some(baseline.clone());
        cache.upsert("auth".to_string(), new_entry);

        // Baseline should be preserved (set by caller)
        assert_eq!(cache.specs["auth"].baseline, Some(baseline));
        assert_eq!(cache.specs["auth"].behaviors, new_behaviors);
    }

    // ── schema v3 triggers rebuild ──────────────────────

    #[test]
    /// cache: schema_v3_triggers_rebuild
    fn schema_v3_triggers_rebuild() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let json = r#"{"schema_version": 3, "specs": {}}"#;
        fs::write(path, json).unwrap();

        let result = GraphCache::load(path);
        assert!(matches!(result, Err(GraphError::SchemaMismatch)));
    }
}
