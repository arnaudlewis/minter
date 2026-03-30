import type {
  DesignSystem,
  TypeScaleConfig,
  ContentSection,
} from "@/types"
import {
  BarChart3,
  CheckCircle2,
  FileText,
  Hash,
  LayoutDashboard,
  Search,
  Settings,
  Eye,
  Paintbrush,
  GitBranch,
  Shield,
  ArrowUpRight,
  ArrowDownRight,
  Minus,
  BookOpen,
  Microscope,
  Lock,
  Hammer,
  Terminal,
} from "lucide-react"

// Map nav item names to icons
const navIcons: Record<string, React.ReactNode> = {
  Overview: <LayoutDashboard size={16} />,
  Validate: <CheckCircle2 size={16} />,
  Watch: <Eye size={16} />,
  Format: <Paintbrush size={16} />,
  Graph: <GitBranch size={16} />,
  Coverage: <Shield size={16} />,
  Settings: <Settings size={16} />,
  Guide: <BookOpen size={16} />,
  Inspect: <Microscope size={16} />,
  Lock: <Lock size={16} />,
  Scaffold: <Hammer size={16} />,
  Ci: <Terminal size={16} />,
}

/**
 * Normalize the type scale so sizes are always ascending (smallest first).
 * The server may return sizes in descending order (largest first).
 * We sort sizes ascending and reorder line_heights and font_weights to match.
 */
function normalizeTypeScale(ts: TypeScaleConfig): TypeScaleConfig {
  const indexed = ts.sizes.map((size, i) => ({
    size,
    lh: ts.line_heights[i] ?? 1.4,
    fw: ts.font_weights[i] ?? 400,
  }))
  indexed.sort((a, b) => a.size - b.size)
  return {
    ...ts,
    sizes: indexed.map((e) => e.size),
    line_heights: indexed.map((e) => e.lh),
    font_weights: indexed.map((e) => e.fw),
  }
}

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

// --- Sidebar ---

function Sidebar({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, layout, type_scale } = design
  const sidebar = layout.sidebar
  if (!sidebar) return null

  const sidebarBg = palette.shades[9]
  const sidebarText = palette.shades[0]
  const sidebarTextMuted = palette.shades[3]
  const activeItem = sidebar.nav_items[0]

  return (
    <aside
      data-testid="design-sidebar"
      style={{
        width: sidebar.width,
        minWidth: sidebar.width,
        backgroundColor: sidebarBg,
        color: sidebarText,
        fontFamily: type_scale.font_family,
        display: "flex",
        flexDirection: "column",
        height: "100%",
        overflow: "hidden",
      }}
    >
      {/* Project name */}
      <div
        style={{
          padding: `14px 16px`,
          borderBottom: `1px solid ${hexToRgba(sidebarText, 0.08)}`,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <div
            style={{
              width: 24,
              height: 24,
              borderRadius: 6,
              background: palette.primary,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontWeight: 600,
              fontSize: "12px",
              color: "#fff",
              flexShrink: 0,
            }}
          >
            {(design.spec_metadata?.project_name ?? "P").charAt(0).toUpperCase()}
          </div>
          <span
            style={{
              fontWeight: 600,
              fontSize: "14px",
              letterSpacing: "-0.02em",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {design.spec_metadata?.project_name ?? "Project"}
          </span>
        </div>
      </div>

      {/* Navigation */}
      <nav
        style={{
          flex: 1,
          padding: `8px 8px`,
          overflowY: "auto",
        }}
      >
        <div
          style={{
            fontSize: "11px",
            fontWeight: 500,
            color: sidebarTextMuted,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            padding: `6px 12px`,
            marginBottom: "2px",
          }}
        >
          Navigation
        </div>
        {sidebar.nav_items.map((item) => {
          const isActive = item === activeItem
          return (
            <div
              key={item}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                padding: `6px 12px`,
                borderRadius: 6,
                marginBottom: 2,
                backgroundColor: isActive
                  ? hexToRgba(palette.primary, 0.15)
                  : "transparent",
                color: isActive ? palette.shades[1] : sidebarTextMuted,
                fontWeight: isActive ? 500 : 400,
                fontSize: "14px",
                cursor: "pointer",
                transition: "background-color 0.15s, color 0.15s",
              }}
            >
              <span style={{ opacity: isActive ? 1 : 0.6, display: "flex", alignItems: "center" }}>
                {navIcons[item] ?? <FileText size={16} />}
              </span>
              {item}
              {isActive && (
                <div
                  style={{
                    marginLeft: "auto",
                    width: 4,
                    height: 16,
                    borderRadius: 2,
                    backgroundColor: palette.primary,
                  }}
                />
              )}
            </div>
          )
        })}

        {/* Divider */}
        <div
          style={{
            height: 1,
            background: hexToRgba(sidebarText, 0.08),
            margin: `12px 12px`,
          }}
        />

        {/* Domain summary */}
        <div
          style={{
            fontSize: "11px",
            fontWeight: 500,
            color: sidebarTextMuted,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            padding: `6px 12px`,
            marginBottom: "2px",
          }}
        >
          Domains
        </div>
        {(design.spec_metadata?.domains ?? []).map((domain) => (
          <div
            key={domain.name}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: `4px 12px`,
              fontSize: "13px",
              color: sidebarTextMuted,
            }}
          >
            <span>{domain.name}</span>
            <span
              style={{
                fontSize: "11px",
                backgroundColor: hexToRgba(sidebarText, 0.08),
                borderRadius: 10,
                padding: "1px 7px",
                fontWeight: 500,
              }}
            >
              {domain.spec_count}
            </span>
          </div>
        ))}
      </nav>

      {/* Bottom section */}
      <div
        style={{
          padding: "12px 16px",
          borderTop: `1px solid ${hexToRgba(sidebarText, 0.08)}`,
          fontSize: "11px",
          color: sidebarTextMuted,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
          <div
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              backgroundColor: palette.semantic.success,
            }}
          />
          All systems operational
        </div>
      </div>
    </aside>
  )
}

// --- Header ---

function Header({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, layout, type_scale, component_styles } = design
  const header = layout.header

  return (
    <header
      data-testid="design-header"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: `0 20px`,
        backgroundColor: "#ffffff",
        borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.12)}`,
        fontFamily: type_scale.font_family,
        height: 48,
        minHeight: 48,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <span
          style={{
            fontSize: "14px",
            fontWeight: 600,
            color: palette.shades[9] ?? palette.neutral,
            letterSpacing: "-0.02em",
          }}
        >
          {header?.title ?? "Dashboard"}
        </span>
        <span
          style={{
            color: hexToRgba(palette.neutral, 0.4),
            fontSize: "14px",
          }}
        >
          /
        </span>
        <span
          style={{
            fontSize: "14px",
            color: palette.neutral,
          }}
        >
          Overview
        </span>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        {header?.has_search && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "6px",
              padding: `6px 10px`,
              borderRadius: component_styles.button_border_radius + "px",
              border: `1px solid ${hexToRgba(palette.neutral, 0.15)}`,
              backgroundColor: hexToRgba(palette.neutral, 0.03),
              color: hexToRgba(palette.neutral, 0.5),
              fontSize: "13px",
              minWidth: 200,
            }}
          >
            <Search size={14} />
            <span>Search specs...</span>
            <span
              style={{
                marginLeft: "auto",
                fontSize: "11px",
                backgroundColor: hexToRgba(palette.neutral, 0.08),
                borderRadius: 4,
                padding: "1px 6px",
                fontFamily: "monospace",
              }}
            >
              /
            </span>
          </div>
        )}
        <div
          style={{
            width: 28,
            height: 28,
            borderRadius: "50%",
            background: `linear-gradient(135deg, ${palette.primary}, ${palette.accent})`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#fff",
            fontWeight: 600,
            fontSize: "11px",
          }}
        >
          AL
        </div>
      </div>
    </header>
  )
}

// --- Metric Cards ---

interface MetricCardData {
  label: string
  value: string | number
  icon: React.ReactNode
  trend: "up" | "down" | "neutral"
  trendValue: string
}

function MetricCards({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, type_scale, component_styles, spec_metadata } = design

  const totalBehaviors = spec_metadata?.total_behavior_count ?? 0
  const coveragePct = totalBehaviors > 0
    ? Math.round((totalBehaviors * 0.85) / totalBehaviors * 100)
    : 0

  const metrics: MetricCardData[] = [
    {
      label: "Specifications",
      value: spec_metadata?.total_spec_count ?? 0,
      icon: <FileText size={18} style={{ color: palette.primary }} />,
      trend: "up",
      trendValue: "+3 this week",
    },
    {
      label: "Behaviors",
      value: spec_metadata?.total_behavior_count ?? 0,
      icon: <Hash size={18} style={{ color: palette.accent }} />,
      trend: "up",
      trendValue: "+18 this week",
    },
    {
      label: "Entities",
      value: spec_metadata?.total_entity_count ?? 0,
      icon: <BarChart3 size={18} style={{ color: palette.semantic.info }} />,
      trend: "neutral",
      trendValue: "No change",
    },
    {
      label: "Coverage",
      value: `${coveragePct}%`,
      icon: <Shield size={18} style={{ color: palette.semantic.success }} />,
      trend: "up",
      trendValue: "+2% this week",
    },
  ]

  return (
    <div
      data-testid="design-metric-cards"
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, 1fr)",
        gap: "12px",
      }}
    >
      {metrics.map((metric) => (
        <div
          key={metric.label}
          style={{
            backgroundColor: "#ffffff",
            border: `1px solid ${hexToRgba(palette.neutral, 0.10)}`,
            borderRadius: component_styles.card_border_radius + "px",
            padding: "12px 14px",
            fontFamily: type_scale.font_family,
            transition: "box-shadow 0.2s, border-color 0.2s",
            boxShadow: `0 1px 2px ${hexToRgba(palette.neutral, 0.04)}`,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              marginBottom: "8px",
            }}
          >
            <span
              style={{
                fontSize: "11px",
                fontWeight: 500,
                color: palette.neutral,
                textTransform: "uppercase",
                letterSpacing: "0.04em",
              }}
            >
              {metric.label}
            </span>
            <div
              style={{
                width: 28,
                height: 28,
                borderRadius: component_styles.button_border_radius + "px",
                backgroundColor: hexToRgba(palette.primary, 0.06),
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              {metric.icon}
            </div>
          </div>
          <div
            style={{
              fontSize: "24px",
              fontWeight: 600,
              color: palette.shades[9] ?? palette.neutral,
              lineHeight: 1.2,
              letterSpacing: "-0.02em",
              fontVariantNumeric: "tabular-nums",
            }}
          >
            {metric.value}
          </div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "4px",
              marginTop: "4px",
              fontSize: "11px",
              color:
                metric.trend === "up"
                  ? palette.semantic.success
                  : metric.trend === "down"
                    ? palette.semantic.error
                    : palette.neutral,
            }}
          >
            {metric.trend === "up" ? (
              <ArrowUpRight size={12} />
            ) : metric.trend === "down" ? (
              <ArrowDownRight size={12} />
            ) : (
              <Minus size={12} />
            )}
            <span>{metric.trendValue}</span>
          </div>
        </div>
      ))}
    </div>
  )
}

// --- Data Table ---

function DataTable({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, type_scale, component_styles } = design
  const spec_metadata = design.spec_metadata

  // Assign statuses for visual variety
  const statusMap: Record<string, "pass" | "warning" | "fail"> = {
    "validate-command": "pass",
    cli: "pass",
    "spec-grammar": "pass",
    "nfr-grammar": "pass",
    "watch-command": "pass",
    "format-command": "pass",
    "scaffold-command": "warning",
    "inspect-command": "pass",
    "graph-command": "pass",
    "mcp-server": "pass",
    "dependency-resolution": "fail",
    "graph-cache": "pass",
    "design-system-generator": "warning",
  }

  const statusColors: Record<string, { bg: string; text: string; label: string }> = {
    pass: {
      bg: hexToRgba(palette.semantic.success, 0.08),
      text: palette.semantic.success,
      label: "Valid",
    },
    warning: {
      bg: hexToRgba(palette.semantic.warning, 0.08),
      text: palette.semantic.warning,
      label: "Warnings",
    },
    fail: {
      bg: hexToRgba(palette.semantic.error, 0.08),
      text: palette.semantic.error,
      label: "Errors",
    },
  }

  return (
    <div
      data-testid="design-data-table"
      style={{
        backgroundColor: "#ffffff",
        border: `1px solid ${hexToRgba(palette.neutral, 0.10)}`,
        borderRadius: component_styles.card_border_radius + "px",
        overflow: "hidden",
        fontFamily: type_scale.font_family,
        boxShadow: `0 1px 2px ${hexToRgba(palette.neutral, 0.04)}`,
      }}
    >
      {/* Table */}
      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          tableLayout: "fixed",
        }}
      >
        <colgroup>
          <col style={{ width: "40%" }} />
          <col style={{ width: "15%" }} />
          <col style={{ width: "15%" }} />
          <col style={{ width: "30%" }} />
        </colgroup>
        <thead>
          <tr
            style={{
              borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.08)}`,
              backgroundColor: hexToRgba(palette.neutral, 0.02),
            }}
          >
            {["Name", "Version", "Behaviors", "Status"].map((col) => (
              <th
                key={col}
                style={{
                  fontSize: "11px",
                  fontWeight: 500,
                  color: palette.neutral,
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  padding: "8px 16px",
                  textAlign: "left",
                }}
              >
                {col}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {(spec_metadata?.spec_metrics ?? []).map((spec, i) => {
            const status = statusMap[spec.name] ?? "pass"
            const color = statusColors[status]
            const isEven = i % 2 === 0

            return (
              <tr
                key={spec.name}
                style={{
                  borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.05)}`,
                  backgroundColor: isEven
                    ? "transparent"
                    : hexToRgba(palette.neutral, 0.015),
                  cursor: "pointer",
                  transition: "background-color 0.15s",
                }}
              >
                <td
                  style={{
                    fontSize: "13px",
                    fontWeight: 500,
                    color: palette.shades[8] ?? palette.neutral,
                    fontFamily: "monospace",
                    padding: "8px 16px",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {spec.name}
                </td>
                <td
                  style={{
                    fontSize: "12px",
                    color: palette.neutral,
                    fontFamily: "monospace",
                    padding: "8px 16px",
                  }}
                >
                  v{spec.version}
                </td>
                <td
                  style={{
                    fontSize: "13px",
                    color: palette.shades[7] ?? palette.neutral,
                    fontVariantNumeric: "tabular-nums",
                    padding: "8px 16px",
                  }}
                >
                  {spec.behavior_count}
                </td>
                <td style={{ padding: "8px 16px" }}>
                  <span
                    style={{
                      display: "inline-flex",
                      alignItems: "center",
                      gap: "4px",
                      fontSize: "11px",
                      fontWeight: 500,
                      color: color.text,
                      backgroundColor: color.bg,
                      borderRadius: 12,
                      padding: "2px 10px",
                    }}
                  >
                    <span
                      style={{
                        width: 5,
                        height: 5,
                        borderRadius: "50%",
                        backgroundColor: color.text,
                      }}
                    />
                    {color.label}
                  </span>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}

// --- Status Panel ---

function StatusPanel({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, type_scale, component_styles } = design

  const total = design.spec_metadata?.total_spec_count ?? 0
  const valid = Math.round(total * 0.82)
  const warnings = Math.round(total * 0.11)
  const errors = total - valid - warnings

  const statuses = [
    { label: "Valid", count: valid, color: palette.semantic.success },
    { label: "Warnings", count: warnings, color: palette.semantic.warning },
    { label: "Errors", count: errors, color: palette.semantic.error },
  ]

  return (
    <div
      data-testid="design-status-panel"
      style={{
        backgroundColor: "#ffffff",
        border: `1px solid ${hexToRgba(palette.neutral, 0.10)}`,
        borderRadius: component_styles.card_border_radius + "px",
        padding: "16px",
        fontFamily: type_scale.font_family,
        boxShadow: `0 1px 2px ${hexToRgba(palette.neutral, 0.04)}`,
      }}
    >
      {/* Stacked bar */}
      <div
        style={{
          display: "flex",
          height: 8,
          borderRadius: 4,
          overflow: "hidden",
          marginBottom: "16px",
        }}
      >
        {statuses.map((s) => (
          <div
            key={s.label}
            style={{
              flex: s.count,
              backgroundColor: s.color,
              transition: "flex 0.3s",
            }}
          />
        ))}
      </div>

      {/* Status items */}
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {statuses.map((s) => (
          <div
            key={s.label}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  backgroundColor: s.color,
                }}
              />
              <span
                style={{
                  fontSize: "13px",
                  color: palette.shades[7] ?? palette.neutral,
                }}
              >
                {s.label}
              </span>
            </div>
            <span
              style={{
                fontSize: "16px",
                fontWeight: 600,
                color: palette.shades[9] ?? palette.neutral,
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {s.count}
            </span>
          </div>
        ))}
      </div>

      {/* Divider */}
      <div
        style={{
          height: 1,
          backgroundColor: hexToRgba(palette.neutral, 0.08),
          margin: `12px 0`,
        }}
      />

      {/* Summary */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          fontSize: "13px",
        }}
      >
        <span style={{ color: palette.neutral }}>Total specifications</span>
        <span
          style={{
            fontWeight: 600,
            color: palette.shades[9] ?? palette.neutral,
          }}
        >
          {total}
        </span>
      </div>
    </div>
  )
}

// --- Content Section Router ---

function ContentSectionRenderer({
  section,
  design,
}: {
  section: ContentSection
  design: DesignSystem
}) {
  const { type_scale, palette } = design

  const sectionTitle = (
    <h2
      style={{
        fontSize: "14px",
        fontWeight: 600,
        color: palette.shades[9] ?? palette.neutral,
        letterSpacing: "-0.01em",
        marginBottom: "12px",
        fontFamily: type_scale.font_family,
      }}
    >
      {section.title}
    </h2>
  )

  switch (section.kind) {
    case "metric-cards":
      return (
        <section>
          {sectionTitle}
          <MetricCards design={design} />
        </section>
      )
    case "data-table":
      return (
        <section>
          {sectionTitle}
          <DataTable design={design} />
        </section>
      )
    case "status-panel":
      return (
        <section>
          {sectionTitle}
          <StatusPanel design={design} />
        </section>
      )
    default:
      return null
  }
}

// --- Main Dashboard ---

interface DashboardPrototypeProps {
  design: DesignSystem
}

export function DashboardPrototype({ design: rawDesign }: DashboardPrototypeProps) {
  const design: DesignSystem = { ...rawDesign, type_scale: normalizeTypeScale(rawDesign.type_scale) }
  const { type_scale } = design

  // Determine widths for sections
  const getGridColumn = (width: string): string => {
    switch (width) {
      case "full":
        return "1 / -1"
      case "half":
        return "span 1"
      case "third":
        return "span 1"
      default:
        return "1 / -1"
    }
  }

  return (
    <div
      data-testid="design-dashboard"
      style={{
        display: "flex",
        height: "100%",
        fontFamily: type_scale.font_family,
        backgroundColor: "#f8fafc",
      }}
    >
      <Sidebar design={design} />

      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden", minWidth: 0 }}>
        <Header design={design} />

        <main
          style={{
            flex: 1,
            overflowY: "auto",
            overflowX: "hidden",
            padding: "20px 24px",
          }}
        >
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "20px",
              maxWidth: 1100,
            }}
          >
            {design.layout.content.sections.map((section, i) => (
              <div key={i} style={{ gridColumn: getGridColumn(section.width) }}>
                <ContentSectionRenderer section={section} design={design} />
              </div>
            ))}
          </div>
        </main>
      </div>
    </div>
  )
}
