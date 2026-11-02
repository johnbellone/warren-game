// editor/components/CommStatus.tsx
// Display of the latest communication events / log messages.

import { Text, ScrollArea, Group, Badge } from '@mantine/core'

export interface CommEvent {
  time: number
  dir: 'in' | 'out'
  summary: string
}

interface Props {
  events: CommEvent[]
}

const maxLines = 20

export function CommStatus({ events }: Props) {
  const recent = events.slice(-maxLines).reverse()

  return (
    <div>
      <Group mb="xs">
        <Text fw={700}>Comm Status</Text>
        <Badge size="xs">{events.length}</Badge>
      </Group>
      <ScrollArea
        style={{ height: 120, width: '100%' }}
        type="auto"
      >
        {recent.length === 0 ? (
          <Text c="dimmed" size="sm">
            No events yet
          </Text>
        ) : (
          recent.map((ev, i) => (
            <Group key={i} gap="xs">
              <Badge size="xs" color={ev.dir === 'in' ? 'green' : 'blue'}>
                {ev.dir === 'in' ? 'IN' : 'OUT'}
              </Badge>
              <Text size="xs">{new Date(ev.time).toLocaleTimeString()}</Text>
              <Text size="xs">{ev.summary}</Text>
            </Group>
          ))
        )}
      </ScrollArea>
    </div>
  )
}
