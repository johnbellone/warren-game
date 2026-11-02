// editor/types.ts
// Shared types for the frontend editor.

export interface Voxel {
  position: [number, number, number]
  color: [number, number, number]
  asset_type: string
}

export interface PeerInfo {
  id: string
  name: string
  color_rgb: [number, number, number]
  last_active_ms: number
}

export interface ClientMsg {
  type: 'Join' | 'VoxelEdit' | 'Heartbeat' | 'Leave'
  name?: string
  color_rgb?: [number, number, number]
  position?: [number, number, number]
  color?: [number, number, number]
  erase?: boolean
}

export interface ServerMsg {
  type: 'Welcome' | 'Roster' | 'VoxelEdit' | 'HeartbeatAck' | 'Error'
  client_id?: string
  snapshot?: Voxel[]
  roster?: PeerInfo[]
  position?: [number, number, number]
  color?: [number, number, number]
  erase?: boolean
  message?: string
}

export interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'error'
  clientId: string
  lastError?: string
}
