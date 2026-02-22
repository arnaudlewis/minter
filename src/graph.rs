use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 1;

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
}

#[derive(Debug)]
pub enum GraphError {
    Corrupted(String),
    SchemaMismatch,
    Io(io::Error),
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
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    /// Insert or update a spec entry in the graph.
    pub fn upsert(
        &mut self,
        name: String,
        content_hash: String,
        version: String,
        behavior_count: usize,
        valid: bool,
        dependencies: Vec<String>,
    ) {
        self.specs.insert(
            name,
            CachedEntry {
                content_hash,
                version,
                behavior_count,
                valid,
                dependencies,
            },
        );
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
        .join(".specval")
        .join("graph.json")
}
