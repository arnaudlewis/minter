use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphCache {
    pub schema_version: u32,
    pub specs: HashMap<String, CachedEntry>,
    #[serde(default)]
    pub nfrs: HashMap<String, NfrCachedEntry>,
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
        assert_eq!(cache.schema_version, 3);
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
        assert_eq!(loaded.schema_version, 3);
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
}
