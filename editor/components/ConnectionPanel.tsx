// editor/components/ConnectionPanel.tsx
// Shows connection status, server address, and connect/disconnect controls.

import { Button, Group, Text, Badge } from '@mantine/core'

export type ConnStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

interface Props {
  status: ConnStatus
  serverAddr: string
  onConnect: () => void
  onDisconnect: () => void
}

const badgeColorMap: Record<ConnStatus, string> = {
  disconnected: 'gray',
  connecting: 'yellow',
  connected: 'green',
  error: 'red',
}

export function ConnectionPanel({ status, serverAddr, onConnect, onDisconnect }: Props) {
  return (
    <div>
      <Text fw={700} mb="xs">
        Connection
      </Text>
      <Group gap="xs" mb="xs">
        <Badge color={badgeColorMap[status]}>{status}</Badge>
        <Text size="sm" c="dimmed">
          {serverAddr}
        </Text>
      </Group>
      <Group gap="xs">
        {status !== 'connected' ? (
          <Button size="xs" onClick={onConnect} disabled={status === 'connecting'}>
            {status === 'connecting' ? 'Connecting...' : 'Connect'}
          </Button>
        ) : (
          <Button size="xs" color="red" onClick={onDisconnect}>
            Disconnect
          </Button>
        )}
      </Group>
    </div>
  )
}
