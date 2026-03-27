import { useEffect, useRef, useState, useCallback } from "react"
import type { ProjectState } from "@/types"

const RECONNECT_DELAY_MS = 2000
const MAX_RECONNECT_DELAY_MS = 30000

export function useWebSocket(onMessage: (state: ProjectState) => void) {
  const [connected, setConnected] = useState(false)
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const delayRef = useRef(RECONNECT_DELAY_MS)
  const onMessageRef = useRef(onMessage)
  onMessageRef.current = onMessage

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:"
    const ws = new WebSocket(`${protocol}//${window.location.host}/ws`)

    ws.onopen = () => {
      setConnected(true)
      delayRef.current = RECONNECT_DELAY_MS
    }

    ws.onmessage = (event) => {
      try {
        const state = JSON.parse(event.data as string) as ProjectState
        onMessageRef.current(state)
      } catch {
        // Ignore malformed messages
      }
    }

    ws.onclose = () => {
      setConnected(false)
      wsRef.current = null
      scheduleReconnect()
    }

    ws.onerror = () => {
      ws.close()
    }

    wsRef.current = ws
  }, [])

  const scheduleReconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    reconnectTimeoutRef.current = setTimeout(() => {
      connect()
      delayRef.current = Math.min(
        delayRef.current * 1.5,
        MAX_RECONNECT_DELAY_MS
      )
    }, delayRef.current)
  }, [connect])

  useEffect(() => {
    connect()
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      wsRef.current?.close()
    }
  }, [connect])

  return { connected }
}
