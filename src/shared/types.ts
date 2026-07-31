export type RenderType = 'sprite-atlas-v2' | 'static-image-v1'
export type PetSource = 'builtin' | 'user'
export type PetAction =
  | 'idle'
  | 'running-right'
  | 'running-left'
  | 'waving'
  | 'jumping'
  | 'failed'
  | 'waiting'
  | 'running'
  | 'review'
  | 'look'

export interface AppSettings {
  alwaysOnTop: boolean
  clickThrough: boolean
  followCursor: boolean
  showBubble: boolean
  startWithWindows: boolean
  /** Retained for persisted/backend schema compatibility; no frontend control exists until audio playback is added. */
  volume: number
  scale: number
  idleIntervalSeconds: number
}

export type DialogueMap = Record<string, string[]>

export interface PetDescriptor {
  id: string
  displayName: string
  description: string
  metadataCustomized: boolean
  renderType: RenderType
  source: PetSource
  deletable: boolean
  assetPath: string
  thumbnailPath: string
  dialogues: DialogueMap
}

export interface PetMetadataDraft {
  displayName: string
  description: string
}

export interface AppState {
  settings: AppSettings
  selectedPetId: string
  pets: PetDescriptor[]
}

export const DEFAULT_PET_ID = 'qpet-z1-sunny-brim'

export const DEFAULT_SETTINGS: AppSettings = {
  alwaysOnTop: true,
  clickThrough: false,
  followCursor: true,
  showBubble: true,
  startWithWindows: false,
  volume: 0.55,
  scale: 1,
  idleIntervalSeconds: 30,
}
