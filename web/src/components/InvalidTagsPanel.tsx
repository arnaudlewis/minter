import type { InvalidTag } from "@/types"
import { ScrollArea } from "@/components/ui/scroll-area"
import { X, AlertCircle } from "lucide-react"
import { Button } from "@/components/ui/button"

interface InvalidTagsPanelProps {
  tags: InvalidTag[]
  isOpen: boolean
  onClose: () => void
}

export function InvalidTagsPanel({ tags, isOpen, onClose }: InvalidTagsPanelProps) {
  return (
    <>
      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50"
          onClick={onClose}
        />
      )}

      {/* Slide panel */}
      <div
        className={`fixed inset-y-0 right-0 z-50 w-[500px] transform border-l border-border bg-card shadow-2xl transition-transform duration-300 ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        {/* Header */}
        <div className="flex items-center gap-3 border-b border-border px-5 py-4">
          <AlertCircle className="size-5 text-red-400" />
          <h2 className="flex-1 text-base font-semibold">
            Invalid Tags ({tags.length})
          </h2>
          <Button
            variant="ghost"
            size="icon"
            className="cursor-pointer"
            onClick={onClose}
            aria-label="Close"
          >
            <X className="size-4" />
          </Button>
        </div>

        {/* Content */}
        <ScrollArea className="h-[calc(100vh-65px)]">
          <div className="space-y-1 p-4">
            <p className="mb-3 text-sm text-muted-foreground">
              These @minter tags in test files reference behaviors that don't exist in any spec.
            </p>
            {tags.map((tag, i) => (
              <div
                key={i}
                className="rounded-md bg-muted/30 px-3 py-2"
              >
                <div className="font-mono text-xs text-muted-foreground">
                  {tag.file}:{tag.line}
                </div>
                <div className="mt-0.5 text-sm text-red-400">
                  {tag.message}
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>
    </>
  )
}
