// editor/main.tsx
// Root component composing all editor panels into the Mantine AppShell.
// Wires Tauri IPC commands when running inside a Tauri webview.

import '@mantine/core/styles.css'

import { useState, useCallback, useEffect } from 'react'
import { createRoot } from 'react-dom/client'
import {
  MantineProvider,
  AppShell,
  Text,
  Space,
} from '@mantine/core'

import { ColorPickerEditor } from './components/ColorPicker'
import { Toolbar, type ToolMode } from './components/Toolbar'
import { RosterPanel } from './components/RosterPanel'
import { ConnectionPanel, type ConnStatus } from './components/ConnectionPanel'
import { CommStatus, type CommEvent } from './components/CommStatus'
import type { PeerInfo } from './types'

// ── Tauri IPC helpers ──────────────────────────────────────────────────────────
// When running inside a Tauri webview, invoke is available. In a plain browser
// (dev without Tauri), fall back to local state so the UI still works.

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    // Not in a Tauri context — return undefined, caller should use local fallback
    return undefined
  }
}

// ── Constants ────────────────────────────────────────────────────────────────

const SERVER_ADDR = 'ws://127.0.0.1:8080'
const PEER_NAME = 'Player1'
const PEER_COLOR: [number, number, number] = [1.0, 0.0, 0.0]

function App() {
  const [brushColor, setBrushColor] = useState<[number, number, number]>([1.0, 0.0, 0.0])
  const [toolMode, setToolMode] = useState<ToolMode>('place')
  const [connStatus, setConnStatus] = useState<ConnStatus>('disconnected')
  const [roster, setRoster] = useState<PeerInfo[]>([])
  const [clientId, setClientId] = useState<string>('')
  const [events, setEvents] = useState<CommEvent[]>([])

  const addEvent = useCallback((dir: 'in' | 'out', summary: string) => {
    setEvents((prev) => [...prev, { time: Date.now(), dir, summary }])
  }, [])

  // ── Tauri IPC handlers ─────────────────────────────────────────────────

  const handleConnect = useCallback(async () => {
    setConnStatus('connecting')
    addEvent('out', `start_server → ${SERVER_ADDR}`)
    const result = await tauriInvoke<string>('start_server')
    if (result !== undefined) {
      addEvent('in', `server: ${result}`)
    }
    setConnStatus('connected')
    setClientId('demo-1')
    setRoster([
      { id: 'demo-1', name: PEER_NAME, color_rgb: PEER_COLOR, last_active_ms: Date.now() },
    ])
  }, [addEvent])

  const handleDisconnect = useCallback(async () => {
    addEvent('out', 'stop_server')
    const result = await tauriInvoke<string>('stop_server')
    if (result !== undefined) {
      addEvent('in', `server: ${result}`)
    }
    setConnStatus('disconnected')
  }, [addEvent])

  const handleColorSelect = useCallback((rgb: [number, number, number]) => {
    setBrushColor(rgb)
    tauriInvoke('set_color', { r: rgb[0], g: rgb[1], b: rgb[2] }).then((result) => {
      addEvent('out', `set_color → ${result ?? rgb.join(',')}`)
    })
  }, [addEvent])

  const handleToolChange = useCallback(async (mode: ToolMode) => {
    setToolMode(mode)
    const result = await tauriInvoke<string>('set_tool', { tool: mode })
    addEvent('out', `set_tool → ${result ?? mode}`)
  }, [addEvent])

  // Start engine on mount if Tauri is available
  useEffect(() => {
    tauriInvoke<string>('start_engine').then((result) => {
      if (result) addEvent('in', `engine: ${result}`)
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <MantineProvider>
      <AppShell
        header={{ height: 56 }}
        navbar={{ width: 280, breakpoint: 'md' }}
      >
        <AppShell.Header p="md" style={{ background: '#1a1a2e' }}>
          <Text size="lg" fw={700} c="white">
            Warren Game Editor
          </Text>
        </AppShell.Header>

        <AppShell.Navbar p="md">
          <Toolbar mode={toolMode} onModeChange={handleToolChange} />
          <Space h="sm" />
          <ColorPickerEditor onSelect={handleColorSelect} currentColor={brushColor} />
          <Space h="sm" />
          <ConnectionPanel
            status={connStatus}
            serverAddr={SERVER_ADDR}
            onConnect={handleConnect}
            onDisconnect={handleDisconnect}
          />
          <Space h="sm" />
          <RosterPanel peers={roster} clientId={clientId} />
        </AppShell.Navbar>

        <AppShell.Main p="md">
          <Text c="dimmed">
            3D viewport runs in a separate Bevy window. This panel is the
            Mantine UI overlay for colour, tools, peer roster, and connection
            status.
          </Text>
          <Space h="md" />
          <CommStatus events={events} />
        </AppShell.Main>
      </AppShell>
    </MantineProvider>
  )
}

const root = createRoot(document.getElementById('root')!)
root.render(<App />)
