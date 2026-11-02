// editor/main.tsx
// Root component composing all editor panels into the Mantine AppShell.

import React, { useState, useCallback } from 'react'
import { createRoot } from 'react-dom/client'
import {
  MantineProvider,
  AppShell,
  Header,
  Navbar,
  Text,
  Divider,
} from '@mantine/core'

import { ColorPickerEditor } from './components/ColorPicker'
import { Toolbar, type ToolMode } from './components/Toolbar'
import { RosterPanel } from './components/RosterPanel'
import { ConnectionPanel, type ConnStatus } from './components/ConnectionPanel'
import { CommStatus, type CommEvent } from './components/CommStatus'
import type { PeerInfo } from './types'

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

  const handleConnect = useCallback(() => {
    setConnStatus('connecting')
    addEvent('out', `Connecting to ${SERVER_ADDR} as ${PEER_NAME}`)
    // In a full implementation, open the WebSocket here
    // For the MVP demo, simulate a successful connection:
    setTimeout(() => {
      setConnStatus('connected')
      setClientId('demo-1')
      setRoster([
        {
          id: 'demo-1',
          name: PEER_NAME,
          color_rgb: PEER_COLOR,
          last_active_ms: Date.now(),
        },
      ])
      addEvent('in', 'Welcome received — connected as demo-1')
    }, 500)
  }, [addEvent])

  const handleDisconnect = useCallback(() => {
    setConnStatus('disconnected')
    addEvent('out', 'Disconnected')
  }, [addEvent])

  return (
    <MantineProvider>
      <AppShell
        header={{ height: 56 }}
        navbar={{ width: 280, breakpoint: 'md' }}
      >
        <Header
          height={56}
          p="md"
          style={{ background: '#1a1a2e', color: 'white' }}
        >
          <Text size="lg" fw={700} c="white">
            Warren Game Editor
          </Text>
        </Header>

        <Navbar p="md">
          <Toolbar mode={toolMode} onModeChange={setToolMode} />
          <Divider my="sm" />
          <ColorPickerEditor onSelect={setBrushColor} currentColor={brushColor} />
          <Divider my="sm" />
          <ConnectionPanel
            status={connStatus}
            serverAddr={SERVER_ADDR}
            onConnect={handleConnect}
            onDisconnect={handleDisconnect}
          />
          <Divider my="sm" />
          <RosterPanel peers={roster} clientId={clientId} />
        </Navbar>

        <AppShell.Main>
          <div style={{ padding: 'md' }}>
            <Text c="dimmed">
              3D viewport runs in a separate Bevy window. This panel is the
              Mantine UI overlay for colour, tools, peer roster, and connection
              status.
            </Text>
            <div style={{ marginTop: 24 }}>
              <CommStatus events={events} />
            </div>
          </div>
        </AppShell.Main>
      </AppShell>
    </MantineProvider>
  )
}

const root = createRoot(document.getElementById('root')!)
root.render(<App />)
