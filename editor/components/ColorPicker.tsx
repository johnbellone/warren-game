// editor/components/ColorPicker.tsx
// Mantine ColorPicker for selecting the voxel brush colour.
// Outputs RGB 0..1 as [r, g, b].

import { Group, Text } from '@mantine/core'
import { useState, useCallback } from 'react'

interface Props {
  onSelect: (rgb: [number, number, number]) => void
  currentColor?: [number, number, number]
}

const presets: { label: string; rgb: [number, number, number] }[] = [
  { label: 'Red',    rgb: [1.0, 0.0, 0.0] },
  { label: 'Green',  rgb: [0.0, 1.0, 0.0] },
  { label: 'Blue',   rgb: [0.0, 0.0, 1.0] },
  { label: 'Yellow', rgb: [1.0, 1.0, 0.0] },
  { label: 'Cyan',   rgb: [0.0, 1.0, 1.0] },
  { label: 'Magenta',rgb: [1.0, 0.0, 1.0] },
  { label: 'Black',  rgb: [0.0, 0.0, 0.0] },
  { label: 'White',  rgb: [1.0, 1.0, 1.0] },
]

function rgbToHex(rgb: [number, number, number]): string {
  const toHex = (v: number) =>
    Math.round(v * 255).toString(16).padStart(2, '0')
  return `#${toHex(rgb[0])}${toHex(rgb[1])}${toHex(rgb[2])}`
}

export function ColorPickerEditor({ onSelect, currentColor = [1, 0, 0] }: Props) {
  const [color, setColor] = useState<string>(rgbToHex(currentColor))

  const handlePreset = useCallback(
    (rgb: [number, number, number]) => {
      const hex = rgbToHex(rgb)
      setColor(hex)
      onSelect(rgb)
    },
    [onSelect]
  )

  return (
    <div>
      <Text fw={700} mb="xs">
        Brush Color
      </Text>
      <Group gap="xs" wrap="wrap">
        {presets.map((p) => {
          const hex = rgbToHex(p.rgb)
          return (
            <button
              key={p.label}
              onClick={() => handlePreset(p.rgb)}
              style={{
                width: 28,
                height: 28,
                borderRadius: 4,
                border: `2px solid ${color === hex ? '#333' : 'transparent'}`,
                backgroundColor: hex,
                cursor: 'pointer',
                padding: 0,
                title: p.label,
              }}
            />
          )
        })}
      </Group>
      <Text size="xs" c="dimmed" mt="xs">
        {color}
      </Text>
    </div>
  )
}
