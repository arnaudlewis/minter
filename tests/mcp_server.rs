mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

// ── MCP Client helper ──────────────────────────────────

struct McpClient {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
    stdin: std::process::ChildStdin,
    next_id: u64,
}

impl McpClient {
    fn new() -> Self {
        let bin_path = assert_cmd::cargo::cargo_bin_cmd!("minter-mcp")
            .get_program()
            .to_owned();
        let mut child = Command::new(bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn minter-mcp");

        let stdout = child.stdout.take().expect("capture stdout");
        let reader = BufReader::new(stdout);
        let stdin = child.stdin.take().expect("capture stdin");

        let mut client = McpClient {
            child,
            reader,
            stdin,
            next_id: 1,
        };
        client.initialize();
        client
    }

    fn send_request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let msg = serde_json::to_string(&request).unwrap();
        writeln!(self.stdin, "{}", msg).expect("write request");
        self.stdin.flush().expect("flush stdin");
        self.read_response(id)
    }

    fn send_notification(&mut self, method: &str, params: Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let msg = serde_json::to_string(&notification).unwrap();
        writeln!(self.stdin, "{}", msg).expect("write notification");
        self.stdin.flush().expect("flush stdin");
    }

    fn read_response(&mut self, expected_id: u64) -> Value {
        loop {
            let mut line = String::new();
            self.reader
                .read_line(&mut line)
                .expect("read response line");
            if line.trim().is_empty() {
                continue;
            }
            let response: Value = serde_json::from_str(line.trim()).expect("parse response JSON");
            // Skip notifications (no id field)
            if response.get("id").is_some() {
                assert_eq!(
                    response["id"].as_u64().unwrap(),
                    expected_id,
                    "response id mismatch"
                );
                return response;
            }
        }
    }

    fn initialize(&mut self) {
        let resp = self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "0.1.0"
                }
            }),
        );
        assert!(resp.get("result").is_some(), "initialize should succeed");
        // Send initialized notification
        self.send_notification("notifications/initialized", json!({}));
    }

    fn call_tool(&mut self, name: &str, args: Value) -> Value {
        let resp = self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": args,
            }),
        );
        resp["result"].clone()
    }

    fn list_tools(&mut self) -> Value {
        let resp = self.send_request("tools/list", json!({}));
        resp["result"]["tools"].clone()
    }

    fn server_info(&mut self) -> Value {
        // server_info was returned by initialize, need to re-init
        // For simplicity, just return the first initialize result
        // Actually, let's just call initialize again to get the info
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "0.1.0"
                }
            }
        });
        let msg = serde_json::to_string(&request).unwrap();
        writeln!(self.stdin, "{}", msg).expect("write request");
        self.stdin.flush().expect("flush stdin");
        let resp = self.read_response(id);
        resp["result"].clone()
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── Tool result helpers ────────────────────────────────

/// Extract the text content from a tool call result.
fn text_content(result: &Value) -> String {
    result["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// Parse the text content as JSON.
fn json_content(result: &Value) -> Value {
    let text = text_content(result);
    serde_json::from_str(&text).expect("parse JSON content")
}

/// Check if the tool result has isError set to true.
fn is_error(result: &Value) -> bool {
    result["isError"].as_bool().unwrap_or(false)
}

// ── Temp file helpers ──────────────────────────────────

fn write_spec(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(format!("{}.spec", name));
    std::fs::write(&path, content).expect("write spec");
    path
}

fn write_nfr(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(format!("{}.nfr", name));
    std::fs::write(&path, content).expect("write nfr");
    path
}

// ── Spec fixtures ──────────────────────────────────────

const VALID_SPEC: &str = "\
spec test-spec v1.0.0
title \"Test Spec\"

description
  A test spec for validation.

motivation
  Testing minter.

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    The system is ready

  when act

  then emits stdout
    assert output contains \"done\"
";

const VALID_NFR: &str = "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Defines performance constraints.

motivation
  Performance matters.


constraint api-response-time [metric]
  \"API endpoints must respond within acceptable latency bounds\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging, production
    benchmark \"100 concurrent requests per endpoint\"
    pass \"p95 < threshold\"

  violation high
  overridable yes
";

const BROKEN_SPEC: &str = "\
spec broken v2.0.0
title \"Broken\"

description
  missing stuff.

motivation
  Broken.
";

const SPEC_A_DEPENDS_B: &str = "\
spec a-spec v1.0.0
title \"Spec A\"

description
  A spec that depends on B.

motivation
  Dependency testing.

behavior do-a [happy_path]
  \"Do A\"

  given
    B is available

  when act

  then emits stdout
    assert output contains \"a\"

depends on b-spec >= 1.0.0
";

const SPEC_B: &str = "\
spec b-spec v1.2.0
title \"Spec B\"

description
  Spec B is a dependency.

motivation
  Dependency target.

behavior do-b [happy_path]
  \"Do B\"

  given
    System ready

  when act

  then emits stdout
    assert output contains \"b\"
";

const SPEC_C_DEPENDS_B: &str = "\
spec c-spec v1.0.0
title \"Spec C\"

description
  Spec C also depends on B.

motivation
  Reverse dependency testing.

behavior do-c [happy_path]
  \"Do C\"

  given
    B is available

  when act

  then emits stdout
    assert output contains \"c\"

depends on b-spec >= 1.0.0
";

const SPEC_B_DEPENDS_C: &str = "\
spec b-spec v1.2.0
title \"Spec B\"

description
  Spec B depends on C.

motivation
  Chain dependency.

behavior do-b [happy_path]
  \"Do B\"

  given
    C is available

  when act

  then emits stdout
    assert output contains \"b\"

depends on c-spec >= 1.0.0
";

const SPEC_C_NO_DEPS: &str = "\
spec c-spec v1.0.0
title \"Spec C\"

description
  Spec C has no dependencies.

motivation
  Leaf node.

behavior do-c [happy_path]
  \"Do C\"

  given
    System ready

  when act

  then emits stdout
    assert output contains \"c\"
";

const SPEC_STANDALONE: &str = "\
spec standalone v1.0.0
title \"Standalone Spec\"

description
  A spec with no dependencies.

motivation
  Testing inspect.

behavior do-thing [happy_path]
  \"Do the thing\"

  given
    System ready

  when act

  then emits stdout
    assert output contains \"done\"
";

// Rich spec for inspect testing: multiple categories, assertions, deps
const SPEC_RICH: &str = "\
spec my-feature v1.2.0
title \"My Feature\"

description
  A rich spec for inspect testing.

motivation
  Test metadata extraction.

behavior create-item [happy_path]
  \"Create an item\"

  given
    User is authenticated

  when create
    item_name = \"widget\"

  then returns item
    assert id is_present
    assert name == \"widget\"

behavior create-duplicate [happy_path]
  \"Handle duplicate creation\"

  given
    User is authenticated
    Item already exists

  when create
    item_name = \"widget\"

  then returns item
    assert id is_present

behavior list-items [happy_path]
  \"List all items\"

  given
    User is authenticated

  when list

  then returns items
    assert items contains \"widget\"

behavior list-empty [happy_path]
  \"List items when none exist\"

  given
    User is authenticated
    No items exist

  when list

  then returns items
    assert count == 0

behavior unauthorized-create [error_case]
  \"Reject create when not authenticated\"

  given
    User is not authenticated

  when create
    item_name = \"widget\"

  then emits stderr
    assert error contains \"unauthorized\"

behavior unauthorized-list [error_case]
  \"Reject list when not authenticated\"

  given
    User is not authenticated

  when list

  then emits stderr
    assert error contains \"unauthorized\"

behavior empty-name [edge_case]
  \"Reject create with empty name\"

  given
    User is authenticated

  when create
    item_name = \"\"

  then emits stderr
    assert error contains \"name required\"

depends on user-auth >= 1.0.0
";

// NFR with mixed constraint types for inspect testing
const NFR_RICH: &str = "\
nfr performance v1.0.0
title \"Performance Requirements\"

description
  Rich NFR for testing inspect.

motivation
  Inspect metadata testing.


constraint api-response-time [metric]
  \"API endpoints respond within bounds\"

  metric \"HTTP response time, p95\"
  threshold < 1s

  verification
    environment staging
    benchmark \"load test\"
    pass \"p95 < threshold\"

  violation high
  overridable yes

constraint throughput [metric]
  \"System handles concurrent load\"

  metric \"Requests per second\"
  threshold >= 100

  verification
    environment staging
    benchmark \"concurrent requests\"
    pass \"rps >= threshold\"

  violation high
  overridable yes

constraint write-throughput [metric]
  \"Write path handles load\"

  metric \"Writes per second\"
  threshold >= 50

  verification
    environment staging
    benchmark \"concurrent writes\"
    pass \"wps >= threshold\"

  violation medium
  overridable yes

constraint no-blocking-io [rule]
  \"No blocking I/O in request path\"

  rule
    All I/O in the HTTP request path must be async.

  verification
    static \"code review for sync I/O calls\"

  violation high
  overridable no
";

// ════════════════════════════════════════════════════════
// Server lifecycle tests (mcp-server.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e initialize-server
#[test]
/// mcp-server: initialize-server
fn initialize_server() {
    let mut client = McpClient::new();
    let info = client.server_info();
    let name = info["serverInfo"]["name"].as_str().unwrap();
    let version = info["serverInfo"]["version"].as_str().unwrap();
    assert_eq!(name, "minter");
    assert!(
        version.split('.').count() >= 3,
        "version should be semver: {}",
        version
    );
    assert!(
        info["capabilities"]["tools"].is_object(),
        "capabilities should include tools"
    );
}

// @minter:e2e list-tools
#[test]
/// mcp-server: list-tools
fn list_tools() {
    let mut client = McpClient::new();
    let tools = client.list_tools();
    let tool_names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert!(tool_names.contains(&"validate"), "missing validate tool");
    assert!(tool_names.contains(&"inspect"), "missing inspect tool");
    assert!(tool_names.contains(&"scaffold"), "missing scaffold tool");
    assert!(tool_names.contains(&"format"), "missing format tool");
    assert!(tool_names.contains(&"graph"), "missing graph tool");
    assert!(tool_names.contains(&"coverage"), "missing coverage tool");

    // Each tool has description and inputSchema
    for tool in tools.as_array().unwrap() {
        assert!(
            tool["description"].is_string(),
            "tool {} missing description",
            tool["name"]
        );
        assert!(
            tool["inputSchema"].is_object(),
            "tool {} missing inputSchema",
            tool["name"]
        );
    }
}

// ════════════════════════════════════════════════════════
// Validate tool tests (mcp-server.spec + mcp-response-format.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e validate-file-pass
#[test]
/// mcp-server: validate-file-pass
fn validate_file_pass() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "test-spec", VALID_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["results"][0]["status"], "pass");
    assert_eq!(data["results"][0]["name"], "test-spec");
    assert!(data["results"][0]["behavior_count"].as_u64().unwrap() > 0);
    assert!(data["results"][0]["errors"].as_array().unwrap().is_empty());
}

// @minter:e2e validate-file-fail
#[test]
/// mcp-server: validate-file-fail
fn validate_file_fail() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["results"][0]["status"], "fail");
    assert!(!data["results"][0]["errors"].as_array().unwrap().is_empty());
}

// @minter:e2e validate-directory
#[test]
/// mcp-server: validate-directory
fn validate_directory() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "test-spec", VALID_SPEC);
    write_nfr(&dir, "performance", VALID_NFR);

    let result = client.call_tool("validate", json!({ "path": dir.path().to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    let names: Vec<&str> = data["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"test-spec"));
    assert!(names.contains(&"performance"));
}

// @minter:e2e validate-deep-mode
#[test]
/// mcp-server: validate-deep-mode
fn validate_deep_mode() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B);

    let a_path = dir.path().join("a-spec.spec");
    let result = client.call_tool(
        "validate",
        json!({ "path": a_path.to_str().unwrap(), "deep": true }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    let names: Vec<&str> = data["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"a-spec"), "should contain a-spec");
    assert!(names.contains(&"b-spec"), "should contain b-spec");
}

// @minter:e2e validate-nfr-file
#[test]
/// mcp-server: validate-nfr-file
fn validate_nfr_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_nfr(&dir, "performance", VALID_NFR);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["results"][0]["status"], "pass");
    assert_eq!(data["results"][0]["name"], "performance");
    assert!(data["results"][0]["constraint_count"].as_u64().unwrap() > 0);
}

// @minter:e2e validate-inline-content
#[test]
/// mcp-server: validate-inline-content
fn validate_inline_content() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "validate",
        json!({ "content": VALID_SPEC, "content_type": "spec" }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["results"][0]["status"], "pass");
    assert_eq!(data["results"][0]["name"], "test-spec");
}

// @minter:e2e validate-nonexistent-path
#[test]
/// mcp-server: validate-nonexistent-path
fn validate_nonexistent_path() {
    let mut client = McpClient::new();
    let result = client.call_tool("validate", json!({ "path": "/nonexistent/path.spec" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("nonexistent"), "error should mention path");
}

// @minter:e2e validate-mixed-results
#[test]
/// mcp-server: validate-mixed-results
fn validate_mixed_results() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "valid-spec", VALID_SPEC);
    write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("validate", json!({ "path": dir.path().to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    let results = data["results"].as_array().unwrap();

    let has_pass = results.iter().any(|r| r["status"] == "pass");
    let has_fail = results.iter().any(|r| r["status"] == "fail");
    assert!(has_pass, "should have a passing result");
    assert!(has_fail, "should have a failing result");
}

// ── Validate result structure (mcp-response-format.spec) ────

// @minter:e2e validate-pass-result-fields
#[test]
/// mcp-response-format: validate-pass-result-fields
fn validate_pass_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "test-spec", VALID_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let r = &data["results"][0];

    assert!(r["file"].is_string(), "file should be present");
    assert_eq!(r["name"], "test-spec");
    assert_eq!(r["version"], "1.0.0");
    assert_eq!(r["type"], "spec");
    assert_eq!(r["status"], "pass");
    assert!(
        r["behavior_count"].is_number(),
        "behavior_count should be present"
    );
    assert!(r["errors"].as_array().unwrap().is_empty());
}

// @minter:e2e validate-nfr-result-fields
#[test]
/// mcp-response-format: validate-nfr-result-fields
fn validate_nfr_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_nfr(&dir, "performance", VALID_NFR);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let r = &data["results"][0];

    assert_eq!(r["name"], "performance");
    assert_eq!(r["version"], "1.0.0");
    assert_eq!(r["type"], "nfr");
    assert_eq!(r["status"], "pass");
    assert!(
        r["constraint_count"].is_number(),
        "constraint_count should be present"
    );
    // NFR should not have behavior_count
    assert!(r.get("behavior_count").is_none() || r["behavior_count"].is_null());
}

// @minter:e2e validate-summary-fields
#[test]
/// mcp-response-format: validate-summary-fields
fn validate_summary_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "valid-spec", VALID_SPEC);
    write_spec(&dir, "broken", BROKEN_SPEC);
    write_nfr(&dir, "performance", VALID_NFR);

    let result = client.call_tool("validate", json!({ "path": dir.path().to_str().unwrap() }));
    let data = json_content(&result);
    let summary = &data["summary"];

    assert_eq!(summary["total"].as_u64().unwrap(), 3);
    assert_eq!(summary["passed"].as_u64().unwrap(), 2);
    assert_eq!(summary["failed"].as_u64().unwrap(), 1);
}

// @minter:e2e validate-deep-dependency-fields
#[test]
/// mcp-response-format: validate-deep-dependency-fields
fn validate_deep_dependency_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B);

    let a_path = dir.path().join("a-spec.spec");
    let result = client.call_tool(
        "validate",
        json!({ "path": a_path.to_str().unwrap(), "deep": true }),
    );
    let data = json_content(&result);
    let results = data["results"].as_array().unwrap();

    let a_result = results.iter().find(|r| r["name"] == "a-spec").unwrap();
    let deps = a_result["dependencies"].as_array().unwrap();
    assert_eq!(deps[0]["name"], "b-spec");
    assert_eq!(deps[0]["constraint"], ">= 1.0.0");

    let b_result = results.iter().find(|r| r["name"] == "b-spec").unwrap();
    let b_deps = b_result["dependencies"].as_array().unwrap();
    assert!(b_deps.is_empty(), "b should have empty dependencies");
}

// @minter:e2e validate-inline-result-omits-file
#[test]
/// mcp-response-format: validate-inline-result-omits-file
fn validate_inline_result_omits_file() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "validate",
        json!({ "content": VALID_SPEC, "content_type": "spec" }),
    );
    let data = json_content(&result);
    let r = &data["results"][0];

    assert!(
        r.get("file").is_none() || r["file"].is_null(),
        "inline result should omit file"
    );
    assert!(r["name"].is_string(), "name should be present");
    assert!(r["status"].is_string(), "status should be present");
}

// ── Error object structure (mcp-response-format.spec) ────

// @minter:e2e error-object-fields
#[test]
/// mcp-response-format: error-object-fields
fn error_object_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let errors = data["results"][0]["errors"].as_array().unwrap();
    assert!(!errors.is_empty(), "should have errors");

    let err = &errors[0];
    assert!(err["line"].is_number(), "error should have line");
    assert!(err["message"].is_string(), "error should have message");
}

// @minter:e2e error-object-includes-file-path
#[test]
/// mcp-response-format: error-object-includes-file-path
fn error_object_includes_file_path() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let errors = data["results"][0]["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    let err = &errors[0];
    assert!(
        err["file"].is_string(),
        "error for file validation should include file path"
    );
}

// @minter:e2e validate-fail-result-fields
#[test]
/// mcp-response-format: validate-fail-result-fields
fn validate_fail_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    // Use a spec that parses but fails semantic validation (no happy_path)
    let sem_fail = "\
spec sem-fail v2.0.0
title \"Semantic Failure\"

description
  Fails semantic validation.

motivation
  Testing fail result fields.

behavior only-error [error_case]
  \"Only error case, no happy_path\"

  given
    Ready

  when act

  then emits stderr
    assert output contains \"error\"
";
    let path = write_spec(&dir, "sem-fail", sem_fail);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let r = &data["results"][0];

    assert_eq!(r["name"], "sem-fail");
    assert_eq!(r["version"], "2.0.0");
    assert_eq!(r["status"], "fail");
    assert!(
        !r["errors"].as_array().unwrap().is_empty(),
        "fail result should have non-empty errors"
    );
}

// @minter:e2e tool-error-structure
#[test]
/// mcp-response-format: tool-error-structure
fn tool_error_structure() {
    let mut client = McpClient::new();
    let result = client.call_tool("validate", json!({ "path": "/nonexistent/missing.spec" }));
    assert!(is_error(&result), "should be a tool error");
    let text = text_content(&result);
    assert!(!text.is_empty(), "error text should describe the failure");
    assert!(
        text.contains("/nonexistent/missing.spec"),
        "error text should include the input path, got: {text}"
    );
}

// @minter:e2e tool-error-lists-valid-options
#[test]
/// mcp-response-format: tool-error-lists-valid-options
fn tool_error_lists_valid_options() {
    let mut client = McpClient::new();
    let result = client.call_tool("format", json!({ "type": "banana" }));
    assert!(is_error(&result), "should be a tool error");
    let text = text_content(&result);
    assert!(
        text.contains("banana"),
        "error should mention the invalid input"
    );
    assert!(
        text.contains("spec"),
        "error should list 'spec' as valid option"
    );
    assert!(
        text.contains("nfr"),
        "error should list 'nfr' as valid option"
    );
}

// ════════════════════════════════════════════════════════
// Inspect tool tests (mcp-server.spec + mcp-response-format.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e inspect-spec-file
#[test]
/// mcp-server: inspect-spec-file
fn inspect_spec_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "my-feature", SPEC_RICH);
    write_spec(&dir, "user-auth", SPEC_STANDALONE); // dependency target not needed for inspect

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["name"], "my-feature");
    assert!(data["categories"].is_object());
    assert!(data["dependencies"].is_array());
}

// @minter:e2e inspect-nfr-file
#[test]
/// mcp-server: inspect-nfr-file
fn inspect_nfr_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_nfr(&dir, "performance", NFR_RICH);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["name"], "performance");
    assert!(data["types"].is_object());
    assert_eq!(data["types"]["metric"].as_u64().unwrap(), 3);
    assert_eq!(data["types"]["rule"].as_u64().unwrap(), 1);
}

// @minter:e2e inspect-inline-content
#[test]
/// mcp-server: inspect-inline-content
fn inspect_inline_content() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "inspect",
        json!({ "content": VALID_SPEC, "content_type": "spec" }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["name"], "test-spec");
}

// @minter:e2e inspect-invalid-file
#[test]
/// mcp-server: inspect-invalid-file
fn inspect_invalid_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    assert!(
        is_error(&result),
        "inspecting invalid file should return error"
    );
}

// @minter:e2e mcp-server/inspect-nonexistent-file
#[test]
fn inspect_nonexistent_file() {
    let mut client = McpClient::new();
    let result = client.call_tool("inspect", json!({ "path": "/nonexistent/missing.spec" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("missing.spec"));
}

// ── Inspect result structure (mcp-response-format.spec) ────

// @minter:e2e inspect-spec-result-fields
#[test]
/// mcp-response-format: inspect-spec-result-fields
fn inspect_spec_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "my-feature", SPEC_RICH);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);

    assert_eq!(data["name"], "my-feature");
    assert_eq!(data["type"], "spec");
    assert_eq!(data["behavior_count"].as_u64().unwrap(), 7);
    assert_eq!(data["categories"]["happy_path"].as_u64().unwrap(), 4);
    assert_eq!(data["categories"]["error_case"].as_u64().unwrap(), 2);
    assert_eq!(data["categories"]["edge_case"].as_u64().unwrap(), 1);

    let deps = data["dependencies"].as_array().unwrap();
    assert_eq!(deps[0]["name"], "user-auth");
    assert_eq!(deps[0]["constraint"], ">= 1.0.0");

    let assertion_types = data["assertion_types"].as_array().unwrap();
    let at_strings: Vec<&str> = assertion_types
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(at_strings.contains(&"equals"));
    assert!(at_strings.contains(&"contains"));
    assert!(at_strings.contains(&"is_present"));
}

// @minter:e2e inspect-nfr-result-fields
#[test]
/// mcp-response-format: inspect-nfr-result-fields
fn inspect_nfr_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_nfr(&dir, "performance", NFR_RICH);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);

    assert_eq!(data["name"], "performance");
    assert_eq!(data["type"], "nfr");
    assert_eq!(data["category"], "performance");
    assert_eq!(data["constraint_count"].as_u64().unwrap(), 4);
    assert_eq!(data["types"]["metric"].as_u64().unwrap(), 3);
    assert_eq!(data["types"]["rule"].as_u64().unwrap(), 1);
}

// @minter:e2e inspect-no-dependencies-field
#[test]
/// mcp-response-format: inspect-no-dependencies-field
fn inspect_no_dependencies_field() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "standalone", SPEC_STANDALONE);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let deps = data["dependencies"].as_array().unwrap();
    assert!(
        deps.is_empty(),
        "standalone spec should have empty dependencies"
    );
}

// ════════════════════════════════════════════════════════
// Scaffold tool tests (mcp-server.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e scaffold-spec-template
#[test]
/// mcp-server: scaffold-spec-template
fn scaffold_spec_template() {
    let mut client = McpClient::new();
    let result = client.call_tool("scaffold", json!({ "type": "spec" }));
    assert!(!is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains("spec"),
        "spec skeleton should contain spec keyword"
    );
    assert!(
        text.contains("behavior"),
        "spec skeleton should contain behavior keyword"
    );
}

// @minter:e2e scaffold-nfr-template
#[test]
/// mcp-server: scaffold-nfr-template
fn scaffold_nfr_template() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "scaffold",
        json!({ "type": "nfr", "category": "performance" }),
    );
    assert!(!is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains("performance"),
        "NFR skeleton should contain category"
    );
    assert!(
        text.contains("constraint"),
        "NFR skeleton should contain constraint keyword"
    );
}

// @minter:e2e scaffold-unknown-category
#[test]
/// mcp-server: scaffold-unknown-category
fn scaffold_unknown_category() {
    let mut client = McpClient::new();
    let result = client.call_tool("scaffold", json!({ "type": "nfr", "category": "banana" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("banana"));
    // Should list valid categories
    assert!(text.contains("performance") || text.contains("security"));
}

// ════════════════════════════════════════════════════════
// Format tool tests (mcp-server.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e format-spec-grammar
#[test]
/// mcp-server: format-spec-grammar
fn format_spec_grammar() {
    let mut client = McpClient::new();
    let result = client.call_tool("format", json!({ "type": "spec" }));
    assert!(!is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains("spec"),
        "spec grammar should contain spec keyword"
    );
    assert!(
        text.contains("behavior"),
        "spec grammar should contain behavior keyword"
    );
}

// @minter:e2e format-nfr-grammar
#[test]
/// mcp-server: format-nfr-grammar
fn format_nfr_grammar() {
    let mut client = McpClient::new();
    let result = client.call_tool("format", json!({ "type": "nfr" }));
    assert!(!is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains("nfr"),
        "NFR grammar should contain nfr keyword"
    );
    assert!(
        text.contains("constraint"),
        "NFR grammar should contain constraint keyword"
    );
}

// @minter:e2e format-unknown-type
#[test]
/// mcp-server: format-unknown-type
fn format_unknown_type() {
    let mut client = McpClient::new();
    let result = client.call_tool("format", json!({ "type": "banana" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("banana"));
    assert!(text.contains("spec") && text.contains("nfr"));
}

// ════════════════════════════════════════════════════════
// Graph tool tests (mcp-server.spec + mcp-response-format.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e graph-full-dependencies
#[test]
/// mcp-server: graph-full-dependencies
fn graph_full_dependencies() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B_DEPENDS_C);
    write_spec(&dir, "c-spec", SPEC_C_NO_DEPS);

    let result = client.call_tool("graph", json!({ "path": dir.path().to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);

    let specs = data["specs"].as_array().unwrap();
    let spec_names: Vec<&str> = specs.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert!(spec_names.contains(&"a-spec"));
    assert!(spec_names.contains(&"b-spec"));
    assert!(spec_names.contains(&"c-spec"));

    let edges = data["edges"].as_array().unwrap();
    let has_a_to_b = edges
        .iter()
        .any(|e| e["from"] == "a-spec" && e["to"] == "b-spec");
    let has_b_to_c = edges
        .iter()
        .any(|e| e["from"] == "b-spec" && e["to"] == "c-spec");
    assert!(has_a_to_b, "should have edge from a to b");
    assert!(has_b_to_c, "should have edge from b to c");
}

// @minter:e2e graph-impacted-specs
#[test]
/// mcp-server: graph-impacted-specs
fn graph_impacted_specs() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B);
    write_spec(&dir, "c-spec", SPEC_C_DEPENDS_B);

    let result = client.call_tool(
        "graph",
        json!({ "path": dir.path().to_str().unwrap(), "impacted": "b-spec" }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);

    assert_eq!(data["target"], "b-spec");
    let impacted: Vec<&str> = data["impacted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(impacted.contains(&"a-spec"));
    assert!(impacted.contains(&"c-spec"));
}

// @minter:e2e graph-unknown-spec
#[test]
/// mcp-server: graph-unknown-spec
fn graph_unknown_spec() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B);

    let result = client.call_tool(
        "graph",
        json!({ "path": dir.path().to_str().unwrap(), "impacted": "nonexistent" }),
    );
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("nonexistent"));
}

// @minter:e2e graph-empty-directory
#[test]
/// mcp-server: graph-empty-directory
fn graph_empty_directory() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();

    let result = client.call_tool("graph", json!({ "path": dir.path().to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("no spec files found"));
}

// ── Graph result structure (mcp-response-format.spec) ────

// @minter:e2e graph-full-result-fields
#[test]
/// mcp-response-format: graph-full-result-fields
fn graph_full_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B_DEPENDS_C);
    write_spec(&dir, "c-spec", SPEC_C_NO_DEPS);

    let result = client.call_tool("graph", json!({ "path": dir.path().to_str().unwrap() }));
    let data = json_content(&result);

    // Check spec entry fields
    for spec in data["specs"].as_array().unwrap() {
        assert!(spec["name"].is_string());
        assert!(spec["file"].is_string());
        assert!(spec["version"].is_string());
    }

    // Check edge fields
    for edge in data["edges"].as_array().unwrap() {
        assert!(edge["from"].is_string());
        assert!(edge["to"].is_string());
        assert!(edge["constraint"].is_string());
    }

    // Check specific edge constraint format
    let a_to_b = data["edges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["from"] == "a-spec")
        .unwrap();
    assert_eq!(a_to_b["constraint"], ">= 1.0.0");
}

// @minter:e2e graph-impacted-result-fields
#[test]
/// mcp-response-format: graph-impacted-result-fields
fn graph_impacted_result_fields() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B);
    write_spec(&dir, "c-spec", SPEC_C_DEPENDS_B);

    let result = client.call_tool(
        "graph",
        json!({ "path": dir.path().to_str().unwrap(), "impacted": "b-spec" }),
    );
    let data = json_content(&result);

    assert_eq!(data["target"], "b-spec");
    for entry in data["impacted"].as_array().unwrap() {
        assert!(entry["name"].is_string());
        assert!(entry["file"].is_string());
        assert!(entry["version"].is_string());
    }
}

// @minter:e2e graph-no-edges-result
#[test]
/// mcp-response-format: graph-no-edges-result
fn graph_no_edges_result() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    write_spec(&dir, "standalone", SPEC_STANDALONE);
    write_spec(&dir, "test-spec", VALID_SPEC);

    let result = client.call_tool("graph", json!({ "path": dir.path().to_str().unwrap() }));
    assert!(!is_error(&result));
    let data = json_content(&result);

    assert!(data["specs"].as_array().unwrap().len() >= 2);
    assert!(data["edges"].as_array().unwrap().is_empty());
}

// ════════════════════════════════════════════════════════
// Agent guidance tests (mcp-agent-guidance.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e initialize-includes-methodology-instructions
#[test]
/// mcp-agent-guidance: initialize-includes-methodology-instructions
fn initialize_includes_methodology_instructions() {
    let mut client = McpClient::new();
    let info = client.server_info();
    let instructions = info["instructions"].as_str().unwrap_or("");
    // Should contain workflow guidance
    assert!(
        instructions.to_lowercase().contains("spec"),
        "instructions should mention specs"
    );
    assert!(
        instructions.contains("initialize_minter"),
        "instructions should mention initialize_minter"
    );
}

// @minter:e2e initialize-minter-returns-full-methodology
#[test]
/// mcp-agent-guidance: initialize-minter-returns-full-methodology
fn initialize_minter_returns_full_methodology() {
    let mut client = McpClient::new();
    let result = client.call_tool("initialize_minter", json!({}));
    assert!(!is_error(&result));
    let text = text_content(&result);

    assert!(
        text.to_lowercase().contains("source of truth"),
        "should contain 'source of truth'"
    );
    assert!(text.to_lowercase().contains("tdd"), "should mention TDD");
}

// @minter:e2e initialize-minter-lists-available-tools
#[test]
/// mcp-agent-guidance: initialize-minter-lists-available-tools
fn initialize_minter_lists_available_tools() {
    let mut client = McpClient::new();
    let result = client.call_tool("initialize_minter", json!({}));
    let text = text_content(&result);

    assert!(text.contains("validate"), "should list validate tool");
    assert!(text.contains("scaffold"), "should list scaffold tool");
    assert!(text.contains("format"), "should list format tool");
    assert!(text.contains("inspect"), "should list inspect tool");
    assert!(text.contains("graph"), "should list graph tool");
}

// @minter:e2e guide-workflow-phases
#[test]
/// mcp-agent-guidance: guide-workflow-phases
fn guide_workflow_phases() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "workflow" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    // Check that key phases are mentioned
    assert!(text.contains("spec") || text.contains("Spec"));
    assert!(text.contains("test") || text.contains("Test"));
}

// @minter:e2e guide-spec-authoring
#[test]
/// mcp-agent-guidance: guide-spec-authoring
fn guide_spec_authoring() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "authoring" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    // Should cover authoring topics
    assert!(
        text.contains("granularity") || text.contains("behavior"),
        "should discuss behavior granularity"
    );
    // Expanded content: coarse/fine signals and project type calibration
    assert!(
        text.contains("Too Coarse") || text.contains("too coarse"),
        "should describe too-coarse signals"
    );
    assert!(
        text.contains("Too Fine") || text.contains("too fine"),
        "should describe too-fine signals"
    );
    assert!(
        text.contains("API") && text.contains("CLI"),
        "should describe project type calibration"
    );
}

// @minter:e2e guide-requirements-smells
#[test]
/// mcp-agent-guidance: guide-requirements-smells
fn guide_requirements_smells() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "smells" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    assert!(
        text.contains("Ambiguity"),
        "should describe ambiguity smell"
    );
    assert!(
        text.contains("NASA") || text.contains("forbidden"),
        "should describe NASA forbidden words"
    );
    assert!(
        text.contains("Compound") || text.contains("compound"),
        "should describe compound behavior smell"
    );
    assert!(
        text.contains("Implementation") || text.contains("leakage"),
        "should describe implementation leakage smell"
    );
    // Each smell should have signal + action
    assert!(text.contains("Signal") || text.contains("signal"));
    assert!(text.contains("Action") || text.contains("action"));
}

// @minter:e2e guide-nfr-design
#[test]
/// mcp-agent-guidance: guide-nfr-design
fn guide_nfr_design() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "nfr" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    // Seven categories
    assert!(
        text.contains("performance"),
        "should list performance category"
    );
    assert!(
        text.contains("reliability"),
        "should list reliability category"
    );
    assert!(text.contains("security"), "should list security category");
    assert!(
        text.contains("observability"),
        "should list observability category"
    );
    assert!(
        text.contains("scalability"),
        "should list scalability category"
    );
    assert!(text.contains("cost"), "should list cost category");
    assert!(
        text.contains("operability"),
        "should list operability category"
    );

    // Constraint types
    assert!(
        text.contains("Metric") || text.contains("metric"),
        "should describe metric constraint type"
    );
    assert!(
        text.contains("Rule") || text.contains("rule"),
        "should describe rule constraint type"
    );

    // Three-level referencing
    assert!(
        text.contains("spec-level") || text.contains("Spec-level"),
        "should describe spec-level referencing"
    );
    assert!(
        text.contains("behavior-level") || text.contains("Behavior-level"),
        "should describe behavior-level referencing"
    );

    // Coverage gap detection
    assert!(
        text.contains("Coverage") || text.contains("coverage") || text.contains("gap"),
        "should describe coverage gap detection"
    );

    // FR/NFR classification
    assert!(
        text.contains("Classification") || text.contains("classification"),
        "should describe FR/NFR classification"
    );

    // Override rules
    assert!(
        text.contains("Override") || text.contains("override"),
        "should describe override rules"
    );
}

// @minter:e2e guide-context-management
#[test]
/// mcp-agent-guidance: guide-context-management
fn guide_context_management() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "context" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    assert!(
        text.contains("Lazy Loading") || text.contains("lazy loading"),
        "should describe the lazy loading sequence"
    );
    assert!(
        text.contains("Guard Rails") || text.contains("guard rails"),
        "should describe guard rails"
    );
    assert!(
        text.contains("subgraph"),
        "should mention scoping to subgraph"
    );
    assert!(
        text.contains("Structure before content"),
        "should describe structure-before-content principle"
    );
}

// @minter:e2e mcp-agent-guidance/guide-unknown-topic
#[test]
fn guide_unknown_topic() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "banana" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("banana"));
    assert!(text.contains("workflow"));
    assert!(text.contains("authoring"));
    assert!(text.contains("smells"), "should list smells as valid topic");
    assert!(text.contains("nfr"), "should list nfr as valid topic");
    assert!(
        text.contains("context"),
        "should list context as valid topic"
    );
    assert!(
        text.contains("methodology"),
        "should list methodology as valid topic"
    );
    assert!(
        text.contains("coverage"),
        "should list coverage as valid topic"
    );
}

// @minter:e2e guide-coverage-tagging
#[test]
/// mcp-agent-guidance: guide-coverage-tagging
fn guide_coverage_tagging() {
    let mut client = McpClient::new();
    let result = client.call_tool("guide", json!({ "topic": "coverage" }));
    assert!(!is_error(&result));
    let text = text_content(&result);

    assert!(
        text.contains("Coverage Tagging"),
        "should contain Coverage Tagging heading"
    );
    assert!(text.contains("@minter"), "should reference @minter tag");
    assert!(text.contains("unit"), "should mention unit type");
    assert!(text.contains("e2e"), "should mention e2e type");
    assert!(text.contains("benchmark"), "should mention benchmark type");
    assert!(
        text.contains("Qualified Names"),
        "should describe qualified names"
    );
    assert!(
        text.contains("Common Mistakes"),
        "should list common mistakes"
    );
}

// @minter:e2e list-tools-includes-agent-guidance
#[test]
/// mcp-agent-guidance: list-tools-includes-agent-guidance
fn list_tools_includes_agent_guidance() {
    let mut client = McpClient::new();
    let tools = client.list_tools();
    let tool_names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert!(
        tool_names.contains(&"initialize_minter"),
        "should list initialize_minter"
    );
    assert!(tool_names.contains(&"guide"), "should list guide");

    // Check descriptions
    let init_tool = tools
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "initialize_minter")
        .unwrap();
    assert!(
        init_tool["description"]
            .as_str()
            .unwrap()
            .contains("MUST be called before"),
        "initialize_minter description should contain 'MUST be called before'"
    );
}

// ════════════════════════════════════════════════════════
// Next-steps tests (mcp-agent-guidance.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e next-steps-after-scaffold
#[test]
/// mcp-agent-guidance: next-steps-after-scaffold
fn next_steps_after_scaffold() {
    let mut client = McpClient::new();
    let result = client.call_tool("scaffold", json!({ "type": "spec" }));
    let text = text_content(&result);

    assert!(
        text.contains("fill in behaviors for each user-observable outcome"),
        "should contain next step for scaffold"
    );
    assert!(
        text.contains("validate the spec with the validate tool"),
        "should contain validate next step"
    );
}

// @minter:e2e next-steps-after-validate-pass
#[test]
/// mcp-agent-guidance: next-steps-after-validate-pass
fn next_steps_after_validate_pass() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "test-spec", VALID_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let next_steps = data["next_steps"].as_array().unwrap();
    let steps: Vec<&str> = next_steps.iter().map(|s| s.as_str().unwrap()).collect();

    assert!(steps.contains(&"write one e2e test per behavior"));
    assert!(steps.contains(&"tests must fail (red) before implementation"));
}

// @minter:e2e next-steps-after-validate-fail
#[test]
/// mcp-agent-guidance: next-steps-after-validate-fail
fn next_steps_after_validate_fail() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "broken", BROKEN_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let next_steps = data["next_steps"].as_array().unwrap();
    let steps: Vec<&str> = next_steps.iter().map(|s| s.as_str().unwrap()).collect();

    assert!(steps.contains(&"fix the errors listed above"));
    assert!(steps.contains(&"re-validate"));
}

// @minter:e2e next-steps-after-format
#[test]
/// mcp-agent-guidance: next-steps-after-format
fn next_steps_after_format() {
    let mut client = McpClient::new();
    let result = client.call_tool("format", json!({ "type": "spec" }));
    let text = text_content(&result);
    assert!(text.contains("use this grammar to write your spec"));
}

// @minter:e2e next-steps-after-inspect
#[test]
/// mcp-agent-guidance: next-steps-after-inspect
fn next_steps_after_inspect() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    // Use a spec with only happy_path (no error_case)
    let path = write_spec(&dir, "standalone", SPEC_STANDALONE);

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);
    let next_steps = data["next_steps"].as_array().unwrap();
    let steps: Vec<&str> = next_steps.iter().map(|s| s.as_str().unwrap()).collect();

    assert!(steps.contains(&"add error_case behaviors for each happy path"));
}

// @minter:e2e next-steps-after-nfr-scaffold
#[test]
/// mcp-agent-guidance: next-steps-after-nfr-scaffold
fn next_steps_after_nfr_scaffold() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "scaffold",
        json!({ "type": "nfr", "category": "performance" }),
    );
    let text = text_content(&result);

    assert!(text.contains("define metric or rule constraints"));
    assert!(text.contains("reference from functional specs using nfr section"));
}

// ════════════════════════════════════════════════════════
// Cross-cutting format rules (mcp-response-format.spec)
// ════════════════════════════════════════════════════════

// @minter:e2e no-ansi-in-responses
#[test]
/// mcp-response-format: no-ansi-in-responses
fn no_ansi_in_responses() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "test-spec", VALID_SPEC);

    // Test validate
    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let text = text_content(&result);
    assert!(
        !text.contains("\x1b["),
        "validate response should not contain ANSI escapes"
    );

    // Test scaffold
    let result = client.call_tool("scaffold", json!({ "type": "spec" }));
    let text = text_content(&result);
    assert!(
        !text.contains("\x1b["),
        "scaffold response should not contain ANSI escapes"
    );

    // Test format
    let result = client.call_tool("format", json!({ "type": "spec" }));
    let text = text_content(&result);
    assert!(
        !text.contains("\x1b["),
        "format response should not contain ANSI escapes"
    );

    // Test initialize_minter
    let result = client.call_tool("initialize_minter", json!({}));
    let text = text_content(&result);
    assert!(
        !text.contains("\x1b["),
        "initialize_minter response should not contain ANSI escapes"
    );
}

// @minter:e2e snake-case-field-names
#[test]
/// mcp-response-format: snake-case-field-names
fn snake_case_field_names() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "test-spec", VALID_SPEC);

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    let data = json_content(&result);

    fn check_keys(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in map.keys() {
                    // snake_case: lowercase, digits, underscores only
                    assert!(
                        key.chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                        "field name '{}' is not snake_case",
                        key
                    );
                    check_keys(&map[key]);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    check_keys(item);
                }
            }
            _ => {}
        }
    }

    check_keys(&data);
}

// @minter:e2e consistent-count-field-naming
#[test]
/// mcp-response-format: consistent-count-field-naming
fn consistent_count_naming() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let spec_path = write_spec(&dir, "test-spec", VALID_SPEC);
    let nfr_path = write_nfr(&dir, "performance", VALID_NFR);

    // Check spec result
    let result = client.call_tool("validate", json!({ "path": spec_path.to_str().unwrap() }));
    let data = json_content(&result);
    let spec_result = &data["results"][0];
    assert!(spec_result["behavior_count"].is_number());
    assert!(
        spec_result.get("constraint_count").is_none() || spec_result["constraint_count"].is_null(),
        "spec should not have constraint_count"
    );

    // Check NFR result
    let result = client.call_tool("validate", json!({ "path": nfr_path.to_str().unwrap() }));
    let data = json_content(&result);
    let nfr_result = &data["results"][0];
    assert!(nfr_result["constraint_count"].is_number());
    assert!(
        nfr_result.get("behavior_count").is_none() || nfr_result["behavior_count"].is_null(),
        "nfr should not have behavior_count"
    );
}

// @minter:e2e text-content-format
#[test]
/// mcp-response-format: text-content-format
fn text_content_format() {
    let mut client = McpClient::new();

    // Scaffold returns plain text
    let result = client.call_tool("scaffold", json!({ "type": "spec" }));
    let text = text_content(&result);
    assert!(
        !text.starts_with('{'),
        "scaffold should return plain text, not JSON"
    );

    // Format returns plain text
    let result = client.call_tool("format", json!({ "type": "spec" }));
    let text = text_content(&result);
    assert!(
        !text.starts_with('{'),
        "format should return plain text, not JSON"
    );
}

// ════════════════════════════════════════════════════════
// Security hardening tests (mcp-server.spec v1.3.0)
// ════════════════════════════════════════════════════════

// @minter:e2e validate-reject-non-spec-extension
#[test]
/// mcp-server: validate-reject-non-spec-extension
fn validate_reject_non_spec_extension() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("readme.md");
    std::fs::write(&path, "# Hello").unwrap();

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains(".spec") || text.contains(".nfr"),
        "error should mention valid extensions"
    );
}

// @minter:e2e validate-reject-oversized-file
#[test]
/// mcp-server: validate-reject-oversized-file
fn validate_reject_oversized_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.spec");
    // Write 11MB file
    let data = "x".repeat(11 * 1024 * 1024);
    std::fs::write(&path, data).unwrap();

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("10MB"), "error should mention 10MB limit");
}

// @minter:e2e validate-reject-oversized-content
#[test]
/// mcp-server: validate-reject-oversized-content
fn validate_reject_oversized_content() {
    let mut client = McpClient::new();
    let data = "x".repeat(11 * 1024 * 1024);

    let result = client.call_tool(
        "validate",
        json!({ "content": data, "content_type": "spec" }),
    );
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("10MB"), "error should mention 10MB limit");
}

// @minter:e2e validate-reject-unreadable-file
#[test]
#[cfg(unix)]
/// mcp-server: validate-reject-unreadable-file
fn validate_reject_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "secret", VALID_SPEC);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

    let result = client.call_tool("validate", json!({ "path": path.to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result).to_lowercase();
    assert!(
        text.contains("permission"),
        "error should mention permission"
    );
    assert!(text.contains("secret"), "error should mention filename");

    // Restore permissions for cleanup
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
}

// @minter:e2e validate-content-takes-precedence
#[test]
/// mcp-server: validate-content-takes-precedence
fn validate_content_takes_precedence() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = write_spec(&dir, "a", VALID_SPEC);

    let inline_content = "\
spec inline-override v1.0.0
title \"Inline Override\"

description
  Inline content should win.

motivation
  Precedence testing.

behavior do-override [happy_path]
  \"Override behavior\"

  given
    System ready

  when act

  then emits stdout
    assert output contains \"override\"
";

    let result = client.call_tool(
        "validate",
        json!({
            "path": path.to_str().unwrap(),
            "content": inline_content,
            "content_type": "spec"
        }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(
        data["results"][0]["name"], "inline-override",
        "inline content should take precedence over path"
    );
}

// @minter:e2e validate-reject-unknown-content-type
#[test]
/// mcp-server: validate-reject-unknown-content-type
fn validate_reject_unknown_content_type() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "validate",
        json!({ "content": "some content", "content_type": "banana" }),
    );
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains("banana"),
        "error should mention the bad content_type"
    );
}

// @minter:e2e validate-require-path-or-content
#[test]
/// mcp-server: validate-require-path-or-content
fn validate_require_path_or_content() {
    let mut client = McpClient::new();
    let result = client.call_tool("validate", json!({}));
    assert!(is_error(&result));
    let text = text_content(&result).to_lowercase();
    assert!(text.contains("path"), "error should mention path");
    assert!(text.contains("content"), "error should mention content");
}

// ── Inspect security tests ─────────────────────────────

// @minter:e2e inspect-reject-non-spec-extension
#[test]
/// mcp-server: inspect-reject-non-spec-extension
fn inspect_reject_non_spec_extension() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    std::fs::write(&path, "key: value").unwrap();

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(
        text.contains(".spec") || text.contains(".nfr"),
        "error should mention valid extensions"
    );
}

// @minter:e2e inspect-reject-oversized-file
#[test]
/// mcp-server: inspect-reject-oversized-file
fn inspect_reject_oversized_file() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.spec");
    let data = "x".repeat(11 * 1024 * 1024);
    std::fs::write(&path, data).unwrap();

    let result = client.call_tool("inspect", json!({ "path": path.to_str().unwrap() }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("10MB"), "error should mention 10MB limit");
}

// ── Scaffold edge case tests ───────────────────────────

// @minter:e2e scaffold-nfr-missing-category
#[test]
/// mcp-server: scaffold-nfr-missing-category
fn scaffold_nfr_missing_category() {
    let mut client = McpClient::new();
    let result = client.call_tool("scaffold", json!({ "type": "nfr" }));
    assert!(is_error(&result));
    let text = text_content(&result).to_lowercase();
    assert!(text.contains("category"), "error should mention category");
}

// @minter:e2e scaffold-unknown-type
#[test]
/// mcp-server: scaffold-unknown-type
fn scaffold_unknown_type() {
    let mut client = McpClient::new();
    let result = client.call_tool("scaffold", json!({ "type": "banana" }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains("banana"), "error should mention the bad type");
}

// ── Graph edge case tests ──────────────────────────────

// @minter:e2e graph-impacted-transitive
#[test]
/// mcp-server: graph-impacted-transitive
fn graph_impacted_transitive() {
    let mut client = McpClient::new();
    let dir = TempDir::new().unwrap();
    // Chain: a → b → c. Impacted by c should include both a and b.
    write_spec(&dir, "a-spec", SPEC_A_DEPENDS_B);
    write_spec(&dir, "b-spec", SPEC_B_DEPENDS_C);
    write_spec(&dir, "c-spec", SPEC_C_NO_DEPS);

    let result = client.call_tool(
        "graph",
        json!({ "path": dir.path().to_str().unwrap(), "impacted": "c-spec" }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    let impacted: Vec<&str> = data["impacted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(
        impacted.contains(&"a-spec"),
        "a-spec should be transitively impacted"
    );
    assert!(
        impacted.contains(&"b-spec"),
        "b-spec should be directly impacted"
    );
}

// @minter:e2e graph-nonexistent-directory
#[test]
/// mcp-server: graph-nonexistent-directory
fn graph_nonexistent_directory() {
    let mut client = McpClient::new();
    let fake_path = "/tmp/minter-nonexistent-test-dir-12345";
    let result = client.call_tool("graph", json!({ "path": fake_path }));
    assert!(is_error(&result));
    let text = text_content(&result);
    assert!(text.contains(fake_path), "error should mention the path");
}

// ═══════════════════════════════════════════════════════════════
// Coverage tool
// ═══════════════════════════════════════════════════════════════

// @minter:e2e coverage-full
#[test]
/// mcp-server: coverage returns JSON report for fully covered specs
fn mcp_coverage_full() {
    let dir = TempDir::new().unwrap();
    write_spec(
        &dir,
        "auth",
        "spec auth v1.0.0\ntitle \"Auth\"\n\ndescription\n  Auth spec.\n\nmotivation\n  Testing.\n\nbehavior login [happy_path]\n  \"login\"\n\n  given\n    The system is ready\n\n  when login\n\n  then emits stdout\n    assert output contains \"ok\"\n",
    );
    // Create a test file with a coverage tag
    let test_path = dir.path().join("test.rs");
    std::fs::write(
        &test_path,
        "// @minter:e2e login\n#[test]\nfn test_login() {}\n",
    )
    .unwrap();

    let mut client = McpClient::new();
    let result = client.call_tool(
        "coverage",
        json!({
            "spec_path": dir.path().to_str().unwrap(),
        }),
    );
    assert!(
        !is_error(&result),
        "coverage should succeed, got: {}",
        text_content(&result)
    );
    let data = json_content(&result);
    assert_eq!(data["total_behaviors"], 1);
    assert_eq!(data["covered_behaviors"], 1);
    assert_eq!(data["coverage_percentage"], 100);
}

// @minter:e2e coverage-uncovered
#[test]
/// mcp-server: coverage reports uncovered behaviors
fn mcp_coverage_uncovered() {
    let dir = TempDir::new().unwrap();
    write_spec(
        &dir,
        "auth",
        "spec auth v1.0.0\ntitle \"Auth\"\n\ndescription\n  Auth spec.\n\nmotivation\n  Testing.\n\nbehavior login [happy_path]\n  \"login\"\n\n  given\n    The system is ready\n\n  when login\n\n  then emits stdout\n    assert output contains \"ok\"\n",
    );

    let mut client = McpClient::new();
    let result = client.call_tool(
        "coverage",
        json!({
            "spec_path": dir.path().to_str().unwrap(),
        }),
    );
    assert!(!is_error(&result), "coverage should succeed");
    let data = json_content(&result);
    assert_eq!(data["total_behaviors"], 1);
    assert_eq!(data["covered_behaviors"], 0);
}

// @minter:e2e coverage-nonexistent-path
#[test]
/// mcp-server: coverage errors on nonexistent path
fn mcp_coverage_nonexistent_path() {
    let mut client = McpClient::new();
    let result = client.call_tool(
        "coverage",
        json!({ "spec_path": "/tmp/minter-nonexistent-coverage-12345" }),
    );
    assert!(is_error(&result));
}

// @minter:e2e coverage-with-scan-dirs
#[test]
/// mcp-server: coverage respects scan directories
fn mcp_coverage_with_scan_dirs() {
    let dir = TempDir::new().unwrap();
    let specs_dir = dir.path().join("specs");
    let tests_dir = dir.path().join("tests");
    std::fs::create_dir_all(&specs_dir).unwrap();
    std::fs::create_dir_all(&tests_dir).unwrap();

    std::fs::write(
        specs_dir.join("auth.spec"),
        "spec auth v1.0.0\ntitle \"Auth\"\n\ndescription\n  Auth spec.\n\nmotivation\n  Testing.\n\nbehavior login [happy_path]\n  \"login\"\n\n  given\n    The system is ready\n\n  when login\n\n  then emits stdout\n    assert output contains \"ok\"\n",
    ).unwrap();
    std::fs::write(
        tests_dir.join("test.rs"),
        "// @minter:e2e login\n#[test]\nfn test_login() {}\n",
    )
    .unwrap();

    let mut client = McpClient::new();
    let result = client.call_tool(
        "coverage",
        json!({
            "spec_path": specs_dir.to_str().unwrap(),
            "scan": [tests_dir.to_str().unwrap()],
        }),
    );
    assert!(!is_error(&result));
    let data = json_content(&result);
    assert_eq!(data["covered_behaviors"], 1);
}
