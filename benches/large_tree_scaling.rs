#[allow(dead_code)]
mod common;

use std::time::Instant;

use common::{minter, spec_with_deps_and_behaviors, spec_with_n_behaviors, temp_dir_with_specs};

/// performance.nfr: large-tree-validation-scaling
/// Validation of large spec trees must remain practical for CI pipelines.
/// Threshold: p95 < 5s for 500 specs across 10 cold-cache runs.
/// Dataset: specs with dependency chains (average depth 3) matching NFR description.
fn main() {
    let n = 500;
    let chain_len = 5; // groups of 5 create depth-4 chains -> average depth ~2-3
    let threshold_ms: u128 = 5000;
    let runs = 10;

    // Build specs with 20 behaviors each (realistic spec size) in dependency chains.
    // Chains: 0->1->2->3->4, 5->6->7->8->9, etc.
    let behaviors_per_spec = 20;
    let specs: Vec<(String, String)> = (0..n)
        .map(|i| {
            let name = format!("lt-{i}");
            let pos_in_chain = i % chain_len;
            if pos_in_chain < chain_len - 1 {
                let dep_name = format!("lt-{}", i + 1);
                (
                    name.clone(),
                    spec_with_deps_and_behaviors(&name, &[&dep_name], behaviors_per_spec),
                )
            } else {
                (
                    name.clone(),
                    spec_with_n_behaviors(&name, behaviors_per_spec),
                )
            }
        })
        .collect();
    let refs: Vec<(&str, &str)> = specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_dir, dir_path) = temp_dir_with_specs(&refs);

    let mut durations: Vec<u128> = Vec::with_capacity(runs);
    for _ in 0..runs {
        // Remove graph cache for cold-cache measurement
        let minter_cache = dir_path.join(".minter");
        if minter_cache.exists() {
            std::fs::remove_dir_all(&minter_cache).ok();
        }

        let start = Instant::now();
        minter()
            .current_dir(&dir_path)
            .arg("validate")
            .arg(".")
            .assert()
            .success();
        durations.push(start.elapsed().as_millis());
    }

    durations.sort();
    let p95_index = (runs as f64 * 0.95).ceil() as usize - 1;
    let p95 = durations[p95_index];

    println!("large-tree-scaling: p95 = {p95}ms (threshold {threshold_ms}ms), all = {durations:?}");

    assert!(
        p95 < threshold_ms,
        "p95 large-tree validation was {p95}ms (threshold {threshold_ms}ms). All: {durations:?}"
    );
}
