import type { PetAction } from '../shared/types'

export const CELL_WIDTH = 192
export const CELL_HEIGHT = 208

interface AnimationDefinition {
  row: number
  durations: readonly number[]
  loop: boolean
}

const STANDARD_DURATIONS = {
  idle: [280, 110, 110, 140, 140, 320],
  directional: [120, 120, 120, 120, 120, 120, 120, 220],
  waving: [140, 140, 140, 280],
  jumping: [140, 140, 140, 140, 280],
  failed: [140, 140, 140, 140, 140, 140, 140, 240],
  waiting: [150, 150, 150, 150, 150, 260],
  running: [120, 120, 120, 120, 120, 220],
  review: [150, 150, 150, 150, 150, 280],
} as const

export const ANIMATIONS: Record<Exclude<PetAction, 'look'>, AnimationDefinition> = {
  idle: { row: 0, durations: STANDARD_DURATIONS.idle, loop: true },
  'running-right': { row: 1, durations: STANDARD_DURATIONS.directional, loop: true },
  'running-left': { row: 2, durations: STANDARD_DURATIONS.directional, loop: true },
  waving: { row: 3, durations: STANDARD_DURATIONS.waving, loop: false },
  jumping: { row: 4, durations: STANDARD_DURATIONS.jumping, loop: false },
  failed: { row: 5, durations: STANDARD_DURATIONS.failed, loop: false },
  waiting: { row: 6, durations: STANDARD_DURATIONS.waiting, loop: false },
  running: { row: 7, durations: STANDARD_DURATIONS.running, loop: true },
  review: { row: 8, durations: STANDARD_DURATIONS.review, loop: false },
}

export interface AtlasFrame {
  row: number
  column: number
  complete: boolean
}

export function lookIndexToFrame(index: number): AtlasFrame {
  const normalized = ((Math.round(index) % 16) + 16) % 16
  return normalized < 8
    ? { row: 9, column: normalized, complete: false }
    : { row: 10, column: normalized - 8, complete: false }
}

export function vectorToLookIndex(dx: number, dy: number, deadzone = 28): number | undefined {
  if (Math.hypot(dx, dy) < deadzone) return undefined
  const degrees = (Math.atan2(dx, -dy) * 180) / Math.PI
  return Math.round(((degrees + 360) % 360) / 22.5) % 16
}

export function frameAt(action: Exclude<PetAction, 'look'>, elapsedMs: number): AtlasFrame {
  const definition = ANIMATIONS[action]
  const total = definition.durations.reduce((sum, duration) => sum + duration, 0)
  const complete = !definition.loop && elapsedMs >= total
  let position = definition.loop ? elapsedMs % total : Math.min(elapsedMs, total - 1)
  let column = 0

  for (let index = 0; index < definition.durations.length; index += 1) {
    if (position < definition.durations[index]) {
      column = index
      break
    }
    position -= definition.durations[index]
  }

  return { row: definition.row, column, complete }
}
