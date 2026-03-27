export interface BehaviorInfo {
  name: string
  description: string
  covered: boolean
  test_types: string[]
  category: string
  nfr_refs: string[]
}

export interface SpecInfo {
  name: string
  version: string
  path: string
  behavior_count: number
  behaviors: BehaviorInfo[]
  validation_status: "Valid" | { Invalid: string[] } | "Unknown"
  nfr_refs: string[]
  dependencies: string[]
  title?: string
  description?: string
  motivation?: string
}

export interface Integrity {
  specs: "Aligned" | "Drifted" | "NoLock"
  nfrs: "Aligned" | "Drifted" | "NoLock"
  tests: "Aligned" | "Drifted" | "NoLock"
  lock_status: "Aligned" | "Drifted" | "NoLock"
}

export interface Drift {
  modified_specs: string[]
  unlocked_specs: string[]
  modified_nfrs: string[]
  unlocked_nfrs: string[]
  modified_tests: string[]
  missing_tests: string[]
}

export interface InvalidTag {
  file: string
  line: number
  message: string
}

export interface ProjectState {
  specs: SpecInfo[]
  nfr_count: number
  test_count: number
  coverage_covered: number
  coverage_total: number
  integrity: Integrity
  drift: Drift
  invalid_tags: InvalidTag[]
  dep_errors: string[]
  errors: string[]
}
