import type { ProjectState, SpecInfo, NfrInfo } from "@/types"

export function mockSpec(overrides?: Partial<SpecInfo>): SpecInfo {
  return {
    name: "test-spec",
    version: "1.0.0",
    path: "/specs/test-spec.spec",
    behavior_count: 3,
    behaviors: [
      { name: "login", description: "Authenticate a user with credentials", covered: true, test_types: ["unit", "e2e"], category: "happy_path", nfr_refs: ["performance#api-latency"] },
      { name: "logout", description: "End the user session", covered: true, test_types: ["e2e"], category: "happy_path", nfr_refs: [] },
      { name: "refresh", description: "Refresh an expired token", covered: false, test_types: [], category: "error_case", nfr_refs: [] },
    ],
    validation_status: "Valid",
    nfr_refs: ["performance#api-latency", "reliability#no-data-loss"],
    dependencies: ["user-command >= 1.0.0"],
    dep_errors: [],
    ...overrides,
  }
}

export function mockNfr(overrides?: Partial<NfrInfo>): NfrInfo {
  return {
    category: "performance",
    version: "1.1.0",
    title: "Performance Requirements",
    description: "Performance constraints for the system",
    path: "/nfrs/performance.nfr",
    constraint_count: 3,
    constraints: [
      {
        name: "api-latency",
        description: "API response time limit",
        constraint_type: "metric",
        threshold: "< 500ms",
        rule_text: null,
        violation: "critical",
        overridable: true,
      },
      {
        name: "throughput",
        description: "Minimum throughput requirement",
        constraint_type: "metric",
        threshold: ">= 1000rps",
        rule_text: null,
        violation: "warning",
        overridable: false,
      },
      {
        name: "no-blocking-calls",
        description: "No synchronous blocking in request path",
        constraint_type: "rule",
        threshold: null,
        rule_text: "All I/O operations must be async",
        violation: "critical",
        overridable: false,
      },
    ],
    validation_status: "Valid",
    referenced_by: ["auth-command", "validate-command"],
    ...overrides,
  }
}

export function mockState(overrides?: Partial<ProjectState>): ProjectState {
  return {
    specs: [mockSpec()],
    nfrs: [mockNfr()],
    nfr_count: 22,
    test_count: 145,
    coverage_covered: 2,
    coverage_total: 3,
    integrity: { specs: "Aligned", nfrs: "Aligned", tests: "Aligned", lock_status: "Aligned" },
    drift: { modified_specs: [], unlocked_specs: [], modified_nfrs: [], unlocked_nfrs: [], modified_tests: [], missing_tests: [] },
    invalid_tags: [],
    dep_errors: [],
    errors: [],
    ...overrides,
  }
}
