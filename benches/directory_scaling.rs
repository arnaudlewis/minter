#[allow(dead_code)]
mod common;

use std::time::Instant;

use common::{minter, spec_with_n_behaviors, temp_dir_with_specs};

// @minter:benchmark #performance#directory-validation-scaling

/// performance.nfr: directory-validation-scaling
/// Directory validation time must scale linearly with spec count.
/// Benchmark: validate directories with 10 and 50 spec files, measure per-spec marginal cost.
fn main() {
    let threshold_per_spec_ms = 20;

    // Build a directory with 10 specs
    let small_specs: Vec<(String, String)> = (0..10)
        .map(|i| {
            let name = format!("scale-s-{i}");
            let content = spec_with_n_behaviors(&name, 3);
            (name, content)
        })
        .collect();
    let small_refs: Vec<(&str, &str)> = small_specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_small_dir, small_path) = temp_dir_with_specs(&small_refs);

    // Build a directory with 50 specs
    let large_specs: Vec<(String, String)> = (0..50)
        .map(|i| {
            let name = format!("scale-l-{i}");
            let content = spec_with_n_behaviors(&name, 3);
            (name, content)
        })
        .collect();
    let large_refs: Vec<(&str, &str)> = large_specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_large_dir, large_path) = temp_dir_with_specs(&large_refs);

    // Warm up
    minter().arg("validate").arg(&small_path).assert().success();
    minter().arg("validate").arg(&large_path).assert().success();

    // Measure small directory (3 runs, take median)
    let mut small_durations: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        minter().arg("validate").arg(&small_path).assert().success();
        small_durations.push(start.elapsed().as_millis());
    }
    small_durations.sort();
    let small_median = small_durations[1];

    // Measure large directory (3 runs, take median)
    let mut large_durations: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        minter().arg("validate").arg(&large_path).assert().success();
        large_durations.push(start.elapsed().as_millis());
    }
    large_durations.sort();
    let large_median = large_durations[1];

    // Calculate per-spec marginal cost
    let delta_specs = 40; // 50 - 10
    let delta_ms = large_median.saturating_sub(small_median);
    let per_spec_ms = delta_ms / delta_specs as u128;

    println!(
        "directory-scaling: per-spec marginal cost = {per_spec_ms}ms \
         (small: {small_median}ms for 10, large: {large_median}ms for 50), \
         threshold = {threshold_per_spec_ms}ms"
    );

    assert!(
        per_spec_ms < threshold_per_spec_ms,
        "Per-spec marginal cost was {per_spec_ms}ms (small: {small_median}ms for 10, large: {large_median}ms for 50), threshold is {threshold_per_spec_ms}ms"
    );
}
