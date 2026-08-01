import { describe, expect, it } from 'vitest'

import tauriConfig from '../../src-tauri/tauri.conf.json'
import candiedHaw from '../../src-tauri/resources/pets/qpet-gu-candied-haw/pet.json'
import sunnyBrim from '../../src-tauri/resources/pets/qpet-z1-sunny-brim/pet.json'
import violetWhisper from '../../src-tauri/resources/pets/qpet-z2-violet-whisper/pet.json'
import { MOCK_STATE } from './mock-state'

describe('built-in pet defaults', () => {
  const manifests = [sunnyBrim, violetWhisper, candiedHaw]

  it('keeps browser defaults aligned with packaged manifests', () => {
    expect(MOCK_STATE.pets.map(pet => ({
      id: pet.id,
      displayName: pet.displayName,
      description: pet.description,
    }))).toEqual(manifests.map(manifest => ({
      id: manifest.id,
      displayName: manifest.displayName,
      description: manifest.description,
    })))
  })

  it('keeps the pet hidden until the Rust store is ready', () => {
    expect(tauriConfig.app.windows.find(window => window.label === 'pet')?.visible).toBe(false)
  })
})
