mod common;

use std::time::Instant;

use common::{minter, temp_dir_with_specs, temp_spec};

/// Generate a spec with N behaviors for benchmarking.
fn spec_with_n_behaviors(name: &str, n: usize) -> String {
    let mut s = format!(
        "\
spec {name} v1.0.0
title \"{name}\"

description
  Benchmark spec with {n} behaviors.

motivation
  Performance testing.

"
    );

    for i in 0..n {
        let category = if i == 0 { "happy_path" } else { "edge_case" };
        s.push_str(&format!(
            "\
behavior do-thing-{i} [{category}]
  \"Does thing {i}\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"

"
        ));
    }
    s
}

/// performance.nfr: validation-latency
/// Single-file validation must complete within 200ms.
/// Benchmark: validate a 50-behavior spec file, p95 < 200ms across 20 runs.
#[test]
fn benchmark_validation_latency() {
    let content = spec_with_n_behaviors("bench-latency", 50);
    let (_dir, path) = temp_spec("bench-latency", &content);

    let threshold_ms = 200;
    let runs = 20;
    let mut durations: Vec<u128> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let start = Instant::now();
        minter()
            .arg("validate")
            .arg(&path)
            .assert()
            .success();
        durations.push(start.elapsed().as_millis());
    }

    durations.sort();
    let p95_index = (runs as f64 * 0.95).ceil() as usize - 1;
    let p95 = durations[p95_index];

    assert!(
        p95 < threshold_ms,
        "p95 validation latency was {p95}ms, threshold is {threshold_ms}ms. All durations: {durations:?}"
    );
}

/// performance.nfr: directory-validation-scaling
/// Directory validation time must scale linearly with spec count.
/// Benchmark: validate directories with 10 and 50 spec files, measure per-spec marginal cost.
#[test]
fn benchmark_directory_validation_scaling() {
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

    assert!(
        per_spec_ms < threshold_per_spec_ms,
        "Per-spec marginal cost was {per_spec_ms}ms (small: {small_median}ms for 10, large: {large_median}ms for 50), threshold is {threshold_per_spec_ms}ms"
    );
}
