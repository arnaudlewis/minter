import { useDesignState } from "@/hooks/useDesignState"
import { DashboardPrototype } from "@/components/design/DashboardPrototype"
import { EmptyDesignState } from "@/components/design/EmptyDesignState"
import { Loader2 } from "lucide-react"

export function DesignPage() {
  const { designSystem, loading, error } = useDesignState()

  if (loading) {
    return (
      <div
        data-testid="design-loading"
        className="flex h-full items-center justify-center"
      >
        <div className="flex items-center gap-2 text-muted-foreground">
          <Loader2 className="size-5 animate-spin" />
          <span className="text-sm">Loading design system...</span>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div
        data-testid="design-error"
        className="flex h-full items-center justify-center"
      >
        <p className="text-sm text-destructive">{error}</p>
      </div>
    )
  }

  if (!designSystem) {
    return <EmptyDesignState />
  }

  return (
    <div data-testid="design-page" className="h-full">
      <DashboardPrototype design={designSystem} />
    </div>
  )
}
