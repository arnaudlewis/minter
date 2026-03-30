import { Card, CardContent } from "@/components/ui/card"
import { Paintbrush } from "lucide-react"

export function EmptyDesignState() {
  return (
    <div
      data-testid="design-empty-state"
      className="flex h-full items-center justify-center"
    >
      <Card className="max-w-md border-dashed">
        <CardContent className="flex flex-col items-center gap-4 py-12 text-center">
          <div className="flex size-12 items-center justify-center rounded-xl bg-muted">
            <Paintbrush className="size-6 text-muted-foreground" />
          </div>
          <div className="space-y-1.5">
            <h3 className="text-lg font-semibold tracking-tight text-foreground">
              No design system generated yet
            </h3>
            <p className="text-sm text-muted-foreground">
              Use <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">generate_layout</code> to
              create a design system. The preview will appear here automatically.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
