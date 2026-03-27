import type { SpecInfo } from "@/types"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog"
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
} from "lucide-react"

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

interface SpecDetailDialogProps {
  spec: SpecInfo | null
  depErrors: string[]
  open: boolean
  onOpenChange: (open: boolean, event?: Event) => void
}

export function SpecDetailDialog({
  spec,
  depErrors,
  open,
  onOpenChange,
}: SpecDetailDialogProps) {
  if (!spec) return null

  const isInvalid =
    typeof spec.validation_status === "object" &&
    "Invalid" in spec.validation_status
  const errors = isInvalid
    ? (spec.validation_status as { Invalid: string[] }).Invalid
    : []

  const coveredCount = spec.behaviors.filter((b) => b.covered).length
  const totalBehaviors = spec.behavior_count
  const pct =
    totalBehaviors > 0 ? Math.round((coveredCount / totalBehaviors) * 100) : 0

  const statusIcon = isInvalid ? (
    <XCircle className="size-5 text-red-400" />
  ) : coveredCount === totalBehaviors && totalBehaviors > 0 ? (
    <CheckCircle2 className="size-5 text-emerald-400" />
  ) : (
    <AlertTriangle className="size-5 text-amber-400" />
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {statusIcon}
            <span className="font-mono">{spec.name}</span>
            <span className="text-sm font-normal text-muted-foreground">
              v{spec.version}
            </span>
          </DialogTitle>
          {!isInvalid && totalBehaviors > 0 && (
            <DialogDescription>
              {coveredCount}/{totalBehaviors} behaviors covered ({pct}%)
            </DialogDescription>
          )}
        </DialogHeader>

        <div className="max-h-[60vh] space-y-4 overflow-y-auto">
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

          {/* Behaviors list */}
          {spec.behaviors.length > 0 && (
            <div>
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                Behaviors
              </h3>
              <div className="space-y-1">
                {spec.behaviors.map((behavior) => (
                  <div
                    key={behavior.name}
                    className="flex items-center gap-2 rounded-md px-3 py-2 text-sm"
                  >
                    {behavior.covered ? (
                      <CheckCircle2 className="size-3.5 shrink-0 text-emerald-400" />
                    ) : (
                      <XCircle className="size-3.5 shrink-0 text-zinc-500" />
                    )}
                    <span className="font-mono text-xs">{behavior.name}</span>
                    <div className="ml-auto flex items-center gap-1">
                      {behavior.covered ? (
                        behavior.test_types.map((type) => (
                          <TestTypeBadge key={type} type={type} />
                        ))
                      ) : (
                        <span className="text-xs text-zinc-500">uncovered</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
