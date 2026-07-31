<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import type { PetAction } from '../shared/types'
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
const image = new Image()
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

function loadSource() {
  loaded = false
  if (!props.source) return
  image.onload = () => {
    loaded = true
    resetAnimation()
  }
  image.onerror = () => emit('loadError')
  image.src = props.source
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
