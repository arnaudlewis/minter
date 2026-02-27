#[allow(dead_code)]
mod common;

use std::time::Instant;

use common::{minter, spec_with_n_behaviors, temp_spec};

/// performance.nfr: validation-latency
/// Single-file validation must complete within 200ms.
/// Benchmark: validate a 50-behavior spec file, p95 < 200ms across 20 runs.
fn main() {
    let content = spec_with_n_behaviors("bench-latency", 50);
    let (_dir, path) = temp_spec("bench-latency", &content);

    let threshold_ms = 200;
    let runs = 20;
    let mut durations: Vec<u128> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let start = Instant::now();
        minter().arg("validate").arg(&path).assert().success();
        durations.push(start.elapsed().as_millis());
    }

    durations.sort();
    let p95_index = (runs as f64 * 0.95).ceil() as usize - 1;
    let p95 = durations[p95_index];

    println!("validation-latency: p95 = {p95}ms (threshold {threshold_ms}ms), all = {durations:?}");

    assert!(
        p95 < threshold_ms,
        "p95 validation latency was {p95}ms, threshold is {threshold_ms}ms. All durations: {durations:?}"
    );
}
