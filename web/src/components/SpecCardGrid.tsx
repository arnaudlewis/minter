import { useState } from "react"
import type { SpecInfo, NfrInfo } from "@/types"
import { Input } from "@/components/ui/input"
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Search,
} from "lucide-react"

function getSpecStatus(spec: SpecInfo, depErrorCount: number): "valid" | "warning" | "error" {
  if (typeof spec.validation_status === "object" && "Invalid" in spec.validation_status) {
    return "error"
  }
  if (depErrorCount > 0) {
    return "error"
  }
  if (spec.validation_status === "Valid") {
    const uncovered = spec.behaviors.filter((b) => !b.covered)
    if (uncovered.length > 0) return "warning"
    return "valid"
  }
  return "warning"
}

function StatusIcon({ status }: { status: "valid" | "warning" | "error" }) {
  switch (status) {
    case "valid":
      return <CheckCircle2 className="size-4 shrink-0 text-emerald-400" />
    case "warning":
      return <AlertTriangle className="size-4 shrink-0 text-amber-400" />
    case "error":
      return <XCircle className="size-4 shrink-0 text-red-400" />
  }
}

function CoverageMiniBar({ covered, total }: { covered: number; total: number }) {
  const pct = total > 0 ? Math.round((covered / total) * 100) : 0
  const color =
    pct >= 80
      ? "bg-emerald-400"
      : pct >= 50
        ? "bg-amber-400"
        : "bg-red-400"

  return (
    <div className="flex items-center gap-1.5">
      <div className="h-1.5 w-20 rounded-full bg-muted/50">
        <div
          className={`h-full rounded-full transition-all duration-300 ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs tabular-nums text-muted-foreground">{pct}%</span>
    </div>
  )
}

function SpecCard({
  spec,
  nfrs,
  onClick,
  onClickNfr,
}: {
  spec: SpecInfo
  nfrs: NfrInfo[]
  onClick: () => void
  onClickNfr?: (nfr: NfrInfo) => void
}) {
  const depErrors = spec.dep_errors ?? []
  const status = getSpecStatus(spec, depErrors.length)
  const isInvalid = typeof spec.validation_status === "object" && "Invalid" in spec.validation_status
  const errors = isInvalid ? (spec.validation_status as { Invalid: string[] }).Invalid : []
  const uncoveredBehaviors = spec.behaviors.filter((b) => !b.covered)
  const coveredCount = spec.behaviors.filter((b) => b.covered).length

  return (
    <button
      type="button"
      data-testid="spec-card"
      className="cursor-pointer rounded-lg border border-border bg-card p-3 text-left transition-all hover:border-zinc-600 hover:shadow-md"
      onClick={onClick}
    >
      {/* Line 1: status icon + name + version */}
      <div className="flex items-center gap-2">
        <StatusIcon status={status} />
        <span className="truncate font-mono text-[13px] font-medium text-foreground">
          {spec.name}
        </span>
        <span className="ml-auto shrink-0 font-mono text-xs text-muted-foreground">
          v{spec.version}
        </span>
      </div>

      {/* Description */}
      {spec.description && (
        <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">
          {spec.description}
        </p>
      )}

      {/* Behavior count + coverage */}
      <div className="mt-1.5 text-xs text-muted-foreground">
        {spec.behavior_count} behaviors
        {!isInvalid && spec.behavior_count > 0 && (
          <> &middot; {Math.round((coveredCount / spec.behavior_count) * 100)}% coverage</>
        )}
      </div>

      {/* NFR category badges (from real NFR data) */}
      {(() => {
        const categories = [...new Set(spec.nfr_refs.map(r => r.split("#")[0]))]
        const matchingNfrs = nfrs.filter(n => categories.includes(n.category))
        if (matchingNfrs.length === 0) return null
        return (
          <div className="mt-1.5 flex flex-wrap gap-1">
            {matchingNfrs.map(nfr => (
              <span
                key={nfr.category}
                className={`rounded-full bg-zinc-500/20 px-1.5 py-0.5 text-[10px] text-zinc-400 ${onClickNfr ? "cursor-pointer transition-colors hover:bg-zinc-500/30 hover:text-zinc-300" : ""}`}
                onClick={onClickNfr ? (e) => { e.stopPropagation(); onClickNfr(nfr) } : undefined}
              >
                {nfr.category}
              </span>
            ))}
          </div>
        )
      })()}

      {/* Coverage mini-bar (only if valid with behaviors) */}
      {!isInvalid && spec.behavior_count > 0 && (
        <div className="mt-1.5">
          <CoverageMiniBar covered={coveredCount} total={spec.behavior_count} />
        </div>
      )}

      {/* Error count (details in panel) */}
      {errors.length > 0 && (
        <div className="mt-1.5 flex items-center gap-1.5 text-xs text-red-400">
          <XCircle className="size-3 shrink-0" />
          <span>{errors.length} validation error{errors.length !== 1 ? "s" : ""}</span>
        </div>
      )}

      {/* Dep error count (details in panel) */}
      {depErrors.length > 0 && (
        <div className="mt-1.5 flex items-center gap-1.5 text-xs text-red-400">
          <XCircle className="size-3 shrink-0" />
          <span>{depErrors.length} dependency error{depErrors.length !== 1 ? "s" : ""}</span>
        </div>
      )}

      {/* Uncovered behaviors warning */}
      {uncoveredBehaviors.length > 0 && (
        <div className="mt-1.5 flex items-start gap-1.5 text-xs text-amber-400">
          <AlertTriangle className="mt-0.5 size-3 shrink-0 text-amber-400" />
          <span>{uncoveredBehaviors.length} uncovered behaviors</span>
        </div>
      )}
    </button>
  )
}

interface SpecCardGridProps {
  specs: SpecInfo[]
  nfrs: NfrInfo[]
  onSelectSpec: (spec: SpecInfo) => void
  onSelectNfr?: (nfr: NfrInfo) => void
}

export function SpecCardGrid({ specs, nfrs, onSelectSpec, onSelectNfr }: SpecCardGridProps) {
  const [search, setSearch] = useState("")

  const filtered = specs.filter((s) =>
    s.name.toLowerCase().includes(search.toLowerCase())
  )

  return (
    <div>
      {/* Search bar */}
      <div className="relative mb-4">
        <Search className="absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search specs..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-9"
        />
      </div>

      {/* Card grid */}
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
        {filtered.map((spec) => (
          <SpecCard
            key={spec.path}
            spec={spec}
            nfrs={nfrs}
            onClick={() => onSelectSpec(spec)}
            onClickNfr={onSelectNfr}
          />
        ))}
      </div>

      {filtered.length === 0 && specs.length > 0 && (
        <p className="mt-8 text-center text-sm text-muted-foreground">
          No specs match &quot;{search}&quot;
        </p>
      )}
    </div>
  )
}
