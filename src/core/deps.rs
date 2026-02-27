use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::io;
use crate::core::parser;
use crate::core::validation::semantic;
use crate::model::{Dependency, Spec};

pub struct ResolvedDep {
    pub spec: Spec,
    pub valid: bool,
}

pub struct ResolutionContext {
    pub siblings: HashMap<String, PathBuf>,
    pub resolved: HashMap<String, ResolvedDep>,
    pub stack: Vec<String>,
    pub errors: Vec<String>,
}

pub fn resolve_and_collect(deps: &[Dependency], ctx: &mut ResolutionContext, depth: usize) {
    if depth > 256 {
        ctx.errors
            .push("dependency resolution exceeded maximum depth of 256".to_string());
        return;
    }
    for dep in deps {
        if ctx.stack.contains(&dep.spec_name) {
            ctx.errors.push(format!(
                "dependency cycle detected: {} \u{2192} {}",
                ctx.stack.join(" \u{2192} "),
                dep.spec_name
            ));
            continue;
        }

        if ctx.resolved.contains_key(&dep.spec_name) {
            check_version_constraint(dep, &ctx.resolved[&dep.spec_name].spec, &mut ctx.errors);
            continue;
        }

        let spec_path = match ctx.siblings.get(&dep.spec_name) {
            Some(p) => p.clone(),
            None => {
                ctx.errors.push(format!(
                    "dependency '{}' not found (no {}.spec in spec tree)",
                    dep.spec_name, dep.spec_name
                ));
                continue;
            }
        };

        let source = match io::read_file_safe(&spec_path) {
            Ok(s) => s,
            Err(e) => {
                ctx.errors
                    .push(format!("cannot read dependency '{}': {}", dep.spec_name, e));
                continue;
            }
        };

        let dep_spec = match parser::parse(&source) {
            Ok(s) => s,
            Err(_) => {
                ctx.errors
                    .push(format!("dependency '{}' has parse errors", dep.spec_name));
                continue;
            }
        };

        let valid = semantic::validate(&dep_spec).is_ok();
        if !valid {
            ctx.errors.push(format!(
                "dependency '{}' has validation errors",
                dep.spec_name
            ));
        }

        check_version_constraint(dep, &dep_spec, &mut ctx.errors);

        let sub_deps = dep_spec.dependencies.clone();
        ctx.resolved.insert(
            dep.spec_name.clone(),
            ResolvedDep {
                spec: dep_spec,
                valid,
            },
        );
        ctx.stack.push(dep.spec_name.clone());

        resolve_and_collect(&sub_deps, ctx, depth + 1);

        ctx.stack.pop();
    }
}

pub fn check_version_constraint(dep: &Dependency, dep_spec: &Spec, errors: &mut Vec<String>) {
    let constraint = &dep.version_constraint;
    let required = constraint.trim_start_matches(">=").trim();

    let req = match semver::Version::parse(required) {
        Ok(v) => v,
        Err(_) => {
            errors.push(format!(
                "dependency '{}' has invalid version constraint: {}",
                dep.spec_name, constraint
            ));
            return;
        }
    };

    let actual = match semver::Version::parse(&dep_spec.version) {
        Ok(v) => v,
        Err(_) => {
            errors.push(format!(
                "dependency '{}' has unparseable version: {}",
                dep.spec_name, dep_spec.version
            ));
            return;
        }
    };

    if actual < req {
        errors.push(format!(
            "dependency '{}' requires >= {} but found {}",
            dep.spec_name, required, dep_spec.version
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// depth-limit: resolve-and-collect-exceeds-max-depth
    #[test]
    fn resolve_and_collect_exceeds_max_depth() {
        let deps = vec![Dependency {
            spec_name: "deep-spec".to_string(),
            version_constraint: ">=1.0.0".to_string(),
        }];
        let mut ctx = ResolutionContext {
            siblings: HashMap::new(),
            resolved: HashMap::new(),
            stack: Vec::new(),
            errors: Vec::new(),
        };
        // Call at depth 257 (exceeds the limit of 256)
        resolve_and_collect(&deps, &mut ctx, 257);
        assert_eq!(ctx.errors.len(), 1);
        assert!(
            ctx.errors[0].contains("dependency resolution exceeded maximum depth of 256"),
            "Expected depth limit error, got: {}",
            ctx.errors[0]
        );
    }
}
