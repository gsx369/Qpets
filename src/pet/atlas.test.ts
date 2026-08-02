import { describe, expect, it } from 'vitest'

import { frameAt, lookIndexToFrame, vectorToLookIndex } from './atlas'

describe('atlas animation contract', () => {
  it('maps the 16 gaze directions across rows 9 and 10', () => {
    expect(lookIndexToFrame(0)).toMatchObject({ row: 9, column: 0 })
    expect(lookIndexToFrame(7)).toMatchObject({ row: 9, column: 7 })
    expect(lookIndexToFrame(8)).toMatchObject({ row: 10, column: 0 })
    expect(lookIndexToFrame(15)).toMatchObject({ row: 10, column: 7 })
  })

  it('uses screen-clockwise gaze semantics', () => {
    expect(vectorToLookIndex(0, -100)).toBe(0)
    expect(vectorToLookIndex(100, 0)).toBe(4)
    expect(vectorToLookIndex(0, 100)).toBe(8)
    expect(vectorToLookIndex(-100, 0)).toBe(12)
    expect(vectorToLookIndex(3, 4)).toBeUndefined()
  })

  it('loops idle while one-shot actions complete', () => {
    expect(frameAt('idle', 1_101).complete).toBe(false)
    expect(frameAt('idle', 1_101).column).toBe(0)
    expect(frameAt('waving', 699)).toMatchObject({ column: 3, complete: false })
    expect(frameAt('waving', 700)).toMatchObject({ column: 3, complete: true })
  })
})
