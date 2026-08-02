<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import type { PetAction } from '../shared/types'
import { IMAGE_LOAD_RETRY_DELAYS_MS, retryDelayAfterFailure } from '../shared/retry'
import { CELL_HEIGHT, CELL_WIDTH, frameAt, lookIndexToFrame } from './atlas'

const props = defineProps<{
  source: string
  action: PetAction
  actionToken: number
  lookIndex?: number
}>()

const emit = defineEmits<{
  complete: []
  loadError: []
}>()

const canvas = ref<HTMLCanvasElement>()
let image = new Image()
const displayWidth = ref(CELL_WIDTH)
const displayHeight = ref(CELL_HEIGHT)
const backingWidth = ref(CELL_WIDTH)
const backingHeight = ref(CELL_HEIGHT)
const canvasStyle = computed(() => ({
  width: `${displayWidth.value}px`,
  height: `${displayHeight.value}px`,
}))
let animationFrame = 0
let startedAt = performance.now()
let completionSent = false
let loaded = false
let resizeObserver: ResizeObserver | undefined
let loadGeneration = 0
let retryTimer: ReturnType<typeof setTimeout> | undefined
let activeImage: HTMLImageElement | undefined

function resetAnimation() {
  startedAt = performance.now()
  completionSent = false
}

function draw(timestamp: number) {
  const context = canvas.value?.getContext('2d')
  if (context && loaded) {
    const frame = props.action === 'look' && props.lookIndex !== undefined
      ? lookIndexToFrame(props.lookIndex)
      : frameAt(props.action === 'look' ? 'idle' : props.action, timestamp - startedAt)

    const scaleX = backingWidth.value / CELL_WIDTH
    const scaleY = backingHeight.value / CELL_HEIGHT
    context.setTransform(scaleX, 0, 0, scaleY, 0, 0)
    context.clearRect(0, 0, CELL_WIDTH, CELL_HEIGHT)
    context.imageSmoothingEnabled = true
    context.imageSmoothingQuality = 'high'
    context.drawImage(
      image,
      frame.column * CELL_WIDTH,
      frame.row * CELL_HEIGHT,
      CELL_WIDTH,
      CELL_HEIGHT,
      0,
      0,
      CELL_WIDTH,
      CELL_HEIGHT,
    )

    if (frame.complete && !completionSent) {
      completionSent = true
      emit('complete')
    }
  }
  animationFrame = requestAnimationFrame(draw)
}

function resizeCanvas() {
  const container = canvas.value?.parentElement
  if (!container) return

  const bounds = container.getBoundingClientRect()
  if (bounds.width <= 0 || bounds.height <= 0) return

  const aspectRatio = CELL_WIDTH / CELL_HEIGHT
  const height = Math.min(bounds.height, bounds.width / aspectRatio)
  const width = height * aspectRatio
  const dpr = Math.max(1, window.devicePixelRatio || 1)

  displayWidth.value = width
  displayHeight.value = height
  backingWidth.value = Math.max(1, Math.round(width * dpr))
  backingHeight.value = Math.max(1, Math.round(height * dpr))
}

function clearCanvas() {
  const element = canvas.value
  const context = element?.getContext('2d')
  if (!element || !context) return
  context.setTransform(1, 0, 0, 1, 0, 0)
  context.clearRect(0, 0, element.width, element.height)
}

function cancelPendingLoad() {
  if (retryTimer) clearTimeout(retryTimer)
  retryTimer = undefined
  if (activeImage) {
    activeImage.onload = null
    activeImage.onerror = null
    activeImage = undefined
  }
}

function tryLoadSource(generation: number, failedAttemptIndex: number) {
  const candidate = new Image()
  candidate.decoding = 'async'
  activeImage = candidate
  candidate.onload = () => {
    if (generation !== loadGeneration) return
    candidate.onload = null
    candidate.onerror = null
    activeImage = undefined
    image = candidate
    loaded = true
    resetAnimation()
  }
  candidate.onerror = () => {
    if (generation !== loadGeneration) return
    candidate.onload = null
    candidate.onerror = null
    activeImage = undefined
    const delay = retryDelayAfterFailure(IMAGE_LOAD_RETRY_DELAYS_MS, failedAttemptIndex)
    if (delay === undefined) {
      emit('loadError')
      return
    }
    retryTimer = setTimeout(() => {
      retryTimer = undefined
      tryLoadSource(generation, failedAttemptIndex + 1)
    }, delay)
  }
  candidate.src = props.source
}

function loadSource() {
  loadGeneration += 1
  cancelPendingLoad()
  loaded = false
  clearCanvas()
  if (!props.source) return
  tryLoadSource(loadGeneration, 0)
}

watch(() => props.source, loadSource)
watch(() => [props.action, props.actionToken, props.lookIndex], resetAnimation)

onMounted(() => {
  loadSource()
  void nextTick(resizeCanvas)
  if (canvas.value?.parentElement) {
    resizeObserver = new ResizeObserver(resizeCanvas)
    resizeObserver.observe(canvas.value.parentElement)
  }
  animationFrame = requestAnimationFrame(draw)
})

onBeforeUnmount(() => {
  loadGeneration += 1
  cancelPendingLoad()
  resizeObserver?.disconnect()
  cancelAnimationFrame(animationFrame)
})
</script>

<template>
  <canvas
    ref="canvas"
    class="atlas-pet"
    :width="backingWidth"
    :height="backingHeight"
    :style="canvasStyle"
    aria-hidden="true"
  />
</template>
