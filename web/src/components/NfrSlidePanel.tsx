import { useEffect, useCallback } from "react"
import type { NfrInfo } from "@/types"
import {
  CheckCircle2,
  XCircle,
  X,
  FileText,
} from "lucide-react"

function NfrStatusIcon({ nfr }: { nfr: NfrInfo }) {
  const isInvalid =
    typeof nfr.validation_status === "object" && "Invalid" in nfr.validation_status
  if (isInvalid) {
    return <XCircle className="size-5 text-red-400" />
  }
  return <CheckCircle2 className="size-5 text-emerald-400" />
}

function ConstraintTypeBadge({ type }: { type: string }) {
  const colors =
    type === "metric"
      ? "bg-blue-500/20 text-blue-400"
      : "bg-purple-500/20 text-purple-400"
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${colors}`}>
      {type}
    </span>
  )
}

const VIOLATION_COLORS: Record<string, string> = {
  critical: "bg-red-500/20 text-red-400",
  high: "bg-orange-500/20 text-orange-400",
  medium: "bg-amber-500/20 text-amber-400",
  low: "bg-zinc-500/20 text-zinc-400",
  warning: "bg-amber-500/20 text-amber-400",
}

function ViolationBadge({ level }: { level: string }) {
  const colors = VIOLATION_COLORS[level.toLowerCase()] ?? "bg-zinc-500/20 text-zinc-400"
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium ${colors}`}>
      {level}
    </span>
  )
}

interface NfrSlidePanelProps {
  nfr: NfrInfo | null
  isOpen: boolean
  onClose: () => void
}

export function NfrSlidePanel({ nfr, isOpen, onClose }: NfrSlidePanelProps) {
  const isInvalid = nfr &&
    typeof nfr.validation_status === "object" &&
    "Invalid" in nfr.validation_status
  const errors = isInvalid
    ? (nfr.validation_status as { Invalid: string[] }).Invalid
    : []

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose()
      }
    },
    [isOpen, onClose]
  )

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown)
    return () => document.removeEventListener("keydown", handleKeyDown)
  }, [handleKeyDown])

  return (
    <>
      {/* Backdrop overlay */}
      <div
        data-testid="nfr-slide-panel-overlay"
        className={`fixed inset-0 z-40 bg-black/50 transition-opacity duration-300 ${
          isOpen ? "opacity-100" : "pointer-events-none opacity-0"
        }`}
        onClick={onClose}
      />

      {/* Slide panel */}
      <div
        data-testid="nfr-slide-panel"
        className={`fixed inset-y-0 right-0 z-50 w-[600px] transform border-l border-border bg-card shadow-2xl transition-transform duration-300 ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        {nfr && (
          <>
            {/* Header */}
            <div className="flex items-center gap-3 border-b border-border px-4 py-3">
              <button
                type="button"
                aria-label="Close"
                onClick={onClose}
                className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              >
                <X className="size-4" />
              </button>
              <div className="flex items-center gap-2">
                <NfrStatusIcon nfr={nfr} />
                <span className="font-mono text-sm font-medium text-foreground">
                  {nfr.category}
                </span>
                <span className="text-sm text-muted-foreground">
                  v{nfr.version}
                </span>
              </div>
            </div>

            {/* Content */}
            <div className="overflow-y-auto" style={{ height: "calc(100% - 56px)" }}>
              <div className="space-y-4 px-4 py-4">
                {/* Description */}
                {nfr.description && (
                  <p className="text-sm text-muted-foreground">
                    {nfr.description}
                  </p>
                )}

                {/* Validation errors */}
                {errors.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Validation Errors
                    </h3>
                    <div className="space-y-1">
                      {errors.map((err, i) => (
                        <div
                          key={i}
                          className="flex items-start gap-2 rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-400"
                        >
                          <XCircle className="mt-0.5 size-3 shrink-0" />
                          <span>{err}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Constraints */}
                {nfr.constraints.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Constraints ({nfr.constraints.length})
                    </h3>
                    <div className="space-y-2">
                      {nfr.constraints.map((constraint) => (
                        <div
                          key={constraint.name}
                          className="rounded-md px-3 py-2"
                        >
                          {/* Name + type badge */}
                          <div className="flex items-center gap-2">
                            <span className="font-mono text-xs font-medium text-foreground">
                              {constraint.name}
                            </span>
                            <ConstraintTypeBadge type={constraint.constraint_type} />
                          </div>

                          {/* Description */}
                          {constraint.description && (
                            <p className="mt-0.5 text-[11px] leading-tight text-muted-foreground">
                              {constraint.description}
                            </p>
                          )}

                          {/* Detail card */}
                          <div className="mt-1.5 space-y-1 rounded-md border border-border/50 bg-muted/40 px-3 py-2 shadow-sm">
                            {constraint.constraint_type === "metric" && constraint.threshold && (
                              <div className="flex items-center gap-2 text-xs">
                                <span className="text-muted-foreground">Threshold:</span>
                                <span className="font-mono text-foreground">{constraint.threshold}</span>
                              </div>
                            )}
                            {constraint.constraint_type === "rule" && constraint.rule_text && (
                              <div className="text-xs">
                                <span className="text-muted-foreground">Rule:</span>
                                <p className="mt-0.5 text-foreground/80 leading-relaxed">{constraint.rule_text}</p>
                              </div>
                            )}
                            <div className="flex items-center gap-2 text-xs">
                              <span className="text-muted-foreground">Violation:</span>
                              <ViolationBadge level={constraint.violation} />
                            </div>
                            <div className="flex items-center gap-2 text-xs">
                              <span className="text-muted-foreground">Overridable:</span>
                              <span className={constraint.overridable ? "text-emerald-400" : "text-muted-foreground"}>
                                {constraint.overridable ? "yes" : "no"}
                              </span>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Referenced by */}
                {nfr.referenced_by.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Referenced By
                    </h3>
                    <div className="space-y-1">
                      {nfr.referenced_by.map((specName) => (
                        <div
                          key={specName}
                          className="flex items-center gap-2 text-sm"
                        >
                          <FileText className="size-3 text-muted-foreground" />
                          <span className="font-mono text-xs text-foreground">
                            {specName}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </>
        )}
      </div>
    </>
  )
}
