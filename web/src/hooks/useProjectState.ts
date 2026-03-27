import { useState, useCallback, useEffect, useRef } from "react"
import type { ProjectState } from "@/types"
import { useWebSocket } from "@/hooks/useWebSocket"

export function useProjectState() {
  const [state, setState] = useState<ProjectState | null>(null)
  const [loading, setLoading] = useState(true)
  const [lockLoading, setLockLoading] = useState(false)
  const [lockSuccess, setLockSuccess] = useState(false)

  const fetchedRef = useRef(false)

  const handleWsMessage = useCallback((wsState: ProjectState) => {
    setState(wsState)
    setLoading(false)
  }, [])

  const { connected } = useWebSocket(handleWsMessage)

  // Fetch initial state via REST and on reconnect
  const fetchState = useCallback(async () => {
    try {
      const res = await fetch("/api/state")
      if (res.ok) {
        const data = (await res.json()) as ProjectState
        setState(data)
      }
    } catch {
      // Will retry on reconnect
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (connected && !fetchedRef.current) {
      fetchedRef.current = true
      fetchState()
    }
    if (connected && fetchedRef.current) {
      // Refresh on reconnect
      fetchState()
    }
    if (!connected) {
      fetchedRef.current = false
    }
  }, [connected, fetchState])

  // Initial fetch regardless of WS
  useEffect(() => {
    fetchState()
  }, [fetchState])

  const regenerateLock = useCallback(async () => {
    setLockLoading(true)
    try {
      const res = await fetch("/api/action/lock", { method: "POST" })
      if (res.ok) {
        await fetchState()
        setLockSuccess(true)
        setTimeout(() => setLockSuccess(false), 2000)
      }
    } catch {
      // Lock failed silently
    } finally {
      setLockLoading(false)
    }
  }, [fetchState])

  return {
    state,
    loading,
    connected,
    lockLoading,
    lockSuccess,
    regenerateLock,
  }
}
