import type { ProjectState } from "@/types"
import { Button } from "@/components/ui/button"
import { AlertCircle, Loader2, RefreshCw } from "lucide-react"

function CoverageBar({
  covered,
  total,
}: {
  covered: number
  total: number
}) {
  const pct = total > 0 ? Math.round((covered / total) * 100) : 0
  const color =
    pct >= 80
      ? "bg-emerald-400"
      : pct >= 50
        ? "bg-amber-400"
        : "bg-red-400"
  const textColor =
    pct >= 80
      ? "text-emerald-400"
      : pct >= 50
        ? "text-amber-400"
        : "text-red-400"

  return (
    <div className="flex items-center gap-2">
      <span className="text-sm text-muted-foreground">Coverage</span>
      <div className="h-1.5 w-20 rounded-full bg-muted/50">
        <div
          className={`h-full rounded-full transition-all duration-500 ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className={`text-sm font-semibold tabular-nums ${textColor}`}>
        {pct}%
      </span>
    </div>
  )
}

function LockStatus({
  lockStatus,
  lockLoading,
  onRegenerateLock,
}: {
  lockStatus: "Aligned" | "Drifted" | "NoLock"
  lockLoading: boolean
  onRegenerateLock: () => void
}) {
  const label =
    lockStatus === "Aligned"
      ? "aligned"
      : lockStatus === "Drifted"
        ? "drifted"
        : "no lock"

  const statusColor =
    lockStatus === "Aligned"
      ? "text-emerald-400"
      : lockStatus === "Drifted"
        ? "text-amber-400"
        : "text-zinc-400"

  const showRegenerate = lockStatus === "Drifted" || lockStatus === "NoLock"

  return (
    <div className="flex items-center gap-2">
      <span className="text-sm text-muted-foreground">Lock:</span>
      <span className={`text-sm font-medium ${statusColor}`}>{label}</span>
      {showRegenerate && (
        <Button
          variant="ghost"
          size="xs"
          disabled={lockLoading}
          onClick={onRegenerateLock}
          aria-label="Regenerate"
        >
          {lockLoading ? (
            <Loader2 className="animate-spin" />
          ) : (
            <RefreshCw />
          )}
          <span>{lockLoading ? "Regenerating..." : "Regenerate"}</span>
        </Button>
      )}
    </div>
  )
}

interface MetricsBarProps {
  state: ProjectState | null
  connected: boolean
  loading: boolean
  lockLoading: boolean
  onRegenerateLock: () => void
  invalidTagCount?: number
  onShowInvalidTags?: () => void
}

export function MetricsBar({
  state,
  connected,
  loading,
  lockLoading,
  onRegenerateLock,
  invalidTagCount = 0,
  onShowInvalidTags,
}: MetricsBarProps) {
  if (loading || !state) {
    return (
      <header className="border-b border-border bg-card px-5 py-2.5">
        <div className="flex items-center gap-3">
          <span className="text-sm font-semibold tracking-tight">minter</span>
          <div className="flex-1" />
          <div className="h-3 w-40 animate-pulse rounded bg-muted" />
        </div>
      </header>
    )
  }

  const totalBehaviors = state.specs.reduce(
    (sum, s) => sum + s.behavior_count,
    0
  )

  return (
    <header className="border-b border-border bg-card px-5 py-2.5">
      <div className="flex items-center gap-4 text-sm">
        <span className="font-semibold tracking-tight">minter</span>

        <div className="flex items-center gap-1.5">
          <span
            className={`inline-block size-2 rounded-full ${
              connected ? "bg-emerald-400" : "bg-red-400"
            }`}
          />
          <span className="text-muted-foreground">
            {connected ? "connected" : "disconnected"}
          </span>
        </div>

        <div className="text-muted-foreground">
          {state.specs.length} specs
          {" \u00B7 "}
          {totalBehaviors} behaviors
          {" \u00B7 "}
          {state.nfr_count} NFRs
          {" \u00B7 "}
          {state.test_count} tags
        </div>

        <div className="flex-1" />

        {invalidTagCount > 0 && onShowInvalidTags && (
          <>
            <button
              type="button"
              className="flex cursor-pointer items-center gap-1.5 rounded-md bg-red-500/15 px-2 py-0.5 text-xs font-medium text-red-400 transition-colors hover:bg-red-500/25"
              onClick={onShowInvalidTags}
            >
              <AlertCircle className="size-3" />
              <span>{invalidTagCount}</span>
              <span className="text-red-400/70">invalid tag{invalidTagCount !== 1 ? "s" : ""}</span>
            </button>
            <div className="h-4 w-px bg-border" />
          </>
        )}

        <CoverageBar
          covered={state.coverage_covered}
          total={state.coverage_total}
        />

        <div className="h-4 w-px bg-border" />

        <LockStatus
          lockStatus={state.integrity.lock_status}
          lockLoading={lockLoading}
          onRegenerateLock={onRegenerateLock}
        />
      </div>
    </header>
  )
}
