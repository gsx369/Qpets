import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, reactive, readonly, ref } from 'vue'

import { MOCK_STATE } from './mock-state'
import { RUNTIME_INITIALIZE_RETRY_DELAYS_MS, retryDelayAfterFailure, waitForRetry } from './retry'
import type { AppSettings, AppState, PetDescriptor, PetMetadataDraft } from './types'

function cloneMockState(): AppState {
  return JSON.parse(JSON.stringify(MOCK_STATE)) as AppState
}

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window
}

export function resolveAssetUrl(path: string): string {
  if (!path || path.startsWith('http:') || path.startsWith('https:') || path.startsWith('data:') || path.startsWith('blob:')) {
    return path
  }
  return isTauriRuntime() ? convertFileSrc(path) : path
}

const state = reactive<AppState>(cloneMockState())
const ready = ref(false)
const busy = ref(false)
const error = ref<string>()
let confirmedSettings: AppSettings = { ...state.settings }
let initializingPromise: Promise<void> | undefined
let initializationGeneration = 0
let stopStateListener: (() => void) | undefined
let mutationTail: Promise<void> = Promise.resolve()
let settingsTimer: ReturnType<typeof setTimeout> | undefined
let pendingSettings: AppSettings | undefined
let pendingSettingsBefore: AppSettings | undefined
let pendingSettingsResolvers: Array<() => void> = []
let settingsIntent = 0
let settledSettingsIntent = 0

function replaceState(next: AppState, preserveOptimisticSettings = false) {
  const optimisticSettings = preserveOptimisticSettings ? { ...state.settings } : undefined
  confirmedSettings = { ...next.settings }
  state.settings = { ...next.settings }
  state.selectedPetId = next.selectedPetId
  state.pets = next.pets.map((pet) => ({ ...pet, dialogues: { ...pet.dialogues } }))
  if (optimisticSettings) state.settings = optimisticSettings
}

function applyIncomingState(next: AppState) {
  replaceState(next, settledSettingsIntent < settingsIntent)
}

function makeIdempotent(action: () => void): () => void {
  let active = true
  return () => {
    if (!active) return
    active = false
    action()
  }
}

async function recoverStateListener(generation: number) {
  let failedAttemptIndex = 0
  while (generation === initializationGeneration && ready.value && !stopStateListener) {
    const delay = retryDelayAfterFailure(RUNTIME_INITIALIZE_RETRY_DELAYS_MS, failedAttemptIndex)
    if (delay === undefined) return
    failedAttemptIndex += 1
    await waitForRetry(delay)
    if (generation !== initializationGeneration || !ready.value || stopStateListener) return

    let unsubscribe: (() => void) | undefined
    try {
      unsubscribe = makeIdempotent(
        await listen<AppState>('qpets://state-changed', (event) => applyIncomingState(event.payload)),
      )
      if (generation !== initializationGeneration) {
        unsubscribe()
        return
      }
      const snapshot = await invoke<AppState>('get_app_state')
      if (generation !== initializationGeneration) {
        unsubscribe()
        return
      }
      stopStateListener = unsubscribe
      applyIncomingState(snapshot)
      return
    } catch {
      unsubscribe?.()
    }
  }
}

function messageFrom(errorValue: unknown): string {
  return errorValue instanceof Error ? errorValue.message : String(errorValue)
}

async function runMutation<T>(operation: () => Promise<T>): Promise<T | undefined> {
  busy.value = true
  error.value = undefined
  try {
    return await operation()
  } catch (cause) {
    error.value = messageFrom(cause)
    return undefined
  } finally {
    busy.value = false
  }
}

function enqueueMutation<T>(operation: () => Promise<T>): Promise<T | undefined> {
  const queued = mutationTail.then(() => runMutation(operation), () => runMutation(operation))
  mutationTail = queued.then(() => undefined, () => undefined)
  return queued
}

async function initialize() {
  if (ready.value) return
  if (initializingPromise) return initializingPromise

  const generation = ++initializationGeneration
  const operation = (async () => {
    if (!isTauriRuntime()) {
      if (generation === initializationGeneration) ready.value = true
      return
    }

    error.value = undefined
    let failedAttemptIndex = 0
    while (generation === initializationGeneration && !ready.value) {
      let unsubscribe: (() => void) | undefined
      try {
        stopStateListener?.()
        stopStateListener = undefined
        try {
          unsubscribe = makeIdempotent(
            await listen<AppState>('qpets://state-changed', (event) => applyIncomingState(event.payload)),
          )
        } catch {
          const snapshot = await invoke<AppState>('get_app_state')
          if (generation !== initializationGeneration) return
          applyIncomingState(snapshot)
          ready.value = true
          error.value = undefined
          void recoverStateListener(generation)
          return
        }
        if (generation !== initializationGeneration) {
          unsubscribe()
          return
        }
        stopStateListener = unsubscribe
        // Subscribe before reading the snapshot: an update before this request is
        // included by the snapshot, and an update afterwards is received by the listener.
        const snapshot = await invoke<AppState>('get_app_state')
        if (generation !== initializationGeneration) return
        applyIncomingState(snapshot)
        ready.value = true
        error.value = undefined
      } catch (cause) {
        if (unsubscribe && stopStateListener === unsubscribe) {
          unsubscribe()
          stopStateListener = undefined
        } else {
          unsubscribe?.()
        }
        if (generation !== initializationGeneration) return
        const delay = retryDelayAfterFailure(RUNTIME_INITIALIZE_RETRY_DELAYS_MS, failedAttemptIndex)
        if (delay === undefined) {
          ready.value = false
          error.value = `应用初始化失败：${messageFrom(cause)}`
          break
        }
        failedAttemptIndex += 1
        await waitForRetry(delay)
      }
    }
  })()
  initializingPromise = operation
  void operation.finally(() => {
    if (initializingPromise === operation) initializingPromise = undefined
  })

  return initializingPromise
}

async function selectPet(petId: string) {
  if (!isTauriRuntime()) {
    state.selectedPetId = petId
    return
  }
  await enqueueMutation(async () => {
    const next = await invoke<AppState>('select_pet', { petId })
    applyIncomingState(next)
  })
}

function updateSettings(settings: AppSettings): Promise<void> {
  const previous = { ...state.settings }
  settingsIntent += 1
  state.settings = { ...settings }
  if (!isTauriRuntime()) {
    settledSettingsIntent = settingsIntent
    return Promise.resolve()
  }

  pendingSettingsBefore ??= previous
  pendingSettings = { ...settings }

  return new Promise((resolve) => {
    pendingSettingsResolvers.push(resolve)
    if (settingsTimer) clearTimeout(settingsTimer)
    settingsTimer = setTimeout(() => {
      settingsTimer = undefined
      void flushPendingSettings()
    }, 120)
  })
}

async function flushPendingSettings() {
  const settings = pendingSettings
  const previous = pendingSettingsBefore
  const intent = settingsIntent
  const resolvers = pendingSettingsResolvers
  pendingSettings = undefined
  pendingSettingsBefore = undefined
  pendingSettingsResolvers = []
  if (!settings) {
    resolvers.forEach(resolve => resolve())
    return
  }

  const result = await enqueueMutation(() => invoke<AppState>('update_settings', { settings }))
  if (result) {
    confirmedSettings = { ...result.settings }
    if (intent === settingsIntent) {
      settledSettingsIntent = intent
      replaceState(result)
    }
  } else if (previous && intent === settingsIntent) {
    settledSettingsIntent = intent
    state.settings = { ...confirmedSettings }
  }
  resolvers.forEach(resolve => resolve())
}

async function addStaticPet() {
  if (!isTauriRuntime()) {
    error.value = '浏览器预览模式不能导入本地图片。'
    return
  }
  const sourcePath = await open({
    multiple: false,
    directory: false,
    filters: [{ name: '透明角色图片', extensions: ['png', 'webp'] }],
  })
  if (typeof sourcePath !== 'string') return
  await enqueueMutation(async () => {
    await invoke<PetDescriptor>('import_static_pet', { sourcePath })
    applyIncomingState(await invoke<AppState>('get_app_state'))
  })
}

async function importPetPackage() {
  if (!isTauriRuntime()) {
    error.value = '浏览器预览模式不能导入角色包。'
    return
  }
  const sourcePath = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Qpets 角色包', extensions: ['qpet', 'zip'] }],
  })
  if (typeof sourcePath !== 'string') return
  await enqueueMutation(async () => {
    await invoke<PetDescriptor>('import_pet_package', { sourcePath })
    applyIncomingState(await invoke<AppState>('get_app_state'))
  })
}

async function deletePet(petId: string) {
  if (!isTauriRuntime()) {
    state.pets = state.pets.filter((pet) => pet.id !== petId)
    if (state.selectedPetId === petId) state.selectedPetId = MOCK_STATE.selectedPetId
    return
  }
  await enqueueMutation(async () => applyIncomingState(await invoke<AppState>('delete_pet', { petId })))
}

async function updatePetMetadata(petId: string, metadata: PetMetadataDraft): Promise<boolean> {
  if (!isTauriRuntime()) {
    const pet = state.pets.find(item => item.id === petId)
    if (!pet) return false
    pet.displayName = metadata.displayName
    pet.description = metadata.description
    pet.metadataCustomized = true
    return true
  }
  const result = await enqueueMutation(() => invoke<AppState>('update_pet_metadata', {
    petId,
    displayName: metadata.displayName,
    description: metadata.description,
  }))
  if (!result) return false
  applyIncomingState(result)
  return true
}

async function resetPetMetadata(petId: string): Promise<boolean> {
  if (!isTauriRuntime()) {
    const pet = state.pets.find(item => item.id === petId)
    const original = MOCK_STATE.pets.find(item => item.id === petId)
    if (!pet || !original) return false
    pet.displayName = original.displayName
    pet.description = original.description
    pet.metadataCustomized = false
    return true
  }
  const result = await enqueueMutation(() => invoke<AppState>('reset_pet_metadata', { petId }))
  if (!result) return false
  applyIncomingState(result)
  return true
}

async function openSettings() {
  if (isTauriRuntime()) await invoke('open_settings')
}

async function visitHomepage(url: string) {
  error.value = undefined
  try {
    if (isTauriRuntime()) await openUrl(url)
    else window.open(url, '_blank', 'noopener,noreferrer')
  } catch (cause) {
    error.value = `无法打开项目主页：${messageFrom(cause)}`
  }
}

export function dismissError() {
  error.value = undefined
}

export function disposeAppRuntime() {
  initializationGeneration += 1
  initializingPromise = undefined
  if (settingsTimer) clearTimeout(settingsTimer)
  settingsTimer = undefined
  pendingSettingsResolvers.forEach(resolve => resolve())
  pendingSettingsResolvers = []
  pendingSettings = undefined
  pendingSettingsBefore = undefined
  stopStateListener?.()
  stopStateListener = undefined
  ready.value = false
}

export function useAppRuntime() {
  return {
    state: readonly(state),
    selectedPet: computed(() => state.pets.find((pet) => pet.id === state.selectedPetId) ?? state.pets[0]),
    ready: readonly(ready),
    busy: readonly(busy),
    error: readonly(error),
    initialize,
    selectPet,
    updateSettings,
    addStaticPet,
    importPetPackage,
    deletePet,
    updatePetMetadata,
    resetPetMetadata,
    openSettings,
    visitHomepage,
    dismissError,
    dispose: disposeAppRuntime,
  }
}
