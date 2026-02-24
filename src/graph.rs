use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphCache {
    pub schema_version: u32,
    pub specs: HashMap<String, CachedEntry>,
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
        }
    }

    /// Load a graph cache from a JSON file.
    /// Returns GraphError if the file is corrupted or has wrong schema.
    pub fn load(path: &Path) -> Result<GraphCache, GraphError> {
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

    /// Save the graph cache to a JSON file.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(io::Error::other)?;
        fs::write(path, json)
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
}
