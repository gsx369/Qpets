<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { IMAGE_LOAD_RETRY_DELAYS_MS, retryDelayAfterFailure } from '../shared/retry'
import type { PetAction } from '../shared/types'

const props = defineProps<{
  source: string
  name: string
  action: PetAction
  lookIndex?: number
}>()

const emit = defineEmits<{
  loadError: []
}>()

const failedAttemptIndex = ref(0)
const imageVisible = ref(false)
let retryTimer: ReturnType<typeof setTimeout> | undefined

function resetImage() {
  if (retryTimer) clearTimeout(retryTimer)
  retryTimer = undefined
  failedAttemptIndex.value = 0
  imageVisible.value = Boolean(props.source)
}

function onImageError(event: Event) {
  const failedSource = (event.currentTarget as HTMLImageElement | null)?.getAttribute('src')
  if (failedSource !== props.source) return
  imageVisible.value = false
  const delay = retryDelayAfterFailure(IMAGE_LOAD_RETRY_DELAYS_MS, failedAttemptIndex.value)
  if (delay === undefined) {
    emit('loadError')
    return
  }
  retryTimer = setTimeout(() => {
    retryTimer = undefined
    failedAttemptIndex.value += 1
    imageVisible.value = Boolean(props.source)
  }, delay)
}

watch(() => props.source, resetImage, { immediate: true })
onBeforeUnmount(() => {
  if (retryTimer) clearTimeout(retryTimer)
})

const gazeStyle = computed(() => {
  if (props.lookIndex === undefined) return undefined
  const radians = (props.lookIndex * 22.5 * Math.PI) / 180
  return {
    '--gaze-x': `${Math.sin(radians) * 5}px`,
    '--gaze-y': `${-Math.cos(radians) * 4}px`,
    '--gaze-rotate': `${Math.sin(radians) * 2.2}deg`,
  }
})
</script>

<template>
  <div class="static-pet" :class="`static-pet--${action}`" :style="gazeStyle">
    <img
      v-if="source && imageVisible"
      :key="`${source}-${failedAttemptIndex}`"
      :src="source"
      :alt="name"
      draggable="false"
      @error="onImageError"
    />
    <div v-else class="pet-loading" aria-hidden="true"><span /></div>
  </div>
</template>
