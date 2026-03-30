mod common;

use std::fs;

use minter::core::design::analysis::analyze_specs;

// ═══════════════════════════════════════════════════════════════
// spec-analysis behaviors
// ═══════════════════════════════════════════════════════════════

// ── Entity extraction ──────────────────────────────────────────

/// spec-analysis: extract-entities-from-aliases-and-returns
#[test]
fn extract_entities_from_aliases_and_returns() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec user-auth v1.0.0
title \"User Auth\"

description
  User authentication.

motivation
  Security.

behavior login [happy_path]
  \"Login\"

  given
    @user = User { email: \"test@test.com\" }

  when authenticate
    email = @user.email

  then returns Session
    assert token is present
";
    fs::write(dir.path().join("user-auth.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    // Entities should include "User" (from alias) and "Session" (from returns)
    assert!(result.entities.contains(&"User".to_string()));
    assert!(result.entities.contains(&"Session".to_string()));
    assert_eq!(result.entities.len(), 2);
}

/// spec-analysis: deduplicate-entities-across-specs
#[test]
fn deduplicate_entities_across_specs() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_a = "\
spec spec-a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-a [happy_path]
  \"Does A\"

  given
    @user = User { name: \"Alice\" }

  when act

  then returns User
    assert name == \"Alice\"
";
    let spec_b = "\
spec spec-b v1.0.0
title \"B\"

description
  B.

motivation
  B.

behavior do-b [happy_path]
  \"Does B\"

  given
    @user = User { name: \"Bob\" }

  when act

  then returns User
    assert name == \"Bob\"
";
    fs::write(dir.path().join("spec-a.spec"), spec_a).unwrap();
    fs::write(dir.path().join("spec-b.spec"), spec_b).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    // "User" appears in both specs but should only appear once
    let user_count = result.entities.iter().filter(|e| *e == "User").count();
    assert_eq!(user_count, 1, "User should appear exactly once after dedup");
}

// ── Archetype classification ───────────────────────────────────

/// spec-analysis: classify-dashboard-by-structure
#[test]
fn classify_dashboard_by_structure() {
    // 20+ specs + hierarchy depth 5+ + web-command spec = dashboard
    let dir = tempfile::TempDir::new().unwrap();
    // Create nested hierarchy: level1/level2/level3/level4/level5/deep.spec
    let deep_dir = dir.path().join("a").join("b").join("c").join("d").join("e");
    fs::create_dir_all(&deep_dir).unwrap();

    let make_spec = |name: &str| {
        format!(
            "\
spec {name} v1.0.0
title \"{name}\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
        )
    };

    // 20 specs at top level
    for i in 0..20 {
        let name = format!("spec-{}", i);
        fs::write(dir.path().join(format!("{}.spec", name)), make_spec(&name)).unwrap();
    }
    // A deep spec for hierarchy depth
    fs::write(deep_dir.join("deep.spec"), make_spec("deep")).unwrap();
    // A web-command spec
    fs::write(
        dir.path().join("web-command.spec"),
        make_spec("web-command"),
    )
    .unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(
        result.archetype.name, "dashboard",
        "Should classify as dashboard with 20+ specs, depth 5+, and web-command"
    );
    assert!(result.archetype.confidence >= 0.7);
}

/// spec-analysis: classify-ecommerce-by-entities
#[test]
fn classify_ecommerce_by_entities() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec shopping v1.0.0
title \"Shopping\"

description
  E-commerce flow.

motivation
  Shopping.

behavior add-to-cart [happy_path]
  \"Add product to cart\"

  given
    @product = Product { id: \"123\" }
    @cart = Cart { items: \"0\" }

  when add-to-cart
    product_id = @product.id

  then returns Order
    assert status == \"pending\"


behavior process-payment [happy_path]
  \"Process payment\"

  given
    @order = Order { total: \"100\" }
    @payment = Payment { method: \"card\" }

  when pay
    order_id = @order.id

  then returns Receipt
    assert paid == \"true\"
";
    fs::write(dir.path().join("shopping.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.archetype.name, "e-commerce");
    assert!(result.archetype.confidence >= 0.5);
}

/// spec-analysis: classify-content-by-entities
#[test]
fn classify_content_by_entities() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec blog v1.0.0
title \"Blog\"

description
  Content management.

motivation
  Publishing.

behavior create-article [happy_path]
  \"Create article\"

  given
    @author = Author { name: \"Jane\" }

  when publish
    title = \"My Post\"

  then returns Article
    assert status == \"published\"


behavior add-comment [happy_path]
  \"Add comment\"

  given
    @post = Post { id: \"1\" }
    @comment = Comment { body: \"Nice\" }

  when comment
    post_id = @post.id

  then returns Comment
    assert id is present
";
    fs::write(dir.path().join("blog.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.archetype.name, "content");
    assert!(result.archetype.confidence >= 0.5);
}

/// spec-analysis: classify-form-heavy-by-entities
#[test]
fn classify_form_heavy_by_entities() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec onboarding v1.0.0
title \"Onboarding\"

description
  Form wizard.

motivation
  Onboarding.

behavior submit-form [happy_path]
  \"Submit form\"

  given
    @form = Form { step: \"1\" }
    @wizard = Wizard { total: \"5\" }

  when submit
    step = @form.step

  then returns Validation
    assert valid == \"true\"


behavior next-step [happy_path]
  \"Next step\"

  given
    @step = Step { index: \"2\" }

  when advance

  then returns Step
    assert index == \"3\"
";
    fs::write(dir.path().join("onboarding.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.archetype.name, "form-heavy");
    assert!(result.archetype.confidence >= 0.5);
}

/// spec-analysis: classify-generic-fallback
#[test]
fn classify_generic_fallback() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec misc v1.0.0
title \"Misc\"

description
  Miscellaneous.

motivation
  Testing.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    fs::write(dir.path().join("misc.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.archetype.name, "generic");
    assert!(result.archetype.confidence < 0.5);
}

/// spec-analysis: structural-overrides-keyword
#[test]
fn structural_overrides_keyword() {
    // 25 specs + depth 8 + web-command → should be dashboard, NOT generic
    let dir = tempfile::TempDir::new().unwrap();
    let deep_dir = dir
        .path()
        .join("a")
        .join("b")
        .join("c")
        .join("d")
        .join("e")
        .join("f")
        .join("g")
        .join("h");
    fs::create_dir_all(&deep_dir).unwrap();

    let make_spec = |name: &str| {
        format!(
            "\
spec {name} v1.0.0
title \"{name}\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
        )
    };

    for i in 0..25 {
        let name = format!("spec-{}", i);
        fs::write(dir.path().join(format!("{}.spec", name)), make_spec(&name)).unwrap();
    }
    fs::write(deep_dir.join("deep.spec"), make_spec("deep")).unwrap();
    fs::write(
        dir.path().join("web-command.spec"),
        make_spec("web-command"),
    )
    .unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_ne!(
        result.archetype.name, "generic",
        "Structural signals (25 specs, depth 8, web-command) should override keyword absence"
    );
    assert_eq!(result.archetype.name, "dashboard");
    assert!(result.archetype.confidence >= 0.7);
}

// ── Archetype reasoning ────────────────────────────────────────

/// spec-analysis: archetype-reasoning-non-empty
#[test]
fn archetype_reasoning_non_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    fs::write(dir.path().join("a.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert!(
        !result.archetype.reasoning.is_empty(),
        "Archetype reasoning must be non-empty"
    );
}

// ── Spec metadata ──────────────────────────────────────────────

/// spec-analysis: project-name-from-root-directory
#[test]
fn project_name_from_root_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    fs::write(dir.path().join("a.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    // Project name comes from the root directory name
    let dir_name = dir
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(result.spec_metadata.project_name, dir_name);
}

/// spec-analysis: domains-group-by-parent-directory
#[test]
fn domains_group_by_parent_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let auth_dir = dir.path().join("auth");
    let billing_dir = dir.path().join("billing");
    fs::create_dir_all(&auth_dir).unwrap();
    fs::create_dir_all(&billing_dir).unwrap();

    let make_spec = |name: &str| {
        format!(
            "\
spec {name} v1.0.0
title \"{name}\"

description
  Test.

motivation
  Test.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
"
        )
    };

    fs::write(auth_dir.join("login.spec"), make_spec("login")).unwrap();
    fs::write(auth_dir.join("register.spec"), make_spec("register")).unwrap();
    fs::write(billing_dir.join("invoice.spec"), make_spec("invoice")).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    let domain_names: Vec<&str> = result
        .spec_metadata
        .domains
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(domain_names.contains(&"auth"));
    assert!(domain_names.contains(&"billing"));

    let auth_domain = result
        .spec_metadata
        .domains
        .iter()
        .find(|d| d.name == "auth")
        .unwrap();
    assert_eq!(auth_domain.spec_count, 2);
    assert!(auth_domain.spec_names.contains(&"login".to_string()));
    assert!(auth_domain.spec_names.contains(&"register".to_string()));
}

/// spec-analysis: per-spec-metrics
#[test]
fn per_spec_metrics() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec my-api v2.1.0
title \"My API\"

description
  API.

motivation
  API.

behavior do-thing [happy_path]
  \"Does\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"


behavior fail-thing [error_case]
  \"Fails\"

  given
    Ready

  when act

  then returns result
    assert status == \"error\"
";
    fs::write(dir.path().join("my-api.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.spec_metadata.spec_metrics.len(), 1);
    let metric = &result.spec_metadata.spec_metrics[0];
    assert_eq!(metric.name, "my-api");
    assert_eq!(metric.version, "2.1.0");
    assert_eq!(metric.behavior_count, 2);
}

/// spec-analysis: aggregate-counts
#[test]
fn aggregate_counts() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec_a = "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-a [happy_path]
  \"Does A\"

  given
    @user = User { name: \"Alice\" }

  when act

  then returns Result
    assert ok == \"true\"
";
    let spec_b = "\
spec b v1.0.0
title \"B\"

description
  B.

motivation
  B.

behavior do-b1 [happy_path]
  \"Does B1\"

  given
    Ready

  when act

  then returns result
    assert ok == \"true\"


behavior do-b2 [error_case]
  \"Does B2\"

  given
    Ready

  when act

  then returns result
    assert ok == \"false\"
";
    fs::write(dir.path().join("a.spec"), spec_a).unwrap();
    fs::write(dir.path().join("b.spec"), spec_b).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.spec_metadata.total_spec_count, 2);
    assert_eq!(result.spec_metadata.total_behavior_count, 3);
    assert!(result.spec_metadata.total_entity_count > 0);
}

// ── App description ────────────────────────────────────────────

/// spec-analysis: app-description-from-entities
#[test]
fn app_description_from_entities() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec shopping v1.0.0
title \"Shopping\"

description
  E-commerce.

motivation
  Shopping.

behavior buy [happy_path]
  \"Buy product\"

  given
    @product = Product { id: \"1\" }
    @cart = Cart { items: \"1\" }

  when buy

  then returns Order
    assert status == \"placed\"
";
    fs::write(dir.path().join("shopping.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert!(
        !result.app_description.is_empty(),
        "App description must be non-empty"
    );
}

// ── Structural metrics ─────────────────────────────────────────

/// spec-analysis: flow-count-and-hierarchy-depth
#[test]
fn flow_count_and_hierarchy_depth() {
    let dir = tempfile::TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();

    let spec = "\
spec a v1.0.0
title \"A\"

description
  A.

motivation
  A.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    fs::write(dir.path().join("a.spec"), spec).unwrap();
    fs::write(sub.join("b.spec"), spec.replace("spec a", "spec b")).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    // flow_count = total behavior count across specs
    assert_eq!(result.flow_count, 2);
    // hierarchy_depth: root (1) + sub (2) = 2
    assert!(result.hierarchy_depth >= 2);
}

/// spec-analysis: behavior-distribution
#[test]
fn behavior_distribution() {
    let dir = tempfile::TempDir::new().unwrap();
    let spec = "\
spec dist v1.0.0
title \"Distribution\"

description
  Test.

motivation
  Test.

behavior happy [happy_path]
  \"Happy\"

  given
    Ready

  when act

  then returns result
    assert ok == \"true\"


behavior error [error_case]
  \"Error\"

  given
    Ready

  when act

  then returns result
    assert ok == \"false\"


behavior edge [edge_case]
  \"Edge\"

  given
    Ready

  when act

  then returns result
    assert ok == \"maybe\"
";
    fs::write(dir.path().join("dist.spec"), spec).unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    assert_eq!(result.behavior_distribution.happy_path, 1);
    assert_eq!(result.behavior_distribution.error_case, 1);
    assert_eq!(result.behavior_distribution.edge_case, 1);
}

// ── Error cases ────────────────────────────────────────────────

/// spec-analysis: empty-directory
#[test]
fn empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();

    let result = analyze_specs(dir.path());
    assert!(result.is_err(), "Empty directory should return an error");
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("no spec files"),
        "Error should mention no spec files: {err}"
    );
}

/// spec-analysis: nonexistent-directory
#[test]
fn nonexistent_directory() {
    let path = std::path::Path::new("/tmp/nonexistent-minter-test-dir-abc123");

    let result = analyze_specs(path);
    assert!(
        result.is_err(),
        "Nonexistent directory should return an error"
    );
}

/// spec-analysis: all-unparseable
#[test]
fn all_unparseable() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("bad.spec"), "this is not valid spec DSL").unwrap();
    fs::write(
        dir.path().join("also-bad.spec"),
        "also not valid spec content",
    )
    .unwrap();

    let result = analyze_specs(dir.path());
    assert!(
        result.is_err(),
        "All unparseable specs should return an error"
    );
}

// ── Edge case ──────────────────────────────────────────────────

/// spec-analysis: skip-unparseable-with-warnings
#[test]
fn skip_unparseable_with_warnings() {
    let dir = tempfile::TempDir::new().unwrap();
    let good_spec = "\
spec good v1.0.0
title \"Good\"

description
  Good.

motivation
  Good.

behavior do-thing [happy_path]
  \"Does a thing\"

  given
    Ready

  when act

  then returns result
    assert status == \"ok\"
";
    fs::write(dir.path().join("good.spec"), good_spec).unwrap();
    fs::write(dir.path().join("bad.spec"), "this is garbage").unwrap();

    let result = analyze_specs(dir.path()).unwrap();
    // Should have analyzed the good spec
    assert_eq!(result.spec_metadata.total_spec_count, 1);
    // Should have a warning about the bad spec
    assert!(
        !result.warnings.is_empty(),
        "Should have warnings about unparseable specs"
    );
    assert!(result.warnings.iter().any(|w: &String| w.contains("bad")));
}
