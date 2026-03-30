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
  dep_errors: string[]
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

export interface NfrConstraintInfo {
  name: string
  description: string
  constraint_type: string  // "metric" or "rule"
  threshold: string | null
  rule_text: string | null
  violation: string
  overridable: boolean
}

export interface NfrInfo {
  category: string
  version: string
  title: string
  description: string
  path: string
  constraint_count: number
  constraints: NfrConstraintInfo[]
  validation_status: "Valid" | { Invalid: string[] } | "Unknown"
  referenced_by: string[]  // spec names
}

export interface ProjectState {
  specs: SpecInfo[]
  nfrs: NfrInfo[]
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

// --- Design system types ---

export interface DesignSystem {
  archetype: ArchetypeResult
  palette: PaletteDefinition
  dark_mode: DarkModeColors
  type_scale: TypeScaleConfig
  spacing: SpacingConfig
  component_styles: ComponentStyleConfig
  layout: LayoutConfig
  spec_metadata: SpecMetadata | null
  decisions: DesignDecision[]
}

export interface ArchetypeResult {
  name: string
  confidence: number
  reasoning: string
}

export interface PaletteDefinition {
  name: string
  description: string
  vibe: string[]
  reference: string
  primary: string
  secondary: string
  accent: string
  neutral: string
  semantic: { success: string; warning: string; error: string; info: string }
  shades: string[]
}

export interface DarkModeColors {
  background: string
  foreground: string
  primary: string
  shades: string[]
  semantic: { success: string; warning: string; error: string; info: string }
}

export interface TypeScaleConfig {
  ratio: number
  level_count: number
  body_size: number
  sizes: number[]
  line_heights: number[]
  font_weights: number[]
  font_family: string
}

export interface SpacingConfig {
  base: number
  ratio: number
  step_count: number
  steps: number[]
}

export interface ComponentStyleConfig {
  button_border_radius: number
  card_border_radius: number
}

export interface LayoutConfig {
  sidebar?: { position: string; width: string; nav_items: string[] }
  header?: { title: string; has_search: boolean }
  content: { sections: ContentSection[] }
}

export interface ContentSection {
  kind: string
  title: string
  width: string
}

export interface SpecMetadata {
  project_name: string
  domains: { name: string; spec_count: number; spec_names: string[] }[]
  spec_metrics: { name: string; version: string; behavior_count: number }[]
  total_spec_count: number
  total_behavior_count: number
  total_entity_count: number
}

export interface DesignDecision {
  property: string
  value: string
  previous_value?: string
}

export type WsMessage =
  | { type: "state-update"; data: ProjectState }
  | { type: "design-update"; data: DesignSystem }
