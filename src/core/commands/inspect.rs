use std::path::Path;

use super::inspect_core;
use crate::core::io;
use crate::core::parser;
use crate::core::parser::nfr as nfr_parser;

/// Display structured metadata for a spec file.
pub fn run_inspect(file: &Path) -> i32 {
    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        return 1;
    }

    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "nfr" {
        return display_nfr(file);
    }

    let source = match io::read_file_safe(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", file.display(), e);
            return 1;
        }
    };

    let spec = match parser::parse(&source) {
        Ok(s) => s,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", file.display(), e);
            }
            return 1;
        }
    };

    let result = inspect_core::inspect_spec(&spec);

    // Header
    println!("{} v{}", result.name, result.version);
    println!("title: {}", result.title);
    println!();

    // Behavior count
    println!(
        "{} {}",
        result.behavior_count,
        if result.behavior_count == 1 {
            "behavior"
        } else {
            "behaviors"
        }
    );

    // Category distribution
    for (cat, n) in &result.categories {
        println!("  {}: {}", cat, n);
    }
    println!();

    // Dependencies
    if result.dependencies.is_empty() {
        println!("no dependencies");
    } else {
        println!("dependencies:");
        for (name, version) in &result.dependencies {
            println!("  {} >= {}", name, version);
        }
    }
    println!();

    // Assertion types
    if !result.assertion_types.is_empty() {
        println!("assertion types:");
        for (at, n) in &result.assertion_types {
            println!("  {}: {}", at, n);
        }
    }

    0
}

/// Display structured metadata for an NFR file.
fn display_nfr(file: &Path) -> i32 {
    let source = match io::read_file_safe(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", file.display(), e);
            return 1;
        }
    };

    let nfr = match nfr_parser::parse_nfr(&source) {
        Ok(n) => n,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}: {}", file.display(), e);
            }
            return 1;
        }
    };

    let result = inspect_core::inspect_nfr(&nfr);

    // Header
    println!("{} v{}", result.category, result.version);
    println!("title: {}", result.title);
    println!();

    // Constraint count
    println!(
        "{} {}",
        result.constraint_count,
        if result.constraint_count == 1 {
            "constraint"
        } else {
            "constraints"
        }
    );

    // Type distribution
    println!("  metric: {}", result.metric_count);
    println!("  rule: {}", result.rule_count);
    println!();

    // Category
    println!("category: {}", result.category);
    println!();

    // No dependencies for NFR files
    println!("no dependencies");

    0
}
