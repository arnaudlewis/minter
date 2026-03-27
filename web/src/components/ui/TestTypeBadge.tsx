const TEST_TYPE_COLORS: Record<string, string> = {
  unit: "bg-blue-500/20 text-blue-400",
  e2e: "bg-purple-500/20 text-purple-400",
  integration: "bg-emerald-500/20 text-emerald-400",
  benchmark: "bg-orange-500/20 text-orange-400",
}

export function TestTypeBadge({ type }: { type: string }) {
  const colors = TEST_TYPE_COLORS[type] ?? "bg-zinc-500/20 text-zinc-400"
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${colors}`}>
      {type}
    </span>
  )
}
