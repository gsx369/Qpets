<script setup lang="ts">
import { cursorPosition, getCurrentWindow } from '@tauri-apps/api/window'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { isTauriRuntime, resolveAssetUrl, useAppRuntime } from '../shared/runtime'
import type { PetAction } from '../shared/types'
import AtlasPet from './AtlasPet.vue'
import StaticPet from './StaticPet.vue'
import { vectorToLookIndex } from './atlas'
import '../styles/pet.css'

const runtime = useAppRuntime()
const action = ref<PetAction>('idle')
const actionToken = ref(0)
const lookIndex = ref<number>()
const bubbleText = ref('')
const assetLoadFailed = ref(false)

const selectedPet = computed(() => runtime.selectedPet.value)
const source = computed(() => resolveAssetUrl(selectedPet.value?.assetPath ?? ''))

let idleTimer: ReturnType<typeof setTimeout> | undefined
let bubbleTimer: ReturnType<typeof setTimeout> | undefined
let actionTimer: ReturnType<typeof setTimeout> | undefined
let gazeTimer: ReturnType<typeof setInterval> | undefined
let gazePolling = false
let clickIndex = 0
let dragStart: { x: number; y: number } | undefined
let dragging = false

const clickActions: PetAction[] = ['waving', 'review', 'waving', 'jumping']
const idleActions: PetAction[] = ['waiting', 'review', 'waiting', 'jumping']

function dialogueKey(nextAction: PetAction) {
  if (nextAction === 'failed') return 'error'
  if (nextAction === 'waving' || nextAction === 'jumping') return 'success'
  if (nextAction === 'waiting' || nextAction === 'review' || nextAction === 'running') return 'working'
  return 'idle'
}

function showDialogue(nextAction: PetAction) {
  if (!runtime.state.settings.showBubble || !selectedPet.value) return
  const lines = selectedPet.value.dialogues[dialogueKey(nextAction)] ?? selectedPet.value.dialogues.idle ?? []
  if (!lines.length) return
  bubbleText.value = lines[Math.floor(Math.random() * lines.length)]
  if (bubbleTimer) clearTimeout(bubbleTimer)
  bubbleTimer = setTimeout(() => { bubbleText.value = '' }, 2_600)
}

function scheduleIdle() {
  if (idleTimer) clearTimeout(idleTimer)
  const base = runtime.state.settings.idleIntervalSeconds * 1_000
  idleTimer = setTimeout(() => {
    const next = idleActions[Math.floor(Math.random() * idleActions.length)]
    setAction(next, Math.random() < 0.45)
  }, base * (0.8 + Math.random() * 0.4))
}

function setAction(next: PetAction, withDialogue = false) {
  if (action.value === next && next === 'look') return
  action.value = next
  actionToken.value += 1
  if (next !== 'look') lookIndex.value = undefined
  if (withDialogue) showDialogue(next)
  if (actionTimer) clearTimeout(actionTimer)

  if (selectedPet.value?.renderType === 'static-image-v1' && !['idle', 'look', 'running-left', 'running-right'].includes(next)) {
    actionTimer = setTimeout(finishAction, 950)
  }
  if (next === 'idle') scheduleIdle()
}

function finishAction() {
  if (action.value === 'idle') return
  setAction('idle')
}

function interact() {
  const next = clickActions[clickIndex % clickActions.length]
  clickIndex += 1
  setAction(next, true)
}

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return
  dragStart = { x: event.screenX, y: event.screenY }
  dragging = false
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
}

function onPointerMove(event: PointerEvent) {
  if (!dragStart || dragging) return
  const dx = event.screenX - dragStart.x
  const dy = event.screenY - dragStart.y
  if (Math.hypot(dx, dy) < 6) return
  dragging = true
  setAction(dx < 0 ? 'running-left' : 'running-right')
  if (isTauriRuntime()) void getCurrentWindow().startDragging()
  actionTimer = setTimeout(finishAction, 1_200)
}

function onPointerUp() {
  if (!dragStart) return
  const wasDragging = dragging
  dragStart = undefined
  dragging = false
  if (wasDragging) actionTimer = setTimeout(finishAction, 240)
  else interact()
}

function onWheel(event: WheelEvent) {
  event.preventDefault()
  const step = event.deltaY < 0 ? 0.05 : -0.05
  const scale = Math.min(1.6, Math.max(0.6, Math.round((runtime.state.settings.scale + step) * 20) / 20))
  void runtime.updateSettings({ ...runtime.state.settings, scale })
}

async function pollGaze() {
  if (gazePolling || !isTauriRuntime() || !runtime.state.settings.followCursor || !['idle', 'look'].includes(action.value)) return
  gazePolling = true
  try {
    const window = getCurrentWindow()
    const [cursor, position, size] = await Promise.all([cursorPosition(), window.outerPosition(), window.outerSize()])
    const index = vectorToLookIndex(
      cursor.x - (position.x + size.width / 2),
      cursor.y - (position.y + size.height / 2),
      54,
    )
    if (index === undefined) {
      if (action.value === 'look') setAction('idle')
    } else if (lookIndex.value !== index || action.value !== 'look') {
      lookIndex.value = index
      action.value = 'look'
    }
  } catch {
    // Window position can be temporarily unavailable while Windows moves it.
  } finally {
    gazePolling = false
  }
}

watch(() => selectedPet.value?.id, () => {
  assetLoadFailed.value = false
  bubbleText.value = ''
  setAction('idle')
})

watch(() => runtime.state.settings.idleIntervalSeconds, scheduleIdle)

onMounted(() => {
  void runtime.initialize()
  scheduleIdle()
  gazeTimer = setInterval(() => void pollGaze(), 100)
})

onBeforeUnmount(() => {
  if (idleTimer) clearTimeout(idleTimer)
  if (bubbleTimer) clearTimeout(bubbleTimer)
  if (actionTimer) clearTimeout(actionTimer)
  if (gazeTimer) clearInterval(gazeTimer)
})
</script>

<template>
  <main
    class="pet-window"
    :aria-label="selectedPet ? `${selectedPet.displayName}桌宠` : 'Qpets 桌宠'"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @wheel="onWheel"
    @contextmenu.prevent="runtime.openSettings"
  >
    <Transition name="bubble">
      <div v-if="bubbleText" class="pet-bubble" role="status">{{ bubbleText }}</div>
    </Transition>

    <div class="pet-stage" :class="{ 'is-busy': runtime.busy.value }">
      <template v-if="selectedPet">
        <AtlasPet
          v-if="selectedPet.renderType === 'sprite-atlas-v2' && source && !assetLoadFailed"
          :key="selectedPet.id"
          :source="source"
          :action="action"
          :action-token="actionToken"
          :look-index="lookIndex"
          @complete="finishAction"
          @load-error="assetLoadFailed = true"
        />
        <StaticPet
          v-else
          :key="`${selectedPet.id}-${actionToken}`"
          :source="assetLoadFailed ? '' : source"
          :name="selectedPet.displayName"
          :action="action"
          :look-index="lookIndex"
        />
      </template>
    </div>

    <button v-if="runtime.error.value" type="button" class="pet-error" title="打开设置查看" @click.stop="runtime.openSettings">!</button>
  </main>
</template>
