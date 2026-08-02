export type SettingsSection = 'library' | 'pet' | 'about'

export interface PetCharacter {
  id: string
  name: string
  /** Optional local asset URL or data URL. */
  cover?: string
  builtIn: boolean
  active?: boolean
  description: string
  metadataCustomized: boolean
  modelType?: '动态' | '静态'
}

export interface CharacterMetadataDraft {
  displayName: string
  description: string
}

export interface PetInteractionSettings {
  alwaysOnTop: boolean
  clickThrough: boolean
  followCursor: boolean
  showBubble: boolean
  startWithWindows: boolean
  scale: number
  idleIntervalSeconds: number
}

export interface SettingsAboutInfo {
  appName: string
  version: string
  description?: string
  homepage?: string
}
