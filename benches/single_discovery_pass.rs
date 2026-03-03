#[allow(dead_code)]
mod common;

use std::time::Instant;

use common::{minter, spec_with_n_behaviors, temp_dir_with_nested_specs};

// @minter:benchmark #performance#single-discovery-pass

/// performance.nfr: single-discovery-pass
/// File discovery must traverse the directory tree at most once per validation invocation.
/// With single discovery, per-spec cost is constant. With per-spec discovery, it grows with N.
fn main() {
    let small_n: usize = 50;
    let large_n: usize = 300;

    // Build small directory (nested 5 levels to amplify walk overhead)
    let small_specs: Vec<(String, String)> = (0..small_n)
        .map(|i| {
            let name = format!(
                "d{}/d{}/d{}/d{}/d{}/sdp-s-{i}",
                i % 3,
                i % 5,
                i % 4,
                i % 2,
                i % 3
            );
            (name, spec_with_n_behaviors(&format!("sdp-s-{i}"), 2))
        })
        .collect();
    let small_refs: Vec<(&str, &str)> = small_specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_small_dir, small_path) = temp_dir_with_nested_specs(&small_refs);

    // Build large directory (same nesting structure, more files)
    let large_specs: Vec<(String, String)> = (0..large_n)
        .map(|i| {
            let name = format!(
                "d{}/d{}/d{}/d{}/d{}/sdp-l-{i}",
                i % 3,
                i % 5,
                i % 4,
                i % 2,
                i % 3
            );
            (name, spec_with_n_behaviors(&format!("sdp-l-{i}"), 2))
        })
        .collect();
    let large_refs: Vec<(&str, &str)> = large_specs
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    let (_large_dir, large_path) = temp_dir_with_nested_specs(&large_refs);

    // Warm up
    minter().arg("validate").arg(&small_path).assert().success();
    minter().arg("validate").arg(&large_path).assert().success();

    // Measure small (3 runs, median)
    let mut small_d: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        minter().arg("validate").arg(&small_path).assert().success();
        small_d.push(start.elapsed().as_millis());
    }
    small_d.sort();
    let small_median = small_d[1];

    // Measure large (3 runs, median)
    let mut large_d: Vec<u128> = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        minter().arg("validate").arg(&large_path).assert().success();
        large_d.push(start.elapsed().as_millis());
    }
    large_d.sort();
    let large_median = large_d[1];

    let small_per = small_median as f64 / small_n as f64;
    let large_per = large_median as f64 / large_n as f64;

    // Guard against near-zero timings on fast CI runners
    if small_per < 0.01 || large_per < 0.01 {
        println!(
            "single-discovery-pass: timings too small to compare reliably \
             ({small_per:.1}ms/spec at {small_n} vs {large_per:.1}ms/spec at {large_n}), skipping"
        );
        return;
    }

    let ratio = large_per / small_per;

    println!(
        "single-discovery-pass: ratio = {ratio:.2}x \
         ({small_per:.1}ms/spec at {small_n} vs {large_per:.1}ms/spec at {large_n})"
    );

    // With single discovery pass, per-spec cost is constant (ratio ~ 1.0).
    // With per-spec discovery (current), per-spec cost grows linearly (ratio > 1.5).
    assert!(
        ratio < 1.5,
        "Per-spec cost ratio {ratio:.2}x ({small_per:.1}ms/spec at {small_n} vs \
         {large_per:.1}ms/spec at {large_n}). \
         Single discovery pass keeps this near 1.0x."
    );
}
