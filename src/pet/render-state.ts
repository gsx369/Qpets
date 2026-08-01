import type { RenderType } from '../shared/types'

export type PetRenderMode = 'waiting' | 'atlas' | 'static'

export function choosePetRenderMode(
  ready: boolean,
  renderType: RenderType | undefined,
  primarySource: string,
  primaryLoadFailed: boolean,
  fallbackSource: string,
): PetRenderMode {
  if (!ready || !renderType) return 'waiting'
  if (renderType === 'sprite-atlas-v2' && primarySource && !primaryLoadFailed) return 'atlas'

  const staticSource = primaryLoadFailed ? fallbackSource : primarySource
  if (staticSource && (renderType === 'static-image-v1' || primaryLoadFailed)) return 'static'
  return 'waiting'
}
