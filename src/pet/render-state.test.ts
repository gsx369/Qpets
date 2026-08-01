import { describe, expect, it } from 'vitest'

import { choosePetRenderMode } from './render-state'

describe('pet render state', () => {
  it('never renders mock text while runtime state or an atlas source is missing', () => {
    expect(choosePetRenderMode(false, 'sprite-atlas-v2', '', false, '')).toBe('waiting')
    expect(choosePetRenderMode(true, 'sprite-atlas-v2', '', false, '')).toBe('waiting')
  })

  it('uses the atlas when ready and the real thumbnail only after a hard failure', () => {
    expect(choosePetRenderMode(true, 'sprite-atlas-v2', 'asset://atlas', false, 'asset://thumb')).toBe('atlas')
    expect(choosePetRenderMode(true, 'sprite-atlas-v2', 'asset://atlas', true, 'asset://thumb')).toBe('static')
    expect(choosePetRenderMode(true, 'sprite-atlas-v2', 'asset://atlas', true, '')).toBe('waiting')
  })

  it('renders static character images without routing through an atlas', () => {
    expect(choosePetRenderMode(true, 'static-image-v1', 'asset://character', false, 'asset://thumb')).toBe('static')
  })
})
