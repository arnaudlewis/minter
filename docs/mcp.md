# MCP Server

`minter-mcp` is a spec authoring assistant exposed over the Model Context Protocol. It is not a CLI wrapper — it is a purpose-built agent tool that understands the DSL, the methodology, and the project graph.

---

## Setup

### Claude Code

```bash
claude mcp add minter minter-mcp
```

### Claude Desktop or Cursor

Add to your MCP config file:

```json
{
  "mcpServers": {
    "minter": {
      "command": "minter-mcp"
    }
  }
}
```

### Verify

Ask your agent: "Call initialize_minter and tell me the methodology." The agent should describe the five-phase workflow.

---

## 11 tools

### initialize_minter

Call this first before using any other tool. Returns the spec-driven methodology, five workflow phases, and a summary of all available tools. Agents that call this first write better specs — the methodology is embedded in the response.

### guide

Return a condensed reference guide for a specific topic.

| Topic | Content |
|-------|---------|
| `methodology` | Full methodology and principles |
| `workflow` | Five-phase workflow summary |
| `authoring` | Granularity, decomposition, entity format |
| `smells` | 9 smell types, Observer Test, Swap Test |
| `nfr` | Categories, constraint types, FR/NFR decision tree |
| `context` | Context management protocol for lazy loading |
| `coverage` | Coverage tagging guide |
| `config` | Configuration reference |
| `lock` | Lock file reference |
| `ci` | CI verification reference |
| `web` | Dashboard reference |

### validate

Validate a spec or NFR file. Accepts a file path or inline content.

Parameters:
- `path` — file or directory path
- `content` — inline spec content (takes precedence over `path` if both provided)
- `content_type` — `"spec"` or `"nfr"` (required with `content`)
- `deep` — enable dependency resolution and NFR cross-validation (boolean)

Returns: structured validation results with pass/fail status and error details with fix suggestions.

Security limits: files over 10MB are rejected; inline content over 10MB is rejected; only `.spec` and `.nfr` extensions are accepted.

### inspect

Return metadata for a spec or NFR file: name, version, behavior/constraint counts, category distribution, dependencies, and assertion types.

Parameters: `path` or `content` + `content_type`

### scaffold

Generate a skeleton file as text content.

Parameters:
- `type` — `"spec"` or `"nfr"`
- `category` — required for `"nfr"` (one of the 7 valid categories)

### format

Return the DSL grammar reference.

Parameters:
- `type` — `"spec"` or `"nfr"`

### graph

Query the dependency graph.

Parameters:
- `path` — directory path
- `impacted` — spec name for reverse dependency analysis (optional)

Returns: structured graph data with all specs, edges, and NFR references.

### list_specs

Return all specs in the project with metadata: name, version, behavior count, validation status, NFR references, and dependencies. Sorted alphabetically.

No parameters — uses project config to discover specs.

### list_nfrs

Return all NFR categories with their constraints: category, version, constraint count, constraint names, types, thresholds, and rule text.

No parameters — uses project config to discover NFRs.

### search

Search across the spec graph by keyword.

Parameters:
- `query` — search term

Returns matches from spec names, behavior names, and NFR constraint names with context about which spec or file each result belongs to.

### assess

Assess a spec for quality issues.

Parameters: `path` or `content` + `content_type`

Returns a structured assessment with:
- `coverage_balance` — count of happy_path, error_case, edge_case behaviors; warns if error cases are missing
- `smells` — list of detected smells (implementation leakage, compound behaviors, NASA forbidden words, etc.) with behavior name, smell type, and fix suggestion
- `missing` — suggested error_case behaviors for uncovered happy paths
- `nfr_gaps` — suggested NFR categories based on what the spec does

A clean spec returns empty arrays for all three fields.

---

## Agent workflow

### Starting a new spec

1. Call `initialize_minter` — agent learns the methodology
2. Call `scaffold` with `type: "spec"` — get the skeleton
3. Fill in behaviors from the feature description
4. Call `validate` — check for errors
5. Call `assess` — check for quality issues
6. Iterate until `validate` passes and `assess` returns clean

### Exploring the project

```
list_specs    → overview of all specs and their status
list_nfrs     → all NFR categories and constraints
graph         → full dependency tree
search        → find specs or behaviors by keyword
```

Use the lazy loading protocol from `guide context` — get the subgraph structure first, then load content for what you need:

1. Call `graph` scoped to your target spec — get the dependency tree
2. Call `inspect` on the root spec — get metadata without full content
3. Call `validate` with `content` inline as you author
4. Load NFRs on demand as you encounter references

Never front-load the full project. Scope to the subgraph you are working in.

### Assessing existing specs

For each spec to review:
1. Call `validate` — check current validity
2. Call `assess` — check quality issues
3. Report smells and suggested improvements

The `assess` tool is the primary quality gate for agent-authored specs. Run it after every significant edit before presenting the spec for review.

---

## Inline content mode

Both `validate` and `inspect` accept inline content, so agents can validate specs as they author them — before writing to disk. When both `path` and `content` are provided, `content` takes precedence.

```json
{
  "tool": "validate",
  "content": "spec my-feature v0.1.0\ntitle \"My Feature\"\n...",
  "content_type": "spec"
}
```

---

## Transport

The MCP server uses stdio transport. It is a separate binary (`minter-mcp`) installed alongside `minter`. Both are included in the Homebrew formula and the release archives.

---

See also:
- [getting-started.md](getting-started.md) — setup walkthrough
- [methodology.md](methodology.md) — the five phases the agent follows
- [authoring-guide.md](authoring-guide.md) — what the agent knows about good specs
