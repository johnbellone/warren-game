// editor/components/RosterPanel.tsx
// Displays the list of currently connected peers.

import { Group, Text, Badge } from '@mantine/core'
import type { PeerInfo } from '../types'

interface Props {
  peers: PeerInfo[]
  clientId: string
}

function peerColor(peer: PeerInfo): string {
  const [r, g, b] = peer.color_rgb
  const toHex = (v: number) =>
    Math.round(v * 255).toString(16).padStart(2, '0')
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`
}

export function RosterPanel({ peers, clientId }: Props) {
  return (
    <div>
      <Text fw={700} mb="xs">
        Peers ({peers.length})
      </Text>
      {peers.length === 0 ? (
        <Text c="dimmed" size="sm">
          No peers connected
        </Text>
      ) : (
        <Group gap="sm" wrap="wrap">
          {peers.map((peer) => (
            <Group key={peer.id} gap="xs">
              <span
                style={{
                  display: 'inline-block',
                  width: 20,
                  height: 20,
                  borderRadius: '50%',
                  backgroundColor: peerColor(peer),
                }}
              />
              <div>
                <Text size="sm" fw={600}>
                  {peer.name}
                  {peer.id === clientId ? ' (you)' : ''}
                </Text>
                <Badge size="xs" color={peer.id === clientId ? 'blue' : 'green'}>
                  active
                </Badge>
              </div>
            </Group>
          ))}
        </Group>
      )}
    </div>
  )
}
