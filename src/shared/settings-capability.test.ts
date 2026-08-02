import { describe, expect, it } from 'vitest'

import settingsCapability from '../../src-tauri/capabilities/settings.json'

describe('settings capability', () => {
  it('allows only the configured GitHub project homepage', () => {
    expect(settingsCapability.permissions).toContainEqual({
      identifier: 'opener:allow-open-url',
      allow: [{ url: 'https://github.com/gsx369/Qpets' }],
    })
    expect(settingsCapability.permissions).not.toContain('opener:allow-default-urls')
  })
})
