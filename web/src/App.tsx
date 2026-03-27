import { useState, useCallback } from "react"
import { useProjectState } from "@/hooks/useProjectState"
import { MetricsBar } from "@/components/MetricsBar"
import { SpecCardGrid } from "@/components/SpecCardGrid"
import { SpecSlidePanel } from "@/components/SpecSlidePanel"
import { InvalidTagsPanel } from "@/components/InvalidTagsPanel"
import type { SpecInfo } from "@/types"

function App() {
  const { state, loading, connected, lockLoading, lockSuccess, regenerateLock } =
    useProjectState()

  const [selectedSpec, setSelectedSpec] = useState<SpecInfo | null>(null)
  const [showInvalidTags, setShowInvalidTags] = useState(false)

  const handleSelectSpec = useCallback((spec: SpecInfo) => {
    setSelectedSpec(spec)
  }, [])

  return (
    <div className="dark min-h-screen bg-background text-foreground">
      <MetricsBar
        state={state}
        connected={connected}
        loading={loading}
        lockLoading={lockLoading}
        lockSuccess={lockSuccess}
        onRegenerateLock={regenerateLock}
        invalidTagCount={state?.invalid_tags.length ?? 0}
        onShowInvalidTags={() => setShowInvalidTags(true)}
      />

      <main className="mx-auto max-w-6xl px-6 py-4">
        <SpecCardGrid
          specs={state?.specs ?? []}
          onSelectSpec={handleSelectSpec}
        />
      </main>

      <SpecSlidePanel
        spec={selectedSpec}
        isOpen={selectedSpec !== null}
        onClose={() => setSelectedSpec(null)}
      />

      <InvalidTagsPanel
        tags={state?.invalid_tags ?? []}
        isOpen={showInvalidTags}
        onClose={() => setShowInvalidTags(false)}
      />
    </div>
  )
}

export default App
