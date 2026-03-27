import type { ProjectState, SpecInfo } from "@/types"

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
    ...overrides,
  }
}

export function mockState(overrides?: Partial<ProjectState>): ProjectState {
  return {
    specs: [mockSpec()],
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
