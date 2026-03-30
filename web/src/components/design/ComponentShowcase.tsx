import type { DesignSystem, TypeScaleConfig } from "@/types"
import { Separator } from "@/components/ui/separator"
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Info,
  FileText,
  LayoutDashboard,
  Eye,
  Paintbrush,
  GitBranch,
  Settings,
} from "lucide-react"

// --- Helpers ---

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

/**
 * Normalize the type scale so sizes are always ascending (smallest first).
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

const navIcons: Record<string, React.ReactNode> = {
  Dashboard: <LayoutDashboard size={16} />,
  Settings: <Settings size={16} />,
  Profile: <FileText size={16} />,
  Overview: <LayoutDashboard size={16} />,
  Validate: <CheckCircle2 size={16} />,
  Watch: <Eye size={16} />,
  Format: <Paintbrush size={16} />,
  Graph: <GitBranch size={16} />,
}

// --- Section wrapper ---

function ShowcaseSection({
  title,
  testId,
  children,
  isLast = false,
  design,
}: {
  title: string
  testId: string
  children: React.ReactNode
  isLast?: boolean
  design: DesignSystem
}) {
  return (
    <section data-testid={testId}>
      <h2
        style={{
          fontSize: "14px",
          fontWeight: 700,
          textTransform: "uppercase",
          letterSpacing: "0.08em",
          color: design.palette.neutral,
          fontFamily: design.type_scale.font_family,
          marginBottom: "20px",
        }}
      >
        {title}
      </h2>
      {children}
      {!isLast && <Separator className="mt-10" />}
    </section>
  )
}

// --- 1. Color Palette ---

function ColorPaletteSection({ design }: { design: DesignSystem }) {
  const { palette } = design

  return (
    <ShowcaseSection title="Color Palette" testId="showcase-color-palette" design={design}>
      {/* Primary swatch */}
      <div style={{ marginBottom: "24px" }}>
        <div
          style={{
            width: "100%",
            height: 80,
            backgroundColor: palette.primary,
            borderRadius: design.component_styles.card_border_radius + "px",
            display: "flex",
            alignItems: "flex-end",
            padding: "12px 16px",
          }}
        >
          <span
            style={{
              color: "#ffffff",
              fontSize: "13px",
              fontFamily: "monospace",
              fontWeight: 500,
            }}
          >
            Primary {palette.primary}
          </span>
        </div>
      </div>

      {/* Shade ramp */}
      <div style={{ marginBottom: "24px" }}>
        <div
          style={{
            fontSize: "12px",
            fontWeight: 500,
            color: palette.neutral,
            marginBottom: "8px",
            fontFamily: design.type_scale.font_family,
          }}
        >
          Shade Ramp
        </div>
        <div style={{ display: "flex", gap: "4px" }}>
          {palette.shades.map((shade, i) => (
            <div key={i} style={{ flex: 1, textAlign: "center" }}>
              <div
                style={{
                  height: 40,
                  backgroundColor: shade,
                  borderRadius: 4,
                  marginBottom: "4px",
                }}
              />
              <span
                style={{
                  fontSize: "10px",
                  fontFamily: "monospace",
                  color: palette.neutral,
                }}
              >
                {shade}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Secondary, Accent, Neutral */}
      <div style={{ display: "flex", gap: "12px", marginBottom: "24px" }}>
        {[
          { label: "Secondary", color: palette.secondary },
          { label: "Accent", color: palette.accent },
          { label: "Neutral", color: palette.neutral },
        ].map((item) => (
          <div key={item.label} style={{ flex: 1 }}>
            <div
              style={{
                height: 48,
                backgroundColor: item.color,
                borderRadius: design.component_styles.card_border_radius + "px",
                marginBottom: "6px",
              }}
            />
            <div
              style={{
                fontSize: "12px",
                fontWeight: 500,
                color: palette.neutral,
                fontFamily: design.type_scale.font_family,
              }}
            >
              {item.label}
            </div>
            <div
              style={{
                fontSize: "11px",
                fontFamily: "monospace",
                color: palette.neutral,
              }}
            >
              {item.color}
            </div>
          </div>
        ))}
      </div>

      {/* Semantic colors */}
      <div style={{ display: "flex", gap: "12px" }}>
        {(
          [
            { label: "Success", color: palette.semantic.success },
            { label: "Warning", color: palette.semantic.warning },
            { label: "Error", color: palette.semantic.error },
            { label: "Info", color: palette.semantic.info },
          ] as const
        ).map((item) => (
          <div
            key={item.label}
            data-testid={`semantic-${item.label.toLowerCase()}`}
            style={{
              flex: 1,
              borderLeft: `3px solid ${item.color}`,
              backgroundColor: hexToRgba(item.color, 0.06),
              borderRadius: `0 ${design.component_styles.card_border_radius}px ${design.component_styles.card_border_radius}px 0`,
              padding: "12px 14px",
            }}
          >
            <div
              style={{
                fontSize: "12px",
                fontWeight: 600,
                color: item.color,
                fontFamily: design.type_scale.font_family,
                marginBottom: "2px",
              }}
            >
              {item.label}
            </div>
            <div
              style={{
                fontSize: "11px",
                fontFamily: "monospace",
                color: palette.neutral,
              }}
            >
              {item.color}
            </div>
          </div>
        ))}
      </div>
    </ShowcaseSection>
  )
}

// --- 2. Typography ---

function TypographySection({ design }: { design: DesignSystem }) {
  const { type_scale, palette } = design

  const sampleTexts = [
    "Caption text",
    "Small text",
    "Body text",
    "Subheading",
    "Component Library",
    "Design System",
    "Page Title",
    "Display",
  ]

  // Sizes are ascending (smallest first), render largest first
  const reversed = [...type_scale.sizes].reverse()
  const reversedLh = [...type_scale.line_heights].reverse()
  const reversedFw = [...type_scale.font_weights].reverse()

  return (
    <ShowcaseSection title="Typography" testId="showcase-typography" design={design}>
      {/* Font family */}
      <div
        style={{
          fontSize: "16px",
          fontWeight: 600,
          fontFamily: type_scale.font_family,
          color: palette.shades[9] ?? palette.neutral,
          marginBottom: "24px",
        }}
      >
        {type_scale.font_family}
      </div>

      {/* Type scale levels */}
      <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
        {reversed.map((size, i) => {
          const lh = reversedLh[i] ?? 1.4
          const fw = reversedFw[i] ?? 400
          const label = sampleTexts[type_scale.sizes.length - 1 - i] ?? `Level ${i + 1}`

          return (
            <div
              key={i}
              style={{
                display: "flex",
                alignItems: "baseline",
                justifyContent: "space-between",
                gap: "24px",
              }}
            >
              <span
                style={{
                  fontSize: `${size}px`,
                  fontWeight: fw,
                  lineHeight: lh,
                  fontFamily: type_scale.font_family,
                  color: palette.shades[9] ?? palette.neutral,
                }}
              >
                {label}
              </span>
              <span
                style={{
                  fontSize: "12px",
                  fontFamily: "monospace",
                  color: palette.neutral,
                  whiteSpace: "nowrap",
                  flexShrink: 0,
                }}
              >
                {size}px / {fw} / {lh}
              </span>
            </div>
          )
        })}
      </div>
    </ShowcaseSection>
  )
}

// --- 3. Spacing ---

function SpacingSection({ design }: { design: DesignSystem }) {
  const { spacing, palette } = design
  const maxStep = Math.max(...spacing.steps)

  return (
    <ShowcaseSection title="Spacing" testId="showcase-spacing" design={design}>
      <div
        style={{
          fontSize: "13px",
          fontWeight: 500,
          color: palette.shades[9] ?? palette.neutral,
          fontFamily: design.type_scale.font_family,
          marginBottom: "16px",
        }}
      >
        Base unit: {spacing.base}px
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
        {spacing.steps.map((step, i) => (
          <div
            key={i}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "12px",
            }}
          >
            <div
              style={{
                width: `${(step / maxStep) * 100}%`,
                minWidth: 4,
                height: 24,
                backgroundColor: hexToRgba(palette.primary, 0.15),
                borderRadius: 4,
              }}
            />
            <span
              style={{
                fontSize: "12px",
                fontFamily: "monospace",
                color: palette.neutral,
                whiteSpace: "nowrap",
                flexShrink: 0,
              }}
            >
              {step}px
            </span>
          </div>
        ))}
      </div>
    </ShowcaseSection>
  )
}

// --- 4. Navigation ---

function NavigationSection({ design }: { design: DesignSystem }) {
  const { palette } = design

  // Get nav items from spec metadata or fallback
  const specNames = design.spec_metadata?.domains
    ?.flatMap((d) => d.spec_names)
    .slice(0, 7) ?? []
  const navItems =
    specNames.length > 0
      ? specNames.map((n) => {
          const parts = n.split("-")
          return parts.map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join(" ")
        })
      : ["Dashboard", "Settings", "Profile"]

  const activeItem = navItems[0]
  const sidebarBg = palette.shades[9] ?? palette.neutral
  const sidebarText = palette.shades[0] ?? "#ffffff"
  const sidebarTextMuted = palette.shades[3] ?? palette.neutral

  return (
    <ShowcaseSection title="Navigation" testId="showcase-navigation" design={design}>
      <div
        data-testid="showcase-nav-preview"
        style={{
          width: 260,
          backgroundColor: sidebarBg,
          borderRadius: design.component_styles.card_border_radius + "px",
          padding: "16px 12px",
          fontFamily: design.type_scale.font_family,
        }}
      >
        {/* Project header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            marginBottom: "16px",
            paddingLeft: "8px",
          }}
        >
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
              color: sidebarText,
              letterSpacing: "-0.02em",
            }}
          >
            {design.spec_metadata?.project_name ?? "Project"}
          </span>
        </div>

        {/* Nav items */}
        <nav>
          {navItems.map((item) => {
            const isActive = item === activeItem
            return (
              <div
                key={item}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "8px",
                  padding: "6px 10px",
                  borderRadius: 6,
                  marginBottom: 2,
                  backgroundColor: isActive
                    ? hexToRgba(palette.primary, 0.2)
                    : "transparent",
                  color: isActive ? sidebarText : sidebarTextMuted,
                  fontWeight: isActive ? 500 : 400,
                  fontSize: "13px",
                  cursor: "pointer",
                }}
              >
                <span style={{ opacity: isActive ? 1 : 0.6, display: "flex", alignItems: "center" }}>
                  {navIcons[item] ?? <FileText size={16} />}
                </span>
                {item}
              </div>
            )
          })}
        </nav>
      </div>
    </ShowcaseSection>
  )
}

// --- 5. Metric Cards ---

function MetricCardsSection({ design }: { design: DesignSystem }) {
  const { palette, type_scale, component_styles, spec_metadata } = design

  const metrics = [
    { label: "Specs", value: spec_metadata?.total_spec_count ?? 0 },
    { label: "Behaviors", value: spec_metadata?.total_behavior_count ?? 0 },
    { label: "Entities", value: spec_metadata?.total_entity_count ?? 0 },
  ]

  return (
    <ShowcaseSection title="Metric Cards" testId="showcase-metric-cards" design={design}>
      <div style={{ display: "flex", gap: "16px" }}>
        {metrics.map((metric) => (
          <div
            key={metric.label}
            style={{
              flex: 1,
              backgroundColor: "#ffffff",
              border: `1px solid ${hexToRgba(palette.neutral, 0.12)}`,
              borderRadius: component_styles.card_border_radius + "px",
              padding: "20px",
              fontFamily: type_scale.font_family,
            }}
          >
            <div
              style={{
                fontSize: "12px",
                fontWeight: 500,
                color: palette.neutral,
                textTransform: "uppercase",
                letterSpacing: "0.04em",
                marginBottom: "8px",
              }}
            >
              {metric.label}
            </div>
            <div
              style={{
                fontSize: "28px",
                fontWeight: 600,
                color: palette.shades[9] ?? palette.neutral,
                lineHeight: 1.2,
                letterSpacing: "-0.02em",
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {metric.value}
            </div>
          </div>
        ))}
      </div>
    </ShowcaseSection>
  )
}

// --- 6. Data Table ---

function DataTableSection({ design }: { design: DesignSystem }) {
  const { palette, type_scale, component_styles, spec_metadata } = design

  const rows = (spec_metadata?.spec_metrics ?? []).slice(0, 8)

  return (
    <ShowcaseSection title="Data Table" testId="showcase-data-table" design={design}>
      <div
        style={{
          backgroundColor: "#ffffff",
          border: `1px solid ${hexToRgba(palette.neutral, 0.12)}`,
          borderRadius: component_styles.card_border_radius + "px",
          overflow: "hidden",
          fontFamily: type_scale.font_family,
        }}
      >
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
                backgroundColor: hexToRgba(palette.neutral, 0.03),
              }}
            >
              {["Name", "Version", "Behaviors", "Status"].map((col) => (
                <th
                  key={col}
                  style={{
                    fontSize: "11px",
                    fontWeight: 600,
                    color: palette.neutral,
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                    padding: "10px 16px",
                    textAlign: "left",
                  }}
                >
                  {col}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((spec, i) => (
              <tr
                key={spec.name}
                style={{
                  borderBottom:
                    i < rows.length - 1
                      ? `1px solid ${hexToRgba(palette.neutral, 0.06)}`
                      : undefined,
                }}
              >
                <td
                  style={{
                    fontSize: "13px",
                    fontWeight: 500,
                    color: palette.shades[8] ?? palette.neutral,
                    fontFamily: "monospace",
                    padding: "10px 16px",
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
                    padding: "10px 16px",
                  }}
                >
                  v{spec.version}
                </td>
                <td
                  style={{
                    fontSize: "13px",
                    color: palette.shades[7] ?? palette.neutral,
                    fontVariantNumeric: "tabular-nums",
                    padding: "10px 16px",
                  }}
                >
                  {spec.behavior_count}
                </td>
                <td style={{ padding: "10px 16px" }}>
                  <span
                    style={{
                      display: "inline-flex",
                      alignItems: "center",
                      gap: "4px",
                      fontSize: "11px",
                      fontWeight: 500,
                      color: palette.semantic.success,
                      backgroundColor: hexToRgba(palette.semantic.success, 0.08),
                      borderRadius: 12,
                      padding: "2px 10px",
                    }}
                  >
                    <span
                      style={{
                        width: 5,
                        height: 5,
                        borderRadius: "50%",
                        backgroundColor: palette.semantic.success,
                      }}
                    />
                    Valid
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </ShowcaseSection>
  )
}

// --- 7. Buttons ---

function ButtonsSection({ design }: { design: DesignSystem }) {
  const { palette, type_scale, component_styles } = design

  const buttonBase: React.CSSProperties = {
    fontFamily: type_scale.font_family,
    fontSize: "13px",
    fontWeight: 500,
    padding: "8px 20px",
    borderRadius: component_styles.button_border_radius + "px",
    cursor: "pointer",
    border: "none",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    transition: "background-color 0.15s",
  }

  return (
    <ShowcaseSection title="Buttons" testId="showcase-buttons" design={design}>
      <div style={{ display: "flex", gap: "12px", alignItems: "center", flexWrap: "wrap" }}>
        {/* Primary */}
        <button
          type="button"
          style={{
            ...buttonBase,
            backgroundColor: palette.primary,
            color: "#ffffff",
          }}
        >
          Primary
        </button>

        {/* Secondary */}
        <button
          type="button"
          style={{
            ...buttonBase,
            backgroundColor: "transparent",
            color: palette.secondary,
            border: `1px solid ${palette.secondary}`,
          }}
        >
          Secondary
        </button>

        {/* Ghost */}
        <button
          type="button"
          style={{
            ...buttonBase,
            backgroundColor: "transparent",
            color: palette.neutral,
          }}
        >
          Ghost
        </button>

        {/* Destructive */}
        <button
          type="button"
          style={{
            ...buttonBase,
            backgroundColor: palette.semantic.error,
            color: "#ffffff",
          }}
        >
          Destructive
        </button>
      </div>
    </ShowcaseSection>
  )
}

// --- 8. Form Elements ---

function FormElementsSection({ design }: { design: DesignSystem }) {
  const { palette, type_scale, component_styles } = design

  const labelStyle: React.CSSProperties = {
    fontSize: "13px",
    fontWeight: 500,
    color: palette.shades[8] ?? palette.neutral,
    fontFamily: type_scale.font_family,
    marginBottom: "6px",
    display: "block",
  }

  const inputStyle: React.CSSProperties = {
    width: "100%",
    fontSize: "13px",
    fontFamily: type_scale.font_family,
    padding: "8px 12px",
    borderRadius: component_styles.button_border_radius + "px",
    border: `1px solid ${hexToRgba(palette.neutral, 0.2)}`,
    outline: "none",
    color: palette.shades[9] ?? palette.neutral,
    backgroundColor: "#ffffff",
  }

  return (
    <ShowcaseSection title="Form Elements" testId="showcase-form-elements" design={design}>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "24px" }}>
        {/* Text input */}
        <div>
          <label style={labelStyle}>Email Address</label>
          <input
            type="text"
            placeholder="name@example.com"
            readOnly
            style={inputStyle}
          />
        </div>

        {/* Select */}
        <div>
          <label style={labelStyle}>Category</label>
          <select
            style={{
              ...inputStyle,
              appearance: "auto",
            }}
            defaultValue=""
          >
            <option value="" disabled>
              Select a category...
            </option>
            <option>Commands</option>
            <option>Grammar</option>
            <option>Infrastructure</option>
          </select>
        </div>

        {/* Checkbox */}
        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <label style={{ ...labelStyle, marginBottom: 0 }}>Options</label>
          {["Enable notifications", "Auto-validate on save"].map((text) => (
            <label
              key={text}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                fontSize: "13px",
                fontFamily: type_scale.font_family,
                color: palette.shades[8] ?? palette.neutral,
                cursor: "pointer",
              }}
            >
              <input
                type="checkbox"
                readOnly
                style={{
                  accentColor: palette.primary,
                  width: 16,
                  height: 16,
                }}
              />
              {text}
            </label>
          ))}
        </div>

        {/* Radio */}
        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <label style={{ ...labelStyle, marginBottom: 0 }}>Theme</label>
          {["Light", "Dark", "System"].map((text, i) => (
            <label
              key={text}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                fontSize: "13px",
                fontFamily: type_scale.font_family,
                color: palette.shades[8] ?? palette.neutral,
                cursor: "pointer",
              }}
            >
              <input
                type="radio"
                name="theme"
                defaultChecked={i === 0}
                readOnly
                style={{
                  accentColor: palette.primary,
                  width: 16,
                  height: 16,
                }}
              />
              {text}
            </label>
          ))}
        </div>
      </div>
    </ShowcaseSection>
  )
}

// --- 9. Alerts & Badges ---

function AlertsBadgesSection({ design }: { design: DesignSystem }) {
  const { palette, type_scale, component_styles } = design

  const alerts = [
    {
      label: "Success",
      color: palette.semantic.success,
      icon: <CheckCircle2 size={16} />,
      message: "All specifications validated successfully.",
    },
    {
      label: "Warning",
      color: palette.semantic.warning,
      icon: <AlertTriangle size={16} />,
      message: "Some specs have deprecated syntax.",
    },
    {
      label: "Error",
      color: palette.semantic.error,
      icon: <XCircle size={16} />,
      message: "Validation failed for 2 specifications.",
    },
    {
      label: "Info",
      color: palette.semantic.info,
      icon: <Info size={16} />,
      message: "A new version of minter is available.",
    },
  ]

  return (
    <ShowcaseSection title="Alerts & Badges" testId="showcase-alerts-badges" design={design} isLast>
      {/* Alerts */}
      <div style={{ display: "flex", flexDirection: "column", gap: "10px", marginBottom: "24px" }}>
        {alerts.map((alert) => (
          <div
            key={alert.label}
            data-testid={`alert-${alert.label.toLowerCase()}`}
            style={{
              borderLeft: `3px solid ${alert.color}`,
              backgroundColor: hexToRgba(alert.color, 0.05),
              borderRadius: `0 ${component_styles.card_border_radius}px ${component_styles.card_border_radius}px 0`,
              padding: "12px 16px",
              display: "flex",
              alignItems: "center",
              gap: "10px",
              fontFamily: type_scale.font_family,
            }}
          >
            <span style={{ color: alert.color, display: "flex", flexShrink: 0 }}>
              {alert.icon}
            </span>
            <span
              style={{
                fontSize: "13px",
                color: palette.shades[8] ?? palette.neutral,
              }}
            >
              {alert.message}
            </span>
          </div>
        ))}
      </div>

      {/* Badges */}
      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        {[
          { label: "Valid", color: palette.semantic.success },
          { label: "Warning", color: palette.semantic.warning },
          { label: "Error", color: palette.semantic.error },
          { label: "Info", color: palette.semantic.info },
          { label: "Primary", color: palette.primary },
          { label: "Accent", color: palette.accent },
        ].map((badge) => (
          <span
            key={badge.label}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "4px",
              fontSize: "11px",
              fontWeight: 500,
              fontFamily: type_scale.font_family,
              color: badge.color,
              backgroundColor: hexToRgba(badge.color, 0.1),
              borderRadius: 12,
              padding: "3px 12px",
            }}
          >
            <span
              style={{
                width: 5,
                height: 5,
                borderRadius: "50%",
                backgroundColor: badge.color,
              }}
            />
            {badge.label}
          </span>
        ))}
      </div>
    </ShowcaseSection>
  )
}

// --- Main Component ---

interface ComponentShowcaseProps {
  design: DesignSystem
}

export function ComponentShowcase({ design: rawDesign }: ComponentShowcaseProps) {
  const design: DesignSystem = {
    ...rawDesign,
    type_scale: normalizeTypeScale(rawDesign.type_scale),
  }

  return (
    <div
      data-testid="component-showcase"
      style={{
        backgroundColor: "#ffffff",
        minHeight: "100%",
        fontFamily: design.type_scale.font_family,
      }}
    >
      <div
        style={{
          maxWidth: 1100,
          margin: "0 auto",
          padding: "40px 32px",
          display: "flex",
          flexDirection: "column",
          gap: "40px",
        }}
      >
        <ColorPaletteSection design={design} />
        <TypographySection design={design} />
        <SpacingSection design={design} />
        <NavigationSection design={design} />
        <MetricCardsSection design={design} />
        <DataTableSection design={design} />
        <ButtonsSection design={design} />
        <FormElementsSection design={design} />
        <AlertsBadgesSection design={design} />
      </div>
    </div>
  )
}
