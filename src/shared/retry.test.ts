import { describe, expect, it } from 'vitest'

import {
  IMAGE_LOAD_RETRY_DELAYS_MS,
  RUNTIME_INITIALIZE_RETRY_DELAYS_MS,
  retryDelayAfterFailure,
} from './retry'

describe('bounded retries', () => {
  it('returns each backoff delay and stops after the configured limit', () => {
    expect(retryDelayAfterFailure(RUNTIME_INITIALIZE_RETRY_DELAYS_MS, 0)).toBe(120)
    expect(retryDelayAfterFailure(RUNTIME_INITIALIZE_RETRY_DELAYS_MS, 2)).toBe(900)
    expect(retryDelayAfterFailure(RUNTIME_INITIALIZE_RETRY_DELAYS_MS, 3)).toBeUndefined()
    expect(retryDelayAfterFailure(IMAGE_LOAD_RETRY_DELAYS_MS, -1)).toBeUndefined()
  })
})
