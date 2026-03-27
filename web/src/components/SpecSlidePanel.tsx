import { useState, useEffect, useCallback } from "react"
import type { SpecInfo, BehaviorInfo } from "@/types"
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  X,
  Search,
  Link,
  GitBranch,
  Info,
} from "lucide-react"
import { Input } from "@/components/ui/input"
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog"

const TEST_TYPE_COLORS: Record<string, string> = {
  unit: "bg-blue-500/20 text-blue-400",
  e2e: "bg-purple-500/20 text-purple-400",
  integration: "bg-emerald-500/20 text-emerald-400",
  benchmark: "bg-orange-500/20 text-orange-400",
}

function TestTypeBadge({ type }: { type: string }) {
  const colors = TEST_TYPE_COLORS[type] ?? "bg-zinc-500/20 text-zinc-400"
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${colors}`}>
      {type}
    </span>
  )
}

function CategoryBadge({ category }: { category: string }) {
  return (
    <span className="inline-flex items-center rounded-full bg-zinc-500/20 px-2 py-0.5 text-[10px] font-medium text-zinc-400">
      {category}
    </span>
  )
}

function StatusIcon({ spec }: { spec: SpecInfo }) {
  const isInvalid =
    typeof spec.validation_status === "object" &&
    "Invalid" in spec.validation_status
  if (isInvalid) {
    return <XCircle className="size-5 text-red-400" />
  }
  const coveredCount = spec.behaviors.filter((b) => b.covered).length
  if (coveredCount === spec.behavior_count && spec.behavior_count > 0) {
    return <CheckCircle2 className="size-5 text-emerald-400" />
  }
  return <AlertTriangle className="size-5 text-amber-400" />
}

function BehaviorDetail({ behavior }: { behavior: BehaviorInfo }) {
  return (
    <div
      data-testid="behavior-detail"
      className="ml-6 mt-1 space-y-1.5 rounded-md bg-muted/30 px-3 py-2"
    >
      <div className="flex items-center gap-2 text-xs">
        <span className="text-muted-foreground">Category:</span>
        <CategoryBadge category={behavior.category} />
      </div>
      <div className="flex items-center gap-2 text-xs">
        <span className="text-muted-foreground">Status:</span>
        <span className={behavior.covered ? "text-emerald-400" : "text-amber-400"}>
          {behavior.covered ? "covered" : "uncovered"}
        </span>
      </div>
      {behavior.covered && behavior.test_types.length > 0 && (
        <div className="flex items-center gap-2 text-xs">
          <span className="text-muted-foreground">Tests:</span>
          <div className="flex items-center gap-1">
            {behavior.test_types.map((type) => (
              <TestTypeBadge key={type} type={type} />
            ))}
          </div>
        </div>
      )}
      {behavior.nfr_refs.length > 0 && (
        <div className="flex items-center gap-2 text-xs">
          <span className="text-muted-foreground">NFRs:</span>
          <span className="text-muted-foreground">
            {behavior.nfr_refs.join(", ")}
          </span>
        </div>
      )}
    </div>
  )
}

function SpecInfoDialog({ spec }: { spec: SpecInfo }) {
  const hasDetails = spec.description || spec.motivation
  return (
    <Dialog>
      <DialogTrigger
        render={
          <button
            type="button"
            aria-label="Spec info"
            className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          />
        }
      >
        <Info className="size-4" />
      </DialogTrigger>
      <DialogContent className="z-[100]">
        <DialogHeader>
          <DialogTitle>{spec.title ?? spec.name}</DialogTitle>
          {hasDetails ? (
            <>
              {spec.description && (
                <div className="space-y-1">
                  <span className="text-sm font-medium text-foreground">Description</span>
                  <DialogDescription>{spec.description}</DialogDescription>
                </div>
              )}
              {spec.motivation && (
                <div className="space-y-1">
                  <span className="text-sm font-medium text-foreground">Motivation</span>
                  <DialogDescription>{spec.motivation}</DialogDescription>
                </div>
              )}
            </>
          ) : (
            <DialogDescription>
              Description and motivation are not yet available from the API.
            </DialogDescription>
          )}
        </DialogHeader>
      </DialogContent>
    </Dialog>
  )
}

interface SpecSlidePanelProps {
  spec: SpecInfo | null
  isOpen: boolean
  onClose: () => void
  onSelectNfrCategory?: (category: string) => void
}

export function SpecSlidePanel({ spec, isOpen, onClose, onSelectNfrCategory }: SpecSlidePanelProps) {
  const depErrors = spec?.dep_errors ?? []
  const [search, setSearch] = useState("")
  const [expandedBehavior, setExpandedBehavior] = useState<string | null>(null)

  // Reset state when spec changes
  useEffect(() => {
    setSearch("")
    setExpandedBehavior(null)
  }, [spec?.name])

  // Handle Escape key
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

  const isInvalid = spec &&
    typeof spec.validation_status === "object" &&
    "Invalid" in spec.validation_status
  const errors = isInvalid
    ? (spec.validation_status as { Invalid: string[] }).Invalid
    : []

  const filteredBehaviors = spec?.behaviors.filter((b) =>
    b.name.toLowerCase().includes(search.toLowerCase())
  ) ?? []

  const toggleBehavior = (name: string) => {
    setExpandedBehavior((prev) => (prev === name ? null : name))
  }

  return (
    <>
      {/* Backdrop overlay */}
      <div
        data-testid="slide-panel-overlay"
        className={`fixed inset-0 z-40 bg-black/50 transition-opacity duration-300 ${
          isOpen ? "opacity-100" : "pointer-events-none opacity-0"
        }`}
        onClick={onClose}
      />

      {/* Slide panel */}
      <div
        data-testid="slide-panel"
        className={`fixed inset-y-0 right-0 z-50 w-[600px] transform border-l border-border bg-card shadow-2xl transition-transform duration-300 ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        {spec && (
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
                <StatusIcon spec={spec} />
                <span className="font-mono text-sm font-medium text-foreground">
                  {spec.name}
                </span>
                <span className="text-sm text-muted-foreground">
                  v{spec.version}
                </span>
                <SpecInfoDialog spec={spec} />
              </div>
            </div>

            {/* Search */}
            <div className="border-b border-border px-4 py-2">
              <div className="relative">
                <Search className="absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search behaviors..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>

            {/* Content */}
            <div className="overflow-y-auto" style={{ height: "calc(100% - 112px)" }}>
              <div className="space-y-4 px-4 py-4">
                {/* Errors section */}
                {errors.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Validation Errors
                    </h3>
                    <div className="space-y-1">
                      {errors.map((err, i) => (
                        <div
                          key={i}
                          data-testid="error-message"
                          className="flex items-start gap-2 rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-400"
                        >
                          <XCircle className="mt-0.5 size-3 shrink-0" />
                          <span>{err}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Dependency errors */}
                {depErrors.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Dependency Errors
                    </h3>
                    <div className="space-y-1">
                      {depErrors.map((err, i) => (
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

                {/* NFR refs section */}
                {spec.nfr_refs.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      NFR References
                    </h3>
                    <div className="flex flex-wrap gap-1.5">
                      {spec.nfr_refs.map((ref) => {
                        const category = ref.split("#")[0]
                        return (
                          <span
                            key={ref}
                            className={`inline-flex items-center gap-1 rounded-full bg-zinc-500/10 px-2.5 py-1 text-xs text-muted-foreground ${onSelectNfrCategory ? "cursor-pointer transition-colors hover:bg-zinc-500/20 hover:text-foreground" : ""}`}
                            onClick={onSelectNfrCategory ? () => onSelectNfrCategory(category) : undefined}
                          >
                            <Link className="size-3" />
                            {ref}
                          </span>
                        )
                      })}
                    </div>
                  </div>
                )}

                {/* Dependencies section */}
                {spec.dependencies.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Dependencies
                    </h3>
                    <div className="space-y-1">
                      {spec.dependencies.map((dep) => (
                        <div
                          key={dep}
                          className="flex items-center gap-2 text-sm"
                        >
                          <GitBranch className="size-3 text-muted-foreground" />
                          <span className="font-mono text-xs text-foreground">
                            {dep}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Behaviors list */}
                {spec.behaviors.length > 0 && (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Behaviors ({filteredBehaviors.length})
                    </h3>
                    <div className="space-y-1">
                      {filteredBehaviors.map((behavior) => (
                        <div
                          key={behavior.name}
                          data-testid="behavior-row"
                          className="rounded-md px-3 py-2"
                        >
                          {/* Title: icon + name + covered/uncovered badge */}
                          <div className="flex items-center gap-2">
                            {behavior.covered ? (
                              <CheckCircle2 className="size-3.5 shrink-0 text-emerald-400" />
                            ) : (
                              <AlertTriangle className="size-3.5 shrink-0 text-amber-400" />
                            )}
                            <span className="min-w-0 flex-1 truncate font-mono text-[13px]">{behavior.name}</span>
                            {behavior.covered ? (
                              <span className="inline-flex shrink-0 items-center rounded-full bg-emerald-500/20 px-2 py-0.5 text-[10px] font-medium text-emerald-400">
                                covered
                              </span>
                            ) : (
                              <span className="inline-flex shrink-0 items-center rounded-full bg-amber-500/20 px-2 py-0.5 text-[10px] font-medium text-amber-400">
                                uncovered
                              </span>
                            )}
                          </div>
                          {/* Description */}
                          {behavior.description && (
                            <p className="ml-[22px] mt-0.5 text-[13px] leading-tight text-muted-foreground">{behavior.description}</p>
                          )}
                          {/* Detail section (always visible) */}
                          <div className="ml-[22px] mt-1.5 space-y-1 rounded-md border border-border/50 bg-muted/40 px-3 py-2 shadow-sm">
                            <div className="flex items-center gap-2 text-xs">
                              <span className="text-muted-foreground">Category:</span>
                              <CategoryBadge category={behavior.category} />
                            </div>
                            {behavior.covered && behavior.test_types.length > 0 && (
                              <div className="flex items-center gap-2 text-xs">
                                <span className="text-muted-foreground">Tests:</span>
                                <div className="flex items-center gap-1">
                                  {behavior.test_types.map((type) => (
                                    <TestTypeBadge key={type} type={type} />
                                  ))}
                                </div>
                              </div>
                            )}
                            {behavior.nfr_refs.length > 0 && (
                              <div className="flex items-center gap-2 text-xs">
                                <span className="text-muted-foreground">NFRs:</span>
                                <div className="flex flex-wrap gap-1">
                                  {behavior.nfr_refs.map((ref) => {
                                    const category = ref.split("#")[0]
                                    return (
                                      <span
                                        key={ref}
                                        className={`text-muted-foreground ${onSelectNfrCategory ? "cursor-pointer underline decoration-dotted hover:text-foreground" : ""}`}
                                        onClick={onSelectNfrCategory ? () => onSelectNfrCategory(category) : undefined}
                                      >
                                        {ref}
                                      </span>
                                    )
                                  })}
                                </div>
                              </div>
                            )}
                          </div>
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
