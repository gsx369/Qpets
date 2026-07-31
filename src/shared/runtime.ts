import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, reactive, readonly, ref } from 'vue'

import { MOCK_STATE } from './mock-state'
import type { AppSettings, AppState, PetDescriptor } from './types'

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
let initialized = false

function replaceState(next: AppState) {
  state.settings = { ...next.settings }
  state.selectedPetId = next.selectedPetId
  state.pets = next.pets.map((pet) => ({ ...pet, dialogues: { ...pet.dialogues } }))
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

async function initialize() {
  if (initialized) return
  initialized = true

  if (!isTauriRuntime()) {
    ready.value = true
    return
  }

  try {
    replaceState(await invoke<AppState>('get_app_state'))
    await listen<AppState>('qpets://state-changed', (event) => replaceState(event.payload))
  } catch (cause) {
    error.value = messageFrom(cause)
  } finally {
    ready.value = true
  }
}

async function selectPet(petId: string) {
  if (!isTauriRuntime()) {
    state.selectedPetId = petId
    return
  }
  await runMutation(async () => replaceState(await invoke<AppState>('select_pet', { petId })))
}

async function updateSettings(settings: AppSettings) {
  const previous = { ...state.settings }
  state.settings = { ...settings }
  if (!isTauriRuntime()) return

  const result = await runMutation(() => invoke<AppState>('update_settings', { settings }))
  if (result) replaceState(result)
  else state.settings = previous
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
  await runMutation(async () => {
    await invoke<PetDescriptor>('import_static_pet', { sourcePath })
    replaceState(await invoke<AppState>('get_app_state'))
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
  await runMutation(async () => {
    await invoke<PetDescriptor>('import_pet_package', { sourcePath })
    replaceState(await invoke<AppState>('get_app_state'))
  })
}

async function deletePet(petId: string) {
  if (!isTauriRuntime()) {
    state.pets = state.pets.filter((pet) => pet.id !== petId)
    if (state.selectedPetId === petId) state.selectedPetId = MOCK_STATE.selectedPetId
    return
  }
  await runMutation(async () => replaceState(await invoke<AppState>('delete_pet', { petId })))
}

async function openSettings() {
  if (isTauriRuntime()) await invoke('open_settings')
}

async function visitHomepage(url: string) {
  if (isTauriRuntime()) await openUrl(url)
  else window.open(url, '_blank', 'noopener,noreferrer')
}

export function dismissError() {
  error.value = undefined
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
    openSettings,
    visitHomepage,
    dismissError,
  }
}
