<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

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
let animationFrame = 0
let startedAt = performance.now()
let completionSent = false
let loaded = false

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
  animationFrame = requestAnimationFrame(draw)
})

onBeforeUnmount(() => cancelAnimationFrame(animationFrame))
</script>

<template>
  <canvas ref="canvas" class="atlas-pet" :width="CELL_WIDTH" :height="CELL_HEIGHT" aria-hidden="true" />
</template>
