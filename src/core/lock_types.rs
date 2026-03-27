use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LockFile {
    #[allow(dead_code)]
    pub version: u64,
    pub specs: BTreeMap<String, LockSpec>,
    #[serde(default)]
    pub nfrs: BTreeMap<String, LockNfr>,
    #[serde(default)]
    pub benchmark_files: BTreeMap<String, LockBenchmarkFile>,
}

#[derive(Debug, Deserialize)]
pub struct LockBenchmarkFile {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct LockSpec {
    pub hash: String,
    #[allow(dead_code)]
    pub behaviors: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub nfrs: Vec<String>,
    #[serde(default)]
    pub test_files: BTreeMap<String, LockTestFile>,
}

#[derive(Debug, Deserialize)]
pub struct LockNfr {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct LockTestFile {
    pub hash: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub covers: Vec<String>,
}
