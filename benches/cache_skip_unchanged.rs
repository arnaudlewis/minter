#[allow(dead_code)]
mod common;

use std::time::Instant;

use common::{minter, spec_with_n_behaviors, temp_dir_with_specs};

/// performance.nfr: cache-skip-unchanged
/// Unchanged spec files must skip parsing when a valid graph cache exists.
///
/// Story:
///   1. Create 200 specs in a temp directory
///   2. First run (cold cache) — validate everything, graph.json gets created
///   3. Second run (warm cache, files unchanged) — should be faster
///   4. Assert speedup >= 2x — RED until cache-skip is implemented
fn main() {
    let n = 500;
    let specs: Vec<(String, String)> = (0..n)
        .map(|i| {
            let name = format!("cache-{i}");
            (name.clone(), spec_with_n_behaviors(&name, 20))
        })
        .collect();
    let refs: Vec<(&str, &str)> = specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_dir, dir_path) = temp_dir_with_specs(&refs);

    // --- Step 1: Warm up OS page caches, then delete graph cache ---
    minter()
        .current_dir(&dir_path)
        .arg("validate")
        .arg(".")
        .assert()
        .success();
    let minter_cache = dir_path.join(".minter");
    if minter_cache.exists() {
        std::fs::remove_dir_all(&minter_cache).ok();
    }

    // --- Step 2: Cold cache — validate, measure time ---
    // Each of 3 runs deletes .minter first; take median.
    let mut cold_d: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let mc = dir_path.join(".minter");
        if mc.exists() {
            std::fs::remove_dir_all(&mc).ok();
        }
        let start = Instant::now();
        minter()
            .current_dir(&dir_path)
            .arg("validate")
            .arg(".")
            .assert()
            .success();
        cold_d.push(start.elapsed().as_millis());
    }
    cold_d.sort();
    let cold_ms = cold_d[1];

    // Verify graph cache was created
    assert!(
        dir_path.join(".minter").join("graph.json").exists(),
        "graph.json should exist after validation — cache write is working"
    );

    // --- Step 3: Warm cache — validate again, files unchanged ---
    let mut warm_d: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        minter()
            .current_dir(&dir_path)
            .arg("validate")
            .arg(".")
            .assert()
            .success();
        warm_d.push(start.elapsed().as_millis());
    }
    warm_d.sort();
    let warm_ms = warm_d[1];

    // --- Step 4: Assert speedup >= 2x ---
    let speedup = if warm_ms == 0 {
        cold_ms as f64
    } else {
        cold_ms as f64 / warm_ms as f64
    };

    println!(
        "cache-skip-unchanged: cold = {cold_ms}ms, warm = {warm_ms}ms, \
         speedup = {speedup:.1}x (expected >= 2.0x)"
    );
    println!("  cold runs: {cold_d:?}");
    println!("  warm runs: {warm_d:?}");

    assert!(
        speedup >= 2.0,
        "Cache speedup was only {speedup:.1}x (cold: {cold_ms}ms, warm: {warm_ms}ms). \
         Expected >= 2.0x when unchanged files skip parsing."
    );
}
