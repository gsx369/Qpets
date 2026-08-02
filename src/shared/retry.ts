export const RUNTIME_INITIALIZE_RETRY_DELAYS_MS = [120, 400, 900] as const
export const IMAGE_LOAD_RETRY_DELAYS_MS = [120, 400, 900] as const

export function retryDelayAfterFailure(
  delays: readonly number[],
  failedAttemptIndex: number,
): number | undefined {
  return failedAttemptIndex >= 0 ? delays[failedAttemptIndex] : undefined
}

export function waitForRetry(delayMs: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}
