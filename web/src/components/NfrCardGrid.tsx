import type { NfrInfo } from "@/types"
import { NfrStatusIcon } from "@/components/ui/NfrStatusIcon"

function NfrCard({
  nfr,
  onClick,
}: {
  nfr: NfrInfo
  onClick: () => void
}) {
  return (
    <button
      type="button"
      data-testid="nfr-card"
      className="cursor-pointer rounded-lg border border-border bg-card p-3 text-left transition-all hover:border-zinc-600 hover:shadow-md"
      onClick={onClick}
    >
      <div className="flex items-center gap-2">
        <NfrStatusIcon nfr={nfr} />
        <span className="truncate font-mono text-sm font-medium text-foreground">
          {nfr.category}
        </span>
        <span className="ml-auto shrink-0 font-mono text-xs text-muted-foreground">
          v{nfr.version}
        </span>
      </div>
      <div className="mt-1.5 text-xs text-muted-foreground">
        {nfr.constraint_count} constraint{nfr.constraint_count !== 1 ? "s" : ""}
      </div>
    </button>
  )
}

interface NfrCardGridProps {
  nfrs: NfrInfo[]
  onSelectNfr: (nfr: NfrInfo) => void
}

export function NfrCardGrid({ nfrs, onSelectNfr }: NfrCardGridProps) {
  if (nfrs.length === 0) return null

  return (
    <div>
      <div className="mb-3 flex items-center gap-3">
        <div className="h-px flex-1 bg-border" />
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          NFR Constraints
        </span>
        <div className="h-px flex-1 bg-border" />
      </div>

      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
        {nfrs.map((nfr) => (
          <NfrCard
            key={nfr.path}
            nfr={nfr}
            onClick={() => onSelectNfr(nfr)}
          />
        ))}
      </div>
    </div>
  )
}
