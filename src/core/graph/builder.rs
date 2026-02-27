use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::core::discover;
use crate::core::graph::{self, NfrCachedEntry};
use crate::core::io;
use crate::core::parser::nfr as nfr_parser;
use crate::model::NfrSpec;

/// Discovered and parsed NFR files with their content hashes.
pub struct NfrDiscovery {
    pub specs: HashMap<String, NfrSpec>,
    pub hashes: HashMap<String, String>,
}

/// Discover, parse, and hash all NFR files in a directory.
pub fn discover_and_parse_nfrs(dir: &Path) -> NfrDiscovery {
    let mut specs = HashMap::new();
    let mut hashes = HashMap::new();
    let nfr_files = discover::discover_nfr_files(dir);
    for path in &nfr_files {
        if let Ok(source) = io::read_file_safe(path)
            && let Ok(nfr) = nfr_parser::parse_nfr(&source)
        {
            let hash = graph::content_hash(&source);
            hashes.insert(nfr.category.clone(), hash);
            specs.insert(nfr.category.clone(), nfr);
        }
    }
    NfrDiscovery { specs, hashes }
}

impl super::GraphState {
    /// Sync NFR cache entries from discovery results. Returns the set of changed categories.
    pub fn sync_nfrs(&mut self, discovery: &NfrDiscovery) -> HashSet<String> {
        let mut changed = HashSet::new();
        for (cat, hash) in &discovery.hashes {
            if self.cache.is_nfr_changed(cat, hash) {
                changed.insert(cat.clone());
                if let Some(nfr) = discovery.specs.get(cat) {
                    self.cache.upsert_nfr(
                        cat.clone(),
                        NfrCachedEntry {
                            content_hash: hash.clone(),
                            version: nfr.version.clone(),
                            constraint_count: nfr.constraints.len(),
                        },
                    );
                    self.dirty = true;
                }
            }
        }
        // Also detect deleted NFR categories
        for cat in self.cache.nfrs.keys() {
            if !discovery.hashes.contains_key(cat) {
                changed.insert(cat.clone());
            }
        }
        changed
    }
}
