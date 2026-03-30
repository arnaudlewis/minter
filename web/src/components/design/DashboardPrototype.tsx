import type {
  DesignSystem,
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
  const { palette, layout, spacing, type_scale } = design
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
          padding: `${spacing.steps[4]}px ${spacing.steps[3]}px`,
          borderBottom: `1px solid ${hexToRgba(sidebarText, 0.08)}`,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: spacing.steps[1] + "px" }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: 6,
              background: palette.primary,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontWeight: type_scale.font_weights[2],
              fontSize: type_scale.sizes[1] + "px",
              color: "#fff",
            }}
          >
            M
          </div>
          <span
            style={{
              fontWeight: type_scale.font_weights[2],
              fontSize: type_scale.sizes[3] + "px",
              letterSpacing: "-0.02em",
            }}
          >
            {design.spec_metadata.project_name}
          </span>
        </div>
      </div>

      {/* Navigation */}
      <nav
        style={{
          flex: 1,
          padding: `${spacing.steps[2]}px ${spacing.steps[2]}px`,
          overflowY: "auto",
        }}
      >
        <div
          style={{
            fontSize: type_scale.sizes[0] + "px",
            fontWeight: type_scale.font_weights[1],
            color: sidebarTextMuted,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            padding: `${spacing.steps[1]}px ${spacing.steps[2]}px`,
            marginBottom: spacing.steps[0] + "px",
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
                gap: spacing.steps[2] + "px",
                padding: `${spacing.steps[1] + 2}px ${spacing.steps[2]}px`,
                borderRadius: 6,
                marginBottom: 2,
                backgroundColor: isActive
                  ? hexToRgba(palette.primary, 0.15)
                  : "transparent",
                color: isActive ? palette.shades[1] : sidebarTextMuted,
                fontWeight: isActive
                  ? type_scale.font_weights[1]
                  : type_scale.font_weights[0],
                fontSize: type_scale.sizes[2] + "px",
                cursor: "pointer",
                transition: "background-color 0.15s, color 0.15s",
              }}
            >
              <span style={{ opacity: isActive ? 1 : 0.6 }}>
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
            margin: `${spacing.steps[3]}px ${spacing.steps[2]}px`,
          }}
        />

        {/* Domain summary */}
        <div
          style={{
            fontSize: type_scale.sizes[0] + "px",
            fontWeight: type_scale.font_weights[1],
            color: sidebarTextMuted,
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            padding: `${spacing.steps[1]}px ${spacing.steps[2]}px`,
            marginBottom: spacing.steps[0] + "px",
          }}
        >
          Domains
        </div>
        {design.spec_metadata.domains.map((domain) => (
          <div
            key={domain.name}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: `${spacing.steps[1]}px ${spacing.steps[2]}px`,
              fontSize: type_scale.sizes[1] + "px",
              color: sidebarTextMuted,
            }}
          >
            <span>{domain.name}</span>
            <span
              style={{
                fontSize: type_scale.sizes[0] + "px",
                backgroundColor: hexToRgba(sidebarText, 0.08),
                borderRadius: 10,
                padding: "1px 7px",
                fontWeight: type_scale.font_weights[1],
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
          padding: `${spacing.steps[3]}px`,
          borderTop: `1px solid ${hexToRgba(sidebarText, 0.08)}`,
          fontSize: type_scale.sizes[0] + "px",
          color: sidebarTextMuted,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: spacing.steps[1] + "px" }}>
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
  const { palette, layout, spacing, type_scale, component_styles } = design
  const header = layout.header

  return (
    <header
      data-testid="design-header"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: `${spacing.steps[2]}px ${spacing.steps[4]}px`,
        backgroundColor: "#ffffff",
        borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.12)}`,
        fontFamily: type_scale.font_family,
        height: 56,
        minHeight: 56,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: spacing.steps[2] + "px" }}>
        <span
          style={{
            fontSize: type_scale.sizes[3] + "px",
            fontWeight: type_scale.font_weights[2],
            color: palette.shades[9],
            letterSpacing: "-0.02em",
          }}
        >
          {header?.title ?? "Dashboard"}
        </span>
        <span
          style={{
            color: hexToRgba(palette.neutral, 0.4),
            fontSize: type_scale.sizes[3] + "px",
          }}
        >
          /
        </span>
        <span
          style={{
            fontSize: type_scale.sizes[2] + "px",
            color: palette.neutral,
          }}
        >
          Overview
        </span>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: spacing.steps[2] + "px" }}>
        {header?.has_search && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: spacing.steps[1] + "px",
              padding: `${spacing.steps[1]}px ${spacing.steps[2]}px`,
              borderRadius: component_styles.button_border_radius + "px",
              border: `1px solid ${hexToRgba(palette.neutral, 0.15)}`,
              backgroundColor: hexToRgba(palette.neutral, 0.03),
              color: hexToRgba(palette.neutral, 0.5),
              fontSize: type_scale.sizes[2] + "px",
              minWidth: 220,
            }}
          >
            <Search size={14} />
            <span>Search specs...</span>
            <span
              style={{
                marginLeft: "auto",
                fontSize: type_scale.sizes[0] + "px",
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
            width: 32,
            height: 32,
            borderRadius: "50%",
            background: `linear-gradient(135deg, ${palette.primary}, ${palette.accent})`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#fff",
            fontWeight: type_scale.font_weights[2],
            fontSize: type_scale.sizes[1] + "px",
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
  const { palette, spacing, type_scale, component_styles, spec_metadata } = design

  const coveragePct = spec_metadata.total_behavior_count > 0
    ? Math.round((spec_metadata.total_behavior_count * 0.85) / spec_metadata.total_behavior_count * 100)
    : 0

  const metrics: MetricCardData[] = [
    {
      label: "Specifications",
      value: spec_metadata.total_spec_count,
      icon: <FileText size={18} style={{ color: palette.primary }} />,
      trend: "up",
      trendValue: "+3 this week",
    },
    {
      label: "Behaviors",
      value: spec_metadata.total_behavior_count,
      icon: <Hash size={18} style={{ color: palette.accent }} />,
      trend: "up",
      trendValue: "+18 this week",
    },
    {
      label: "Entities",
      value: spec_metadata.total_entity_count,
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
        gap: spacing.steps[3] + "px",
      }}
    >
      {metrics.map((metric) => (
        <div
          key={metric.label}
          style={{
            backgroundColor: "#ffffff",
            border: `1px solid ${hexToRgba(palette.neutral, 0.10)}`,
            borderRadius: component_styles.card_border_radius + "px",
            padding: `${spacing.steps[4]}px`,
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
              marginBottom: spacing.steps[2] + "px",
            }}
          >
            <span
              style={{
                fontSize: type_scale.sizes[1] + "px",
                fontWeight: type_scale.font_weights[1],
                color: palette.neutral,
                textTransform: "uppercase",
                letterSpacing: "0.04em",
              }}
            >
              {metric.label}
            </span>
            <div
              style={{
                width: 32,
                height: 32,
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
              fontSize: type_scale.sizes[5] + "px",
              fontWeight: type_scale.font_weights[2],
              color: palette.shades[9],
              lineHeight: type_scale.line_heights[5],
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
              marginTop: spacing.steps[1] + "px",
              fontSize: type_scale.sizes[0] + "px",
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
  const { palette, spacing, type_scale, component_styles, spec_metadata } = design

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
      {/* Table header */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "2fr 1fr 1fr 1fr",
          padding: `${spacing.steps[2]}px ${spacing.steps[4]}px`,
          borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.08)}`,
          backgroundColor: hexToRgba(palette.neutral, 0.02),
        }}
      >
        {["Name", "Version", "Behaviors", "Status"].map((col) => (
          <span
            key={col}
            style={{
              fontSize: type_scale.sizes[0] + "px",
              fontWeight: type_scale.font_weights[1],
              color: palette.neutral,
              textTransform: "uppercase",
              letterSpacing: "0.06em",
            }}
          >
            {col}
          </span>
        ))}
      </div>

      {/* Table rows */}
      {spec_metadata.spec_metrics.map((spec, i) => {
        const status = statusMap[spec.name] ?? "pass"
        const color = statusColors[status]
        const isEven = i % 2 === 0

        return (
          <div
            key={spec.name}
            style={{
              display: "grid",
              gridTemplateColumns: "2fr 1fr 1fr 1fr",
              padding: `${spacing.steps[2]}px ${spacing.steps[4]}px`,
              borderBottom: `1px solid ${hexToRgba(palette.neutral, 0.05)}`,
              backgroundColor: isEven
                ? "transparent"
                : hexToRgba(palette.neutral, 0.015),
              alignItems: "center",
              transition: "background-color 0.15s",
              cursor: "pointer",
            }}
          >
            <span
              style={{
                fontSize: type_scale.sizes[2] + "px",
                fontWeight: type_scale.font_weights[1],
                color: palette.shades[8],
                fontFamily: "monospace",
              }}
            >
              {spec.name}
            </span>
            <span
              style={{
                fontSize: type_scale.sizes[1] + "px",
                color: palette.neutral,
                fontFamily: "monospace",
              }}
            >
              v{spec.version}
            </span>
            <span
              style={{
                fontSize: type_scale.sizes[2] + "px",
                color: palette.shades[7],
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {spec.behavior_count}
            </span>
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                fontSize: type_scale.sizes[0] + "px",
                fontWeight: type_scale.font_weights[1],
                color: color.text,
                backgroundColor: color.bg,
                borderRadius: 12,
                padding: "2px 10px",
                width: "fit-content",
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
          </div>
        )
      })}
    </div>
  )
}

// --- Status Panel ---

function StatusPanel({
  design,
}: {
  design: DesignSystem
}) {
  const { palette, spacing, type_scale, component_styles, spec_metadata } = design

  const total = spec_metadata.total_spec_count
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
        padding: spacing.steps[4] + "px",
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
          marginBottom: spacing.steps[4] + "px",
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
      <div style={{ display: "flex", flexDirection: "column", gap: spacing.steps[3] + "px" }}>
        {statuses.map((s) => (
          <div
            key={s.label}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: spacing.steps[2] + "px" }}>
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
                  fontSize: type_scale.sizes[2] + "px",
                  color: palette.shades[7],
                }}
              >
                {s.label}
              </span>
            </div>
            <span
              style={{
                fontSize: type_scale.sizes[3] + "px",
                fontWeight: type_scale.font_weights[2],
                color: palette.shades[9],
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
          margin: `${spacing.steps[3]}px 0`,
        }}
      />

      {/* Summary */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          fontSize: type_scale.sizes[2] + "px",
        }}
      >
        <span style={{ color: palette.neutral }}>Total specifications</span>
        <span
          style={{
            fontWeight: type_scale.font_weights[2],
            color: palette.shades[9],
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
  const { type_scale, palette, spacing } = design

  const sectionTitle = (
    <h2
      style={{
        fontSize: type_scale.sizes[3] + "px",
        fontWeight: type_scale.font_weights[2],
        color: palette.shades[9],
        letterSpacing: "-0.01em",
        marginBottom: spacing.steps[3] + "px",
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

export function DashboardPrototype({ design }: DashboardPrototypeProps) {
  const { spacing, type_scale } = design

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

      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <Header design={design} />

        <main
          style={{
            flex: 1,
            overflowY: "auto",
            padding: spacing.steps[5] + "px",
          }}
        >
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: spacing.steps[5] + "px",
              maxWidth: 1200,
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
