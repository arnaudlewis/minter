use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::deps::{self, ResolutionContext, ResolvedDep};
use crate::core::parser;
use crate::core::parser::fr::ParseError;
use crate::core::parser::nfr as nfr_parser;
use crate::core::validation::crossref::CrossRefError;
use crate::core::validation::semantic::SemanticError;
use crate::core::validation::{crossref as nfr_crossref, nfr_semantic, semantic};
use crate::model::{NfrSpec, Spec};

/// Result of validating a spec from source text.
pub struct SpecValidation {
    pub spec: Option<Spec>,
    pub parse_errors: Vec<ParseError>,
    pub semantic_errors: Vec<SemanticError>,
    pub crossref_errors: Vec<CrossRefError>,
    pub dep_errors: Vec<String>,
    pub resolved_deps: HashMap<String, ResolvedDep>,
    pub is_valid: bool,
}

/// Result of validating an NFR from source text.
pub struct NfrValidation {
    pub nfr: Option<NfrSpec>,
    pub parse_errors: Vec<ParseError>,
    pub semantic_errors: Vec<SemanticError>,
    pub is_valid: bool,
}

/// Validate a spec from source text.
///
/// If `siblings` and `nfr_specs` are provided, also resolves dependencies
/// and cross-validates NFR references (deep mode).
pub fn validate_spec(
    source: &str,
    spec_name_for_stack: Option<&str>,
    siblings: Option<&HashMap<String, PathBuf>>,
    nfr_specs: Option<&HashMap<String, NfrSpec>>,
) -> SpecValidation {
    let spec = match parser::parse(source) {
        Ok(s) => s,
        Err(errors) => {
            return SpecValidation {
                spec: None,
                parse_errors: errors,
                semantic_errors: vec![],
                crossref_errors: vec![],
                dep_errors: vec![],
                resolved_deps: HashMap::new(),
                is_valid: false,
            };
        }
    };

    let semantic_errors = match semantic::validate(&spec) {
        Ok(()) => vec![],
        Err(errors) => errors,
    };

    if !semantic_errors.is_empty() {
        return SpecValidation {
            spec: Some(spec),
            parse_errors: vec![],
            semantic_errors,
            crossref_errors: vec![],
            dep_errors: vec![],
            resolved_deps: HashMap::new(),
            is_valid: false,
        };
    }

    // Deep mode: resolve dependencies + cross-validate NFRs
    let mut crossref_errors = vec![];
    let mut dep_errors = vec![];
    let mut resolved_deps = HashMap::new();

    if let Some(siblings) = siblings {
        let stack_name = spec_name_for_stack
            .map(String::from)
            .unwrap_or_else(|| spec.name.clone());
        let mut res_ctx = ResolutionContext {
            siblings: siblings.clone(),
            resolved: HashMap::new(),
            stack: vec![stack_name],
            errors: Vec::new(),
        };
        deps::resolve_and_collect(&spec.dependencies, &mut res_ctx, 0);
        dep_errors = res_ctx.errors;
        resolved_deps = res_ctx.resolved;
    }

    if let Some(nfr_specs) = nfr_specs {
        let has_nfr_refs =
            !spec.nfr_refs.is_empty() || spec.behaviors.iter().any(|b| !b.nfr_refs.is_empty());
        if has_nfr_refs {
            if let Err(errors) = nfr_crossref::cross_validate(&spec, nfr_specs) {
                crossref_errors = errors;
            }
        }
    }

    let is_valid = dep_errors.is_empty() && crossref_errors.is_empty();

    SpecValidation {
        spec: Some(spec),
        parse_errors: vec![],
        semantic_errors: vec![],
        crossref_errors,
        dep_errors,
        resolved_deps,
        is_valid,
    }
}

/// Validate an NFR from source text.
pub fn validate_nfr(source: &str) -> NfrValidation {
    let nfr = match nfr_parser::parse_nfr(source) {
        Ok(n) => n,
        Err(errors) => {
            return NfrValidation {
                nfr: None,
                parse_errors: errors,
                semantic_errors: vec![],
                is_valid: false,
            };
        }
    };

    let semantic_errors = match nfr_semantic::validate(&nfr) {
        Ok(()) => vec![],
        Err(errors) => errors,
    };

    let is_valid = semantic_errors.is_empty();

    NfrValidation {
        nfr: Some(nfr),
        parse_errors: vec![],
        semantic_errors,
        is_valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SPEC: &str = "\
spec user-login v1.0.0
title \"User Login\"

description
  Handles user authentication

motivation
  Core auth flow

behavior successful-login [happy_path]
  \"User logs in with valid credentials\"

  given
    User has a registered account

  when login

  then side_effect
    assert status == \"success\"
";

    const VALID_NFR: &str = "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  System performance constraints

motivation
  Ensure responsive UX

constraint response-time [metric]
  \"API response time\"

  metric \"Wall-clock latency\"
  threshold < 200ms

  verification
    environment staging
    benchmark \"Validate a 50-behavior spec file\"
    pass \"p95 < threshold\"

  violation high
  overridable no

constraint caching [rule]
  \"All reads must be cached\"

  rule
    All read endpoints must use cache

  verification
    static \"lint-check\"

  violation low
  overridable no
";

    #[test]
    /// validate_core: valid_spec_shallow
    fn valid_spec_shallow() {
        let result = validate_spec(VALID_SPEC, None, None, None);
        assert!(result.is_valid);
        assert!(result.parse_errors.is_empty());
        assert!(result.semantic_errors.is_empty());
        assert!(result.spec.is_some());
        assert_eq!(result.spec.as_ref().unwrap().name, "user-login");
    }

    #[test]
    /// validate_core: invalid_spec_parse_error
    fn invalid_spec_parse_error() {
        let result = validate_spec("not a valid spec", None, None, None);
        assert!(!result.is_valid);
        assert!(!result.parse_errors.is_empty());
        assert!(result.spec.is_none());
    }

    #[test]
    /// validate_core: valid_nfr
    fn valid_nfr() {
        let result = validate_nfr(VALID_NFR);
        assert!(result.is_valid);
        assert!(result.parse_errors.is_empty());
        assert!(result.semantic_errors.is_empty());
        assert!(result.nfr.is_some());
        assert_eq!(result.nfr.as_ref().unwrap().category, "performance");
    }

    #[test]
    /// validate_core: invalid_nfr_parse_error
    fn invalid_nfr_parse_error() {
        let result = validate_nfr("not a valid nfr");
        assert!(!result.is_valid);
        assert!(!result.parse_errors.is_empty());
        assert!(result.nfr.is_none());
    }
}
