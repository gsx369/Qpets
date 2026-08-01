import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { MOCK_STATE } from './mock-state'

const tauri = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  open: vi.fn(),
  openUrl: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => path,
  invoke: tauri.invoke,
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: tauri.listen }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: tauri.open }))
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: tauri.openUrl }))

function appState() {
  return JSON.parse(JSON.stringify(MOCK_STATE))
}

let disposeRuntime: (() => void) | undefined

beforeEach(() => {
  vi.useFakeTimers()
  vi.resetModules()
  vi.stubGlobal('window', { __TAURI_INTERNALS__: {} })
  tauri.invoke.mockReset()
  tauri.listen.mockReset()
  tauri.open.mockReset()
  tauri.openUrl.mockReset()
  disposeRuntime = undefined
})

afterEach(() => {
  disposeRuntime?.()
  vi.unstubAllGlobals()
  vi.useRealTimers()
})

describe('app runtime initialization', () => {
  it('retries a transient snapshot failure and then becomes ready', async () => {
    const unlisten = vi.fn()
    tauri.listen.mockResolvedValue(unlisten)
    tauri.invoke.mockRejectedValueOnce(new Error('store not ready')).mockResolvedValueOnce(appState())
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()
    disposeRuntime = runtime.dispose

    const initialization = runtime.initialize()
    await vi.advanceTimersByTimeAsync(120)
    await initialization

    expect(tauri.invoke).toHaveBeenCalledTimes(2)
    expect(unlisten).toHaveBeenCalledTimes(1)
    expect(runtime.ready.value).toBe(true)
    expect(runtime.error.value).toBeUndefined()
  })

  it('uses a state snapshot when event subscription is temporarily unavailable', async () => {
    tauri.listen.mockRejectedValue(new Error('events unavailable'))
    tauri.invoke.mockResolvedValue(appState())
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()
    disposeRuntime = runtime.dispose

    await runtime.initialize()

    expect(tauri.invoke).toHaveBeenCalledTimes(1)
    expect(runtime.ready.value).toBe(true)
    expect(runtime.error.value).toBeUndefined()
  })

  it('restores the state listener in the background after snapshot fallback', async () => {
    const unlisten = vi.fn()
    tauri.listen.mockRejectedValueOnce(new Error('events unavailable')).mockResolvedValueOnce(unlisten)
    tauri.invoke.mockResolvedValue(appState())
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()
    disposeRuntime = runtime.dispose

    await runtime.initialize()
    await vi.advanceTimersByTimeAsync(120)

    expect(tauri.listen).toHaveBeenCalledTimes(2)
    expect(tauri.invoke).toHaveBeenCalledTimes(2)
    expect(runtime.ready.value).toBe(true)
    expect(runtime.error.value).toBeUndefined()
  })

  it('cancels a pending retry when the owning view is disposed', async () => {
    tauri.listen.mockResolvedValue(vi.fn())
    tauri.invoke.mockRejectedValue(new Error('store not ready'))
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()
    disposeRuntime = runtime.dispose

    const initialization = runtime.initialize()
    await vi.advanceTimersByTimeAsync(0)
    expect(tauri.invoke).toHaveBeenCalledTimes(1)

    runtime.dispose()
    disposeRuntime = undefined
    await vi.advanceTimersByTimeAsync(2_000)
    await initialization

    expect(tauri.invoke).toHaveBeenCalledTimes(1)
    expect(runtime.ready.value).toBe(false)
  })

  it('reports a terminal initialization failure and can recover on a later retry', async () => {
    const unlisten = vi.fn()
    tauri.listen.mockResolvedValue(unlisten)
    tauri.invoke.mockRejectedValue(new Error('store not ready'))
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()
    disposeRuntime = runtime.dispose

    const failedInitialization = runtime.initialize()
    await vi.advanceTimersByTimeAsync(1_420)
    await failedInitialization

    expect(tauri.invoke).toHaveBeenCalledTimes(4)
    expect(runtime.ready.value).toBe(false)
    expect(runtime.error.value).toBe('应用初始化失败：store not ready')

    tauri.invoke.mockResolvedValue(appState())
    await runtime.initialize()

    expect(tauri.invoke).toHaveBeenCalledTimes(5)
    expect(runtime.ready.value).toBe(true)
    expect(runtime.error.value).toBeUndefined()
  })

  it('unsubscribes only once when disposed during a pending snapshot', async () => {
    const unlisten = vi.fn()
    let rejectSnapshot: ((reason: Error) => void) | undefined
    tauri.listen.mockResolvedValue(unlisten)
    tauri.invoke.mockImplementation(() => new Promise((_resolve, reject) => {
      rejectSnapshot = reject
    }))
    const { useAppRuntime } = await import('./runtime')
    const runtime = useAppRuntime()

    const initialization = runtime.initialize()
    await vi.advanceTimersByTimeAsync(0)
    runtime.dispose()
    rejectSnapshot?.(new Error('cancelled'))
    await initialization

    expect(unlisten).toHaveBeenCalledTimes(1)
    expect(runtime.ready.value).toBe(false)
  })
})
