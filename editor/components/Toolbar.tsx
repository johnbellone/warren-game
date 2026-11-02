// editor/components/Toolbar.tsx
// Place / Erase mode selector for the voxel editor.

import { ActionIcon, Group, Text, Tooltip } from '@mantine/core'

export type ToolMode = 'place' | 'erase'

interface Props {
  mode: ToolMode
  onModeChange: (mode: ToolMode) => void
}

export function Toolbar({ mode, onModeChange }: Props) {
  return (
    <div>
      <Text fw={700} mb="xs">
        Tools
      </Text>
      <Group gap="sm">
        <Tooltip label="Place Voxel">
          <ActionIcon
            size="xl"
            color={mode === 'place' ? 'blue' : 'gray'}
            variant={mode === 'place' ? 'filled' : 'subtle'}
            onClick={() => onModeChange('place')}
          >
            {mode === 'place' ? '◼' : '▪'}
          </ActionIcon>
        </Tooltip>
        <Tooltip label="Erase Voxel">
          <ActionIcon
            size="xl"
            color={mode === 'erase' ? 'red' : 'gray'}
            variant={mode === 'erase' ? 'filled' : 'subtle'}
            onClick={() => onModeChange('erase')}
          >
            {mode === 'erase' ? '✗' : '○'}
          </ActionIcon>
        </Tooltip>
      </Group>
      <Text size="xs" c="dimmed" mt="xs">
        Current: {mode === 'place' ? 'Place Voxel' : 'Erase Voxel'}
      </Text>
    </div>
  )
}
